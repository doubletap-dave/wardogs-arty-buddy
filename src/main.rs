#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod range;

use std::sync::Arc;
use std::time::Duration;

use eframe::egui;
use egui::{
    Align, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame,
    Layout, Margin, Rect, RichText, Sense, Stroke, StrokeKind, Vec2, ViewportCommand, pos2,
    vec2,
};

use range::{BoardRead, ClipUpdate, absorb_wardogs, format_number, read_board, update_from_clip};

const CANVAS: Color32 = Color32::from_rgb(0x1C, 0x24, 0x16);
const PANEL: Color32 = Color32::from_rgb(0x24, 0x2C, 0x1A);
const WELL: Color32 = Color32::from_rgb(0x10, 0x14, 0x0B);
const INK: Color32 = Color32::from_rgb(0xD8, 0xD0, 0xB8);
const TAN: Color32 = Color32::from_rgb(0xC4, 0xB4, 0x8A);
const AMBER: Color32 = Color32::from_rgb(0xE2, 0xA2, 0x2A);
const OLIVE: Color32 = Color32::from_rgb(0x7D, 0x8A, 0x52);
const LINE: Color32 = Color32::from_rgb(0x4E, 0x5A, 0x38);
const TITLE: &str = concat!("Wardogs Arty Buddy ", env!("CARGO_PKG_VERSION"));
const PAD: i8 = 10;
const WINDOW_SIZE: Vec2 = vec2(400.0, 196.0);

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(WINDOW_SIZE)
            .with_max_inner_size(WINDOW_SIZE)
            .with_resizable(false)
            .with_maximize_button(false)
            .with_always_on_top()
            .with_transparent(true)
            .with_title(TITLE),
        ..Default::default()
    };

    eframe::run_native(
        TITLE,
        options,
        Box::new(|cc| Ok(Box::new(ArtyBuddy::new(cc)))),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ink {
    Green,
    Red,
    Blue,
    White,
    Amber,
}

impl Ink {
    const ALL: [Self; 5] = [Self::Green, Self::Red, Self::Blue, Self::White, Self::Amber];

    fn color(self) -> Color32 {
        match self {
            Self::Green => Color32::from_rgb(0x6B, 0xF0, 0x4A),
            Self::Red => Color32::from_rgb(0xFF, 0x45, 0x3A),
            Self::Blue => Color32::from_rgb(0x3D, 0xA5, 0xFF),
            Self::White => Color32::from_rgb(0xF4, 0xF4, 0xF4),
            Self::Amber => Color32::from_rgb(0xE2, 0xA2, 0x2A),
        }
    }
}

struct ArtyBuddy {
    you_x: String,
    you_y: String,
    enemy_x: String,
    enemy_y: String,
    ink: Ink,
    locked_ppp: f32,
    own_locked: bool,
    last_clip: String,
    clipboard: Option<arboard::Clipboard>,
}

impl ArtyBuddy {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        install_style(&cc.egui_ctx);
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
}

impl eframe::App for ArtyBuddy {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(200));
        self.watch_clipboard();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.sync_window(ui.ctx());
        absorb_wardogs(&mut self.you_x, &mut self.you_y);
        absorb_wardogs(&mut self.enemy_x, &mut self.enemy_y);
        let reading = read_board(&self.you_x, &self.you_y, &self.enemy_x, &self.enemy_y);
        let plate = Color32::from_rgba_unmultiplied(8, 12, 8, 176);
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(plate)
                    .inner_margin(Margin::same(PAD)),
            )
            .show(ui, |ui| {
                self.hud(ui, &reading);
            });
    }
}

impl ArtyBuddy {
    fn hud(&mut self, ui: &mut egui::Ui, reading: &BoardRead) {
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
        range_well(ui, reading, ink);
    }
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "stencil".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/BlackOpsOne-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "barlow".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Barlow-Medium.ttf"
        ))),
    );
    fonts.font_data.insert(
        "sharetech".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/ShareTechMono-Regular.ttf"
        ))),
    );

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "barlow".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "sharetech".to_owned());
    fonts.families.insert(
        FontFamily::Name("stencil".into()),
        vec!["stencil".to_owned()],
    );
    ctx.set_fonts(fonts);
}

