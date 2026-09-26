#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Idea progenitor: KingPredict.

mod range;

use std::sync::Arc;
use std::time::Duration;

use eframe::egui;
use egui::{
    Align, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame,
    Layout, Margin, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2, ViewportCommand, pos2,
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
const GUTTER: i8 = 18;
const WINDOW_SIZE: Vec2 = vec2(400.0 + GUTTER as f32 * 2.0, 196.0 + GUTTER as f32 * 2.0);

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
    Rainbow,
    BlueWave,
    PurpleWave,
    RedWave,
}

#[derive(Clone, Copy)]
struct Palette {
    center: f32,
    span: f32,
    text: Color32,
    bands: [Color32; 4],
}

impl Ink {
    const ALL: [Self; 9] = [
        Self::Green,
        Self::Red,
        Self::Blue,
        Self::White,
        Self::Amber,
        Self::Rainbow,
        Self::BlueWave,
        Self::PurpleWave,
        Self::RedWave,
    ];

    fn palette(self) -> Option<Palette> {
        match self {
            Self::Rainbow => Some(Palette {
                center: 0.0,
                span: 1.0,
                text: Color32::from_rgb(0xF4, 0xF4, 0xF4),
                bands: [
                    Color32::from_rgb(0xFF, 0x2A, 0x2A),
                    Color32::from_rgb(0xFF, 0xE1, 0x2B),
                    Color32::from_rgb(0x3D, 0xFF, 0x6A),
                    Color32::from_rgb(0x3D, 0xA5, 0xFF),
                ],
            }),
            Self::BlueWave => Some(Palette {
                center: 0.50,
                span: 0.20,
                text: Color32::from_rgb(0x3D, 0xA5, 0xFF),
                bands: [
                    Color32::from_rgb(0x0A, 0x3A, 0xC4),
                    Color32::from_rgb(0x1E, 0x6B, 0xFF),
                    Color32::from_rgb(0x3D, 0xA5, 0xFF),
                    Color32::from_rgb(0x7A, 0xFF, 0xF0),
                ],
            }),
            Self::PurpleWave => Some(Palette {
                center: 0.76,
                span: 0.16,
                text: Color32::from_rgb(0xD0, 0x7B, 0xFF),
                bands: [
                    Color32::from_rgb(0x4A, 0x14, 0x8C),
                    Color32::from_rgb(0x7B, 0x2C, 0xBF),
                    Color32::from_rgb(0xC7, 0x7D, 0xFF),
                    Color32::from_rgb(0xFF, 0x6A, 0xD5),
                ],
            }),
            Self::RedWave => Some(Palette {
                center: 0.98,
                span: 0.16,
                text: Color32::from_rgb(0xFF, 0x45, 0x3A),
                bands: [
                    Color32::from_rgb(0xFF, 0x1E, 0x4A),
                    Color32::from_rgb(0xFF, 0x45, 0x3A),
                    Color32::from_rgb(0xFF, 0x7A, 0x18),
                    Color32::from_rgb(0xFF, 0xD0, 0x00),
                ],
            }),
            _ => None,
        }
    }

