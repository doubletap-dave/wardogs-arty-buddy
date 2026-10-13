use std::time::Duration;

use eframe::egui;
use egui::{Align, Color32, Frame, Layout, Margin, RichText, ViewportCommand};

use crate::hud::{range_well, readout};
use crate::ink::{Ink, ink_squares};
use crate::range::{ClipUpdate, absorb_wardogs, read_board_with_dz, update_from_clip};
use crate::terrain::{self, Map};
use crate::theme::{self, GUTTER, PAD, WINDOW_SIZE, mono};
use crate::transparency::hold_transparency;
use crate::update::{DownloadedUpdate, UpdateCheck};
use crate::wave::{paint_chromatic_plate, paint_wave};

pub(crate) struct ArtyBuddy {
    you_x: String,
    you_y: String,
    enemy_x: String,
    enemy_y: String,
    ink: Ink,
    map: Map,
    locked_ppp: f32,
    own_locked: bool,
    last_clip: String,
    last_clip_poll: f64,
    clipboard: Option<arboard::Clipboard>,
    update_check: UpdateCheck,
    pending_update: Option<DownloadedUpdate>,
    install_at: Option<f64>,
}

impl ArtyBuddy {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::install_style(&cc.egui_ctx);
        cc.egui_ctx.set_zoom_factor(1.0);
        Self {
            you_x: String::new(),
            you_y: String::new(),
            enemy_x: String::new(),
            enemy_y: String::new(),
            ink: Ink::Green,
            map: Map::Bakurani,
            locked_ppp: 0.0,
            own_locked: false,
            last_clip: String::new(),
            last_clip_poll: 0.0,
            clipboard: arboard::Clipboard::new().ok(),
            update_check: UpdateCheck::new(),
            pending_update: None,
            install_at: None,
        }
    }

    fn sync_window(&mut self, ctx: &egui::Context) {
        let ppp = ctx.pixels_per_point();
        if (ppp - self.locked_ppp).abs() <= 0.01 {
            return;
        }
        self.locked_ppp = ppp;
        // Min and max are stored as physical pixels. Re-send them when the
        // monitor scale changes, or the old pixel size sticks on the new screen.
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(WINDOW_SIZE));
        ctx.send_viewport_cmd(ViewportCommand::MaxInnerSize(WINDOW_SIZE));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(WINDOW_SIZE));
    }

    fn watch_clipboard(&mut self) {
        let Some(clipboard) = self.clipboard.as_mut() else {
            return;
        };
        let Ok(raw) = clipboard.get_text() else {
            return;
        };
        let text = raw.trim();
        if text.is_empty() || text == self.last_clip {
            return;
        }
        let Some(update) = update_from_clip(self.own_locked, text) else {
            self.last_clip = text.to_owned();
            return;
        };
        match update {
            ClipUpdate::Own { x, y } => {
                self.you_x = x;
                self.you_y = y;
                self.enemy_x.clear();
                self.enemy_y.clear();
                self.own_locked = true;
            }
            ClipUpdate::Target { x, y } => {
                self.enemy_x = x;
                self.enemy_y = y;
            }
        }
        self.last_clip = text.to_owned();
        if clipboard.clear().is_ok() {
            self.last_clip.clear();
        }
    }

    fn clear(&mut self) {
        self.you_x.clear();
        self.you_y.clear();
        self.enemy_x.clear();
        self.enemy_y.clear();
        self.own_locked = false;
        self.last_clip.clear();
    }

    fn solve_board(&self) -> crate::range::BoardRead {
        let gun = terrain::elevation_of(self.map, &self.you_x, &self.you_y);
        let target = terrain::elevation_of(self.map, &self.enemy_x, &self.enemy_y);
        let dz = gun.zip(target).map(|(g, t)| t - g);
        read_board_with_dz(&self.you_x, &self.you_y, &self.enemy_x, &self.enemy_y, dz)
    }

    fn hud(&mut self, ui: &mut egui::Ui, reading: &crate::range::BoardRead) {
        let ink = self.ink.color();
        ui.horizontal(|ui| {
            ink_squares(ui, &mut self.ink);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(egui::Button::new(
                        RichText::new("CLEAR").font(mono(12.0)).color(ink),
                    ))
                    .clicked()
                {
                    self.clear();
                }
                if let Some(update) = &self.pending_update {
                    let label = if self.install_at.is_some() {
                        format!("INSTALLING {}", update.version)
                    } else {
                        format!("UPDATE {}", update.version)
                    };
                    let button =
                        egui::Button::new(RichText::new(label).font(mono(12.0)).color(ink));
                    if ui.add(button).clicked() && self.install_at.is_none() {
                        self.install_at = Some(0.0);
                    }
                }
            });
        });
        ui.add_space(6.0);
        readout(ui, "OWN", &self.you_x, &self.you_y, ink);
        readout(ui, "TGT", &self.enemy_x, &self.enemy_y, ink);
        ui.add_space(4.0);
        crate::hud::elevation_row(
            ui,
            self.map,
            &self.you_x,
            &self.you_y,
            &self.enemy_x,
            &self.enemy_y,
            ink,
        );
        ui.add_space(6.0);
        range_well(ui, reading, self.ink);
        ui.add_space(6.0);
        crate::hud::map_row(ui, &mut self.map, ink);
    }
}