fn install_style(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.all_styles_mut(|style| {
        style.visuals.dark_mode = true;
        style.visuals.window_fill = CANVAS;
        style.visuals.panel_fill = CANVAS;
        style.visuals.extreme_bg_color = WELL;
        style.visuals.faint_bg_color = PANEL;
        style.visuals.override_text_color = Some(INK);
        style.visuals.window_stroke = Stroke::new(1.0, LINE);
        style.visuals.selection.bg_fill = Color32::from_rgb(0x5A, 0x46, 0x16);
        style.visuals.selection.stroke = Stroke::new(1.0, AMBER);
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TAN);
        style.visuals.widgets.inactive.bg_fill = WELL;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, LINE);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, INK);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x2C, 0x36, 0x1E);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, OLIVE);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, AMBER);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(0x3A, 0x32, 0x16);
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, AMBER);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, AMBER);
        style.visuals.widgets.open.bg_fill = PANEL;
        style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, AMBER);
        style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, INK);

        for widget in [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            widget.corner_radius = CornerRadius::ZERO;
        }

        style.spacing.item_spacing = vec2(8.0, 4.0);
        style.spacing.button_padding = vec2(10.0, 4.0);
        style.spacing.window_margin = Margin::same(PAD);
        style.text_styles.insert(
            egui::TextStyle::Heading,
            FontId::new(26.0, FontFamily::Name("stencil".into())),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(16.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            FontId::new(14.0, FontFamily::Monospace),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            FontId::new(13.0, FontFamily::Monospace),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            FontId::new(18.0, FontFamily::Monospace),
        );
    });
}

fn stencil(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("stencil".into()))
}

fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

fn ink_squares(ui: &mut egui::Ui, ink: &mut Ink) {
    let mut picked = None;
    for choice in Ink::ALL {
        let (rect, response) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::click());
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, choice.color());
        if *ink == choice {
            ui.painter().rect_stroke(
                rect,
                CornerRadius::ZERO,
                Stroke::new(2.0, Color32::BLACK),
                StrokeKind::Inside,
            );
        }
        if response.clicked() {
            picked = Some(choice);
        }
        ui.add_space(6.0);
    }
    if let Some(choice) = picked {
        *ink = choice;
    }
}

fn readout(ui: &mut egui::Ui, label: &str, x: &str, y: &str, color: Color32) {
    let x = if x.is_empty() { "—" } else { x };
    let y = if y.is_empty() { "—" } else { y };
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).font(mono(12.0)).color(color));
        ui.label(
            RichText::new(format!("{x}   {y}"))
                .font(mono(16.0))
                .color(color),
        );
    });
}

fn range_well(ui: &mut egui::Ui, reading: &BoardRead, ink: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 88.0), Sense::hover());
    let fault = if ink.r() > 220 && ink.g() < 90 {
        Color32::WHITE
    } else {
        Color32::from_rgb(0xFF, 0x5A, 0x4A)
    };
    let fill = Color32::from_rgba_unmultiplied(0, 0, 0, 150);
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, fill);
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, ink),
        StrokeKind::Inside,
    );
    corner_ticks(painter, rect, ink);

    let center = rect.center();
    let (label, color) = match reading {
        BoardRead::Ready(fix) => (format!("{}m", format_number(fix.meters)), ink),
        BoardRead::Need(_) => ("STANDBY".to_owned(), ink),
        BoardRead::Fault(_) => ("FAULT".to_owned(), fault),
    };
    let digits = label.trim_end_matches('m');
    let mut number_size = if label.ends_with('m')
        && digits
            .chars()
            .all(|ch| ch.is_ascii_digit() || ch == '.' || ch == '-')
    {
        match label.chars().count() {
            0..=6 => 72.0,
            7..=9 => 56.0,
            _ => 40.0,
        }
    } else {
        34.0
    };
    number_size *= 0.72;
    let font = stencil(number_size);
    let text_width = painter
        .layout_no_wrap(label.clone(), font.clone(), color)
        .size()
        .x;
    let half = text_width * 0.5 + 14.0;
    let hairline = Stroke::new(1.0, ink);
    painter.line_segment(
        [
            pos2(rect.left() + 18.0, center.y),
            pos2(center.x - half, center.y),
        ],
        hairline,
    );
    painter.line_segment(
        [
            pos2(center.x + half, center.y),
            pos2(rect.right() - 18.0, center.y),
        ],
        hairline,
    );
    painter.text(center, Align2::CENTER_CENTER, label, font, color);
}

fn corner_ticks(painter: &egui::Painter, rect: Rect, color: Color32) {
    let inset = 8.0;
    let arm = 16.0;
    let stroke = Stroke::new(2.0, color);
    let corners = [
        rect.left_top() + vec2(inset, inset),
        rect.right_top() + vec2(-inset, inset),
        rect.left_bottom() + vec2(inset, -inset),
        rect.right_bottom() + vec2(-inset, -inset),
    ];
    let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    for (origin, (sx, sy)) in corners.into_iter().zip(signs) {
        painter.line_segment([origin, origin + vec2(arm * sx, 0.0)], stroke);
        painter.line_segment([origin, origin + vec2(0.0, arm * sy)], stroke);
    }
}