    fn color(self) -> Color32 {
        match self {
            Self::Green => Color32::from_rgb(0x6B, 0xF0, 0x4A),
            Self::Red => Color32::from_rgb(0xFF, 0x45, 0x3A),
            Self::Blue => Color32::from_rgb(0x3D, 0xA5, 0xFF),
            Self::White => Color32::from_rgb(0xF4, 0xF4, 0xF4),
            Self::Amber => Color32::from_rgb(0xE2, 0xA2, 0x2A),
            Self::Rainbow | Self::BlueWave | Self::PurpleWave | Self::RedWave => {
                self.palette().unwrap().text
            }
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
    last_clip_poll: f64,
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

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.sync_window(ui.ctx());
        absorb_wardogs(&mut self.you_x, &mut self.you_y);
        absorb_wardogs(&mut self.enemy_x, &mut self.enemy_y);
        let reading = read_board(&self.you_x, &self.you_y, &self.enemy_x, &self.enemy_y);
        let plate = Color32::from_rgba_unmultiplied(8, 12, 8, 176);
        let panel = egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(plate)
                    .inner_margin(Margin::same(PAD + GUTTER)),
            )
            .show(ui, |ui| {
                self.hud(ui, &reading);
            });
        if let Some(palette) = self.ink.palette() {
            let t = ui.input(|input| input.time) as f32;
            paint_wave(
                ui.painter(),
                panel.response.rect.shrink(9.0),
                (t / 6.5).fract(),
                palette,
                2.2,
                8.0,
            );
        }
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
        range_well(ui, reading, self.ink);
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
        paint_swatch(ui.painter(), rect, choice);
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

fn paint_swatch(painter: &egui::Painter, rect: Rect, ink: Ink) {
    let Some(palette) = ink.palette() else {
        painter.rect_filled(rect, CornerRadius::ZERO, ink.color());
        return;
    };
    let bands = palette.bands.len() as f32;
    for (index, color) in palette.bands.iter().enumerate() {
        let x0 = rect.left() + rect.width() * (index as f32 / bands);
        let x1 = rect.left() + rect.width() * ((index + 1) as f32 / bands);
        painter.rect_filled(
            Rect::from_min_max(pos2(x0, rect.top()), pos2(x1, rect.bottom())),
            CornerRadius::ZERO,
            *color,
        );
    }
}

fn range_well(ui: &mut egui::Ui, reading: &BoardRead, ink: Ink) {
    let color = ink.color();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 88.0), Sense::hover());
    let fault = if color.r() > 220 && color.g() < 90 {
        Color32::WHITE
    } else {
        Color32::from_rgb(0xFF, 0x5A, 0x4A)
    };
    let fill = Color32::from_rgba_unmultiplied(0, 0, 0, 150);
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, fill);
    let wave = ink.palette().map(|palette| {
        let t = ui.input(|input| input.time) as f32;
        ((t / 6.5 + 0.37).fract(), palette)
    });
    if let Some((phase, palette)) = wave {
        paint_wave(painter, rect.shrink(1.0), phase, palette, 1.8, 5.0);
        corner_ticks_wave(painter, rect, phase, palette);
    } else {
        painter.rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, color),
            StrokeKind::Inside,
        );
        corner_ticks(painter, rect, color);
    }

    let center = rect.center();
    let (label, color) = match reading {
        BoardRead::Ready(fix) => (format!("{}m", format_number(fix.meters)), color),
        BoardRead::Need(_) => ("STANDBY".to_owned(), color),
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
    let left = [
        pos2(rect.left() + 18.0, center.y),
        pos2(center.x - half, center.y),
    ];
    let right = [
        pos2(center.x + half, center.y),
        pos2(rect.right() - 18.0, center.y),
    ];
    if let Some((phase, palette)) = wave {
        paint_complement_line(painter, left[0], left[1], phase, palette);
        paint_complement_line(painter, right[1], right[0], phase, palette);
    } else {
        let hairline = Stroke::new(1.0, color);
        painter.line_segment(left, hairline);
        painter.line_segment(right, hairline);
    }
    painter.text(center, Align2::CENTER_CENTER, label, font, color);
}

fn corner_ticks(painter: &egui::Painter, rect: Rect, color: Color32) {
    paint_corners(painter, rect, |_| color);
}

fn corner_ticks_wave(painter: &egui::Painter, rect: Rect, phase: f32, palette: Palette) {
    paint_corners(painter, rect, |along| border_wave(along, phase, palette));
}

fn paint_corners(painter: &egui::Painter, rect: Rect, color_at: impl Fn(f32) -> Color32) {
    let inset = 8.0;
    let arm = 16.0;
    let w = rect.width().max(1.0);
    let h = rect.height().max(1.0);
    let total = (w + h) * 2.0;
    let spots = [
        (rect.left_top() + vec2(inset, inset), 1.0, 1.0, 0.0),
        (rect.right_top() + vec2(-inset, inset), -1.0, 1.0, w / total),
        (
            rect.left_bottom() + vec2(inset, -inset),
            1.0,
            -1.0,
            (w * 2.0 + h) / total,
        ),
        (
            rect.right_bottom() + vec2(-inset, -inset),
            -1.0,
            -1.0,
            (w + h) / total,
        ),
    ];
    for (origin, sx, sy, along) in spots {
        let stroke = Stroke::new(2.2, color_at(along));
        painter.line_segment([origin, origin + vec2(arm * sx, 0.0)], stroke);
        painter.line_segment([origin, origin + vec2(0.0, arm * sy)], stroke);
    }
}