impl eframe::App for ArtyBuddy {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = if self.ink.palette().is_some() {
            32
        } else {
            200
        };
        ctx.request_repaint_after(Duration::from_millis(frame));
        let now = ctx.input(|input| input.time);
        if now - self.last_clip_poll >= 0.2 {
            self.last_clip_poll = now;
            self.watch_clipboard();
        }
        if let Some(update) = self.update_check.poll(now, self.pending_update.is_some()) {
            if update.startup {
                self.install_at = Some(now + 3.0);
            }
            self.pending_update = Some(update);
        }
        let install_now = self.install_at.is_some_and(|at| now >= at);
        if install_now {
            if let Some(update) = self.pending_update.take() {
                if crate::update::apply(&update.path) {
                    std::process::exit(0);
                }
            }
            self.install_at = None;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        hold_transparency(frame);
        self.sync_window(ui.ctx());
        absorb_wardogs(&mut self.you_x, &mut self.you_y);
        absorb_wardogs(&mut self.enemy_x, &mut self.enemy_y);
        let reading = self.solve_board();
        let palette = self.ink.palette();
        let phase = (ui.input(|input| input.time) as f32 / 6.5).fract();
        if let Some(palette) = palette {
            paint_chromatic_plate(ui.painter(), ui.max_rect(), phase, palette);
        }
        let plate = Color32::from_rgba_unmultiplied(8, 12, 8, 176);
        let panel = egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(if palette.is_some() {
                        Color32::TRANSPARENT
                    } else {
                        plate
                    })
                    .inner_margin(Margin::same(PAD + GUTTER)),
            )
            .show(ui, |ui| {
                self.hud(ui, &reading);
            });
        if let Some(palette) = palette {
            paint_wave(
                ui.painter(),
                panel.response.rect.shrink(9.0),
                phase,
                palette,
                2.2,
                8.0,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::range::BoardRead;

    fn test_app() -> ArtyBuddy {
        ArtyBuddy {
            you_x: String::new(),
            you_y: String::new(),
            enemy_x: String::new(),
            enemy_y: String::new(),
            ink: Ink::Green,
            map: Map::Bakurani,
            locked_ppp: 0.0,
            own_locked: false,
            last_clip: String::new(),
            last_clip_poll: 0.0,
            clipboard: None,
            update_check: UpdateCheck::new(),
            pending_update: None,
            install_at: None,
        }
    }

    #[test]
    fn clear_resets_both_own_and_target_and_unlocks_own() {
        let mut app = test_app();
        app.you_x = "94.32".into();
        app.you_y = "108.68".into();
        app.enemy_x = "88.63".into();
        app.enemy_y = "99.76".into();
        app.own_locked = true;
        app.last_clip = "x88.63, y99.76".into();

        app.clear();

        assert!(app.you_x.is_empty(), "you_x should be cleared");
        assert!(app.you_y.is_empty(), "you_y should be cleared");
        assert!(app.enemy_x.is_empty(), "enemy_x should be cleared");
        assert!(app.enemy_y.is_empty(), "enemy_y should be cleared");
        assert!(!app.own_locked, "own_locked should be reset to false");
        assert!(app.last_clip.is_empty(), "last_clip should be cleared");
    }

    #[test]
    fn switching_map_recalculates_effective_range_using_map_elevation() {
        let mut app = test_app();
        app.you_x = "94.32".into();
        app.you_y = "108.68".into();
        app.enemy_x = "88.63".into();
        app.enemy_y = "99.76".into();

        app.map = Map::Bakurani;
        let bak_fix = match app.solve_board() {
            BoardRead::Ready(fix) => fix,
            other => panic!("expected Ready on Bakurani, got {other:?}"),
        };

        app.map = Map::Ozeti;
        let oze_fix = match app.solve_board() {
            BoardRead::Ready(fix) => fix,
            other => panic!("expected Ready on Ozeti, got {other:?}"),
        };

        assert!(
            (bak_fix.meters - bak_fix.flat_meters).abs() > 10.0,
            "expected elevation-adjusted range to differ from flat range ({}), got {}",
            bak_fix.flat_meters,
            bak_fix.meters
        );
        assert!(
            (bak_fix.meters - oze_fix.meters).abs() > 1.0,
            "switching map from Bakurani ({}) to Ozeti ({}) should recalculate range",
            bak_fix.meters,
            oze_fix.meters
        );
    }
}
