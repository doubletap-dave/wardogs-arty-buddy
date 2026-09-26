use std::time::Duration;

use eframe::egui;
use egui::{Align, Color32, Frame, Layout, Margin, RichText, ViewportCommand};

use crate::hud::{range_well, readout};
use crate::ink::{Ink, ink_squares};
use crate::range::{ClipUpdate, absorb_wardogs, read_board, update_from_clip};
use crate::theme::{self, GUTTER, PAD, WINDOW_SIZE, mono};
use crate::transparency::hold_transparency;
use crate::wave::{paint_chromatic_plate, paint_wave};

pub(crate) struct ArtyBuddy {
    you_x: String,
    you_y: String,
    enemy_x: String,
    enemy_y: String,
    ink: Ink,
    locked_ppp: f32,
    own_locked: bool,
    last_clip: String,
    last_clip_poll: f64,
    clipboard: Option<arboard::Clipboard>,
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
            locked_ppp: 0.0,
            own_locked: false,
            last_clip: String::new(),
            last_clip_poll: 0.0,
            clipboard: arboard::Clipboard::new().ok(),
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
            });
        });
        ui.add_space(6.0);
        readout(ui, "OWN", &self.you_x, &self.you_y, ink);
        readout(ui, "TGT", &self.enemy_x, &self.enemy_y, ink);
        ui.add_space(6.0);
        range_well(ui, reading, self.ink);
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
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        hold_transparency(frame);
        self.sync_window(ui.ctx());
        absorb_wardogs(&mut self.you_x, &mut self.you_y);
        absorb_wardogs(&mut self.enemy_x, &mut self.enemy_y);
        let reading = read_board(&self.you_x, &self.you_y, &self.enemy_x, &self.enemy_y);
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