fn wave_amount(along: f32, phase: f32) -> f32 {
    let theta = (along + phase) * std::f32::consts::TAU;
    0.5 - 0.5 * theta.cos()
}

fn border_hue(palette: Palette, along: f32, phase: f32) -> f32 {
    if palette.span >= 0.9 {
        along + phase
    } else {
        palette.center + wave_amount(along, phase) * palette.span
    }
}

fn blend_bands(bands: [Color32; 4], t: f32) -> Color32 {
    let scaled = t.clamp(0.0, 1.0) * (bands.len() - 1) as f32;
    let index = (scaled.floor() as usize).min(bands.len() - 2);
    let mut span = scaled - index as f32;
    span = span * span * (3.0 - 2.0 * span);
    mix(bands[index], bands[index + 1], span)
}

fn border_wave(along: f32, phase: f32, palette: Palette) -> Color32 {
    let color = if palette.span >= 0.9 {
        hsv(border_hue(palette, along, phase), 0.95, 1.0, 255)
    } else {
        blend_bands(palette.bands, wave_amount(along, phase))
    };
    with_alpha(color, 235)
}

fn border_glow(along: f32, phase: f32, palette: Palette) -> Color32 {
    with_alpha(border_wave(along, phase, palette), 72)
}

fn paint_wave(
    painter: &egui::Painter,
    rect: Rect,
    phase: f32,
    palette: Palette,
    core: f32,
    glow: f32,
) {
    paint_perimeter(painter, rect, glow, |along| {
        border_glow(along, phase, palette)
    });
    paint_perimeter(painter, rect, core, |along| {
        border_wave(along, phase, palette)
    });
}

fn complement_color(palette: Palette, amount: f32, alpha: u8) -> Color32 {
    let amount = amount.clamp(0.0, 1.0);
    if palette.span >= 0.9 {
        return hsv(amount + 0.5, 0.85, 1.0, alpha);
    }
    let hue = palette.center + 0.5 + amount * palette.span;
    let value = 0.62 + 0.38 * amount;
    hsv(hue, 0.88, value, alpha)
}

fn paint_complement_line(
    painter: &egui::Painter,
    from: Pos2,
    to: Pos2,
    phase: f32,
    palette: Palette,
) {
    let steps = 12;
    let delta = to - from;
    for step in 0..steps {
        let t0 = step as f32 / steps as f32;
        let t1 = (step + 1) as f32 / steps as f32;
        let mid = (t0 + t1) * 0.5;
        let amount = wave_amount(mid * 0.5, phase);
        let alpha = (220.0 - 110.0 * mid) as u8;
        painter.line_segment(
            [from + delta * t0, from + delta * t1],
            Stroke::new(1.6, complement_color(palette, amount, alpha)),
        );
    }
}

fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn hsv(hue: f32, sat: f32, val: f32, alpha: u8) -> Color32 {
    let h = hue.rem_euclid(1.0) * 6.0;
    let i = h.floor() as u8;
    let f = h - i as f32;
    let p = val * (1.0 - sat);
    let q = val * (1.0 - sat * f);
    let t = val * (1.0 - sat * (1.0 - f));
    let (r, g, b) = match i {
        0 => (val, t, p),
        1 => (q, val, p),
        2 => (p, val, t),
        3 => (p, q, val),
        4 => (t, p, val),
        _ => (val, p, q),
    };
    Color32::from_rgba_unmultiplied(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        alpha,
    )
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t).round() as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t).round() as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t).round() as u8,
    )
}

fn point_along_rect(rect: Rect, t: f32) -> Pos2 {
    let t = t.rem_euclid(1.0);
    let w = rect.width().max(0.0);
    let h = rect.height().max(0.0);
    let total = (w + h) * 2.0;
    if total <= f32::EPSILON {
        return rect.left_top();
    }
    let d = t * total;
    if d <= w {
        pos2(rect.left() + d, rect.top())
    } else if d <= w + h {
        pos2(rect.right(), rect.top() + (d - w))
    } else if d <= w * 2.0 + h {
        pos2(rect.right() - (d - w - h), rect.bottom())
    } else {
        pos2(rect.left(), rect.bottom() - (d - w * 2.0 - h))
    }
}

fn paint_perimeter(
    painter: &egui::Painter,
    rect: Rect,
    width: f32,
    color_at: impl Fn(f32) -> Color32,
) {
    if rect.width() < 4.0 || rect.height() < 4.0 {
        return;
    }
    let total = (rect.width() + rect.height()) * 2.0;
    let step = 5.0;
    let mut cursor = 0.0;
    let mut prev = point_along_rect(rect, 0.0);
    while cursor < total - 0.25 {
        let next_d = (cursor + step).min(total);
        let next = point_along_rect(rect, next_d / total);
        let color = color_at((cursor + next_d) * 0.5 / total);
        if color.a() > 0 {
            painter.line_segment([prev, next], Stroke::new(width, color));
        }
        prev = next;
        cursor = next_d;
    }
}

#[cfg(test)]
mod chroma_tests {
    use super::*;

    fn channel_jump(a: Color32, b: Color32) -> i16 {
        (a.r() as i16 - b.r() as i16).abs()
            + (a.g() as i16 - b.g() as i16).abs()
            + (a.b() as i16 - b.b() as i16).abs()
    }

    #[test]
    fn hue_wraps_to_red() {
        let a = hsv(0.0, 1.0, 1.0, 255);
        let b = hsv(1.0, 1.0, 1.0, 255);
        assert_eq!(a, b);
        assert_eq!(a, Color32::from_rgb(255, 0, 0));
    }

    #[test]
    fn perimeter_meets_itself() {
        let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(40.0, 10.0));
        let start = point_along_rect(rect, 0.0);
        let end = point_along_rect(rect, 1.0);
        assert!((start.x - end.x).abs() < 0.01);
        assert!((start.y - end.y).abs() < 0.01);
        let top_right = point_along_rect(rect, 40.0 / 100.0);
        assert!((top_right.x - 40.0).abs() < 0.01);
        assert!(top_right.y.abs() < 0.01);
    }

    #[test]
    fn themed_wave_meets_itself_without_a_hard_step() {
        let blue = Ink::BlueWave.palette().unwrap();
        assert!((wave_amount(0.0, 0.2) - wave_amount(1.0, 0.2)).abs() < 0.001);
        let mut prev = border_wave(0.0, 0.2, blue);
        for step in 1..=48 {
            let color = border_wave(step as f32 / 48.0, 0.2, blue);
            let jump = channel_jump(prev, color);
            assert!(jump < 100, "step {step} jumped {jump}");
            prev = color;
        }
    }

    #[test]
    fn side_lines_use_the_opposite_family() {
        let blue = complement_color(Ink::BlueWave.palette().unwrap(), 0.5, 255);
        let purple = complement_color(Ink::PurpleWave.palette().unwrap(), 0.5, 255);
        let red = complement_color(Ink::RedWave.palette().unwrap(), 0.5, 255);
        assert!(blue.r() > blue.b(), "blue's opposite should run warm");
        assert!(
            purple.g() > purple.b(),
            "purple's opposite should run green"
        );
        assert!(red.b() > red.r(), "red's opposite should run cool");
    }

    #[test]
    fn themed_waves_stay_inside_their_family() {
        let blue = Ink::BlueWave.palette().unwrap();
        let purple = Ink::PurpleWave.palette().unwrap();
        let red = Ink::RedWave.palette().unwrap();
        for along in [0.0, 0.25, 0.5, 0.8] {
            let hue = border_hue(blue, along, 0.3).rem_euclid(1.0);
            assert!((0.50..0.72).contains(&hue), "{hue}");
            let hue = border_hue(purple, along, 0.3).rem_euclid(1.0);
            assert!((0.76..0.94).contains(&hue), "{hue}");
            let hue = border_hue(red, along, 0.3).rem_euclid(1.0);
            assert!(hue >= 0.96 || hue <= 0.16, "{hue}");
        }
    }

    #[test]
    fn rainbow_wave_covers_the_frame_and_scrolls() {
        let rainbow = Ink::Rainbow.palette().unwrap();
        let left = border_hue(rainbow, 0.0, 0.1);
        let opposite = border_hue(rainbow, 0.5, 0.1);
        assert!((opposite - left - 0.5).abs() < 0.01);
        let shifted = border_hue(rainbow, 0.25, 0.4);
        let still = border_hue(rainbow, 0.25, 0.0);
        assert!((shifted - still - 0.4).abs() < 0.01);
    }

    #[test]
    fn solid_inks_have_no_wave() {
        assert!(Ink::Green.palette().is_none());
        assert!(Ink::Amber.palette().is_none());
        assert!(Ink::Rainbow.palette().is_some());
    }
}
