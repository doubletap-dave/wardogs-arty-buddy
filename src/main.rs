#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod range;

use std::sync::Arc;

use eframe::egui;
use egui::{
    Align, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame,
    Layout, Margin, Rect, RichText, Sense, Stroke, StrokeKind, TextEdit, ViewportCommand,
    WindowLevel, pos2, vec2,
};

use range::{BoardRead, absorb_wardogs, format_number, read_board};

const CANVAS: Color32 = Color32::from_rgb(0x1C, 0x24, 0x16);
const PANEL: Color32 = Color32::from_rgb(0x24, 0x2C, 0x1A);
const WELL: Color32 = Color32::from_rgb(0x10, 0x14, 0x0B);
const INK: Color32 = Color32::from_rgb(0xD8, 0xD0, 0xB8);
const TAN: Color32 = Color32::from_rgb(0xC4, 0xB4, 0x8A);
const AMBER: Color32 = Color32::from_rgb(0xE2, 0xA2, 0x2A);
const BRICK: Color32 = Color32::from_rgb(0x8C, 0x3A, 0x2F);
const OLIVE: Color32 = Color32::from_rgb(0x7D, 0x8A, 0x52);
const LINE: Color32 = Color32::from_rgb(0x4E, 0x5A, 0x38);

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([760.0, 560.0])
            .with_min_inner_size([640.0, 480.0])
            .with_transparent(true)
            .with_title("Wardogs Arty Buddy"),
        ..Default::default()
    };

    eframe::run_native(
        "Wardogs Arty Buddy",
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

    fn label(self) -> &'static str {
        match self {
            Self::Green => "GREEN",
            Self::Red => "RED",
            Self::Blue => "BLUE",
            Self::White => "WHITE",
            Self::Amber => "AMBER",
        }
    }

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
    game_mode: bool,
    pass_clicks: bool,
    ink: Ink,
    window_is_game: bool,
    passing: bool,
}

impl ArtyBuddy {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        install_style(&cc.egui_ctx);
        Self {
            you_x: String::new(),
            you_y: String::new(),
            enemy_x: String::new(),
            enemy_y: String::new(),
            game_mode: false,
            pass_clicks: false,
            ink: Ink::Green,
            window_is_game: false,
            passing: false,
        }
    }

    fn sync_window(&mut self, ctx: &egui::Context) {
        let pass = self.game_mode && self.pass_clicks;
        if pass != self.passing {
            self.passing = pass;
            ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(pass));
        }
        if self.game_mode == self.window_is_game {
            return;
        }
        self.window_is_game = self.game_mode;
        if self.game_mode {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(540.0, 400.0)));
            ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(500.0, 360.0)));
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
        } else {
            self.pass_clicks = false;
            ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(false));
            self.passing = false;
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
            ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(640.0, 480.0)));
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(760.0, 560.0)));
        }
    }

    fn clear(&mut self) {
        self.you_x.clear();
        self.you_y.clear();
        self.enemy_x.clear();
        self.enemy_y.clear();
    }
}

impl eframe::App for ArtyBuddy {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        if self.game_mode {
            Color32::TRANSPARENT.to_normalized_gamma_f32()
        } else {
            CANVAS.to_normalized_gamma_f32()
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.pass_clicks && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.pass_clicks = false;
        }
        self.sync_window(ui.ctx());
        absorb_wardogs(&mut self.you_x, &mut self.you_y);
        absorb_wardogs(&mut self.enemy_x, &mut self.enemy_y);
        let reading = read_board(&self.you_x, &self.you_y, &self.enemy_x, &self.enemy_y);
        let plate = if self.game_mode {
            Color32::from_rgba_unmultiplied(8, 12, 8, 176)
        } else {
            CANVAS
        };
        let margin = if self.game_mode { 10 } else { 16 };
        egui::CentralPanel::default()
            .frame(Frame::new().fill(plate).inner_margin(Margin::same(margin)))
            .show(ui, |ui| {
                if self.game_mode {
                    self.game_hud(ui, &reading);
                } else {
                    if header(ui) {
                        self.game_mode = true;
                    }
                    ui.add_space(12.0);
                    ui.columns(2, |cols| {
                        station(
                            cols,
                            0,
                            "OWN STATION",
                            OLIVE,
                            PANEL,
                            LINE,
                            &mut self.you_x,
                            &mut self.you_y,
                        );
                        station(
                            cols,
                            1,
                            "TARGET",
                            BRICK,
                            PANEL,
                            LINE,
                            &mut self.enemy_x,
                            &mut self.enemy_y,
                        );
                    });
                    ui.add_space(12.0);
                    range_well(ui, &reading, None, false);
                    ui.add_space(10.0);
                    footer(ui, &reading, &mut || self.clear());
                }
            });
    }
}

impl ArtyBuddy {
    fn game_hud(&mut self, ui: &mut egui::Ui, reading: &BoardRead) {
        let ink = self.ink.color();
        let plate = Color32::from_rgba_unmultiplied(0, 0, 0, 150);
        ui.horizontal(|ui| {
            ui.label(RichText::new("GAME").font(stencil(20.0)).color(ink));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(egui::Button::new(
                        RichText::new("BOARD").font(mono(12.0)).color(ink),
                    ))
                    .clicked()
                {
                    self.game_mode = false;
                }
                let pass_label = if self.pass_clicks {
                    "CLICKS PASS"
                } else {
                    "PASS CLICKS"
                };
                if ui
                    .add(egui::Button::new(
                        RichText::new(pass_label).font(mono(12.0)).color(ink),
                    ))
                    .clicked()
                {
                    self.pass_clicks = !self.pass_clicks;
                }
            });
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| ink_swatches(ui, &mut self.ink));
        ui.add_space(8.0);
        ui.columns(2, |cols| {
            station(
                cols,
                0,
                "OWN",
                ink,
                plate,
                ink,
                &mut self.you_x,
                &mut self.you_y,
            );
            station(
                cols,
                1,
                "TARGET",
                ink,
                plate,
                ink,
                &mut self.enemy_x,
                &mut self.enemy_y,
            );
        });
        ui.add_space(8.0);
        range_well(ui, reading, Some(ink), true);
        if self.pass_clicks {
            ui.add_space(6.0);
            ui.label(
                RichText::new("Clicks pass through. Alt+Tab back here, then Esc.")
                    .font(mono(12.0))
                    .color(ink),
            );
        }
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

        style.spacing.item_spacing = vec2(12.0, 8.0);
        style.spacing.button_padding = vec2(14.0, 8.0);
        style.spacing.window_margin = Margin::same(16);
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

fn header(ui: &mut egui::Ui) -> bool {
    let mut enter_game = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new("WARDOGS").font(stencil(28.0)).color(TAN));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add(egui::Button::new(
                    RichText::new("GAME").font(mono(13.0)).color(TAN),
                ))
                .clicked()
            {
                enter_game = true;
            }
            ui.label(
                RichText::new("FIRE DIRECTION")
                    .font(mono(13.0))
                    .color(OLIVE),
            );
        });
    });
    ui.label(RichText::new("ARTY BUDDY").font(stencil(18.0)).color(INK));
    ui.add_space(6.0);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 2.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, AMBER);
    enter_game
}

fn ink_swatches(ui: &mut egui::Ui, ink: &mut Ink) {
    for choice in Ink::ALL {
        let color = choice.color();
        let selected = *ink == choice;
        if ui
            .add(
                egui::Button::new(RichText::new(choice.label()).font(mono(12.0)).color(color))
                    .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 150))
                    .stroke(Stroke::new(if selected { 2.0 } else { 1.0 }, color)),
            )
            .clicked()
        {
            *ink = choice;
        }
    }
}

fn station(
    cols: &mut [egui::Ui],
    index: usize,
    title: &str,
    accent: Color32,
    plate: Color32,
    stroke: Color32,
    x: &mut String,
    y: &mut String,
) {
    let ui = &mut cols[index];
    Frame::new()
        .fill(plate)
        .stroke(Stroke::new(1.0, stroke))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (mark, _) = ui.allocate_exact_size(vec2(4.0, 14.0), Sense::hover());
                ui.painter().rect_filled(mark, CornerRadius::ZERO, accent);
                ui.label(RichText::new(title).font(mono(13.0)).color(TAN));
            });
            ui.add_space(10.0);
            coord_field(ui, "X", x, "x00.00, y00.00");
            ui.add_space(8.0);
            coord_field(ui, "Y", y, "00.00");
        });
}

fn coord_field(ui: &mut egui::Ui, axis: &str, value: &mut String, hint: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [16.0, 28.0],
            egui::Label::new(RichText::new(axis).font(mono(16.0)).color(AMBER)),
        );
        ui.add(
            TextEdit::singleline(value)
                .font(mono(20.0))
                .desired_width(ui.available_width())
                .hint_text(hint)
                .margin(Margin::symmetric(8, 6)),
        );
    });
}

fn range_well(ui: &mut egui::Ui, reading: &BoardRead, ink: Option<Color32>, compact: bool) {
    let height = if compact { 112.0 } else { 188.0 };
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let accent = ink.unwrap_or(AMBER);
    let line = ink.unwrap_or(LINE);
    let caption_color = ink.unwrap_or(TAN);
    let fault = match ink {
        Some(color) if color.r() > 220 && color.g() < 90 => Color32::WHITE,
        _ => BRICK,
    };
    let fill = if ink.is_some() {
        Color32::from_rgba_unmultiplied(0, 0, 0, 150)
    } else {
        WELL
    };
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, fill);
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, line),
        StrokeKind::Inside,
    );
    corner_ticks(painter, rect, accent);

    let center = rect.center();
    let gap = if compact { 92.0 } else { 150.0 };
    let hairline = Stroke::new(1.0, line);
    painter.line_segment(
        [
            pos2(rect.left() + 18.0, center.y),
            pos2(center.x - gap, center.y),
        ],
        hairline,
    );
    painter.line_segment(
        [
            pos2(center.x + gap, center.y),
            pos2(rect.right() - 18.0, center.y),
        ],
        hairline,
    );

    let (number, caption, color) = match reading {
        BoardRead::Ready(fix) => (format_number(fix.meters), "METERS".to_owned(), accent),
        BoardRead::Need(field) => (
            "STANDBY".to_owned(),
            field.label().to_owned(),
            caption_color,
        ),
        BoardRead::Fault(field) => (
            "FAULT".to_owned(),
            format!("{} is not a number", field.label()),
            fault,
        ),
    };
    let numeric = number
        .chars()
        .all(|ch| ch.is_ascii_digit() || ch == '.' || ch == '-');
    let mut number_size = if numeric {
        match number.chars().count() {
            0..=5 => 72.0,
            6..=8 => 56.0,
            _ => 40.0,
        }
    } else {
        34.0
    };
    if compact {
        number_size *= 0.72;
    }
    painter.text(
        center + vec2(0.0, -14.0),
        Align2::CENTER_CENTER,
        number,
        stencil(number_size),
        color,
    );
    painter.text(
        pos2(center.x, rect.bottom() - 28.0),
        Align2::CENTER_CENTER,
        caption,
        mono(13.0),
        if matches!(reading, BoardRead::Fault(_)) {
            fault
        } else {
            caption_color
        },
    );
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

fn footer(ui: &mut egui::Ui, reading: &BoardRead, clear: &mut dyn FnMut()) {
    ui.horizontal(|ui| {
        match reading {
            BoardRead::Ready(fix) => {
                delta(ui, "ΔX", format_number(fix.dx));
                delta(ui, "ΔY", format_number(fix.dy));
                delta(ui, "GRID", format_number(fix.grid));
            }
            BoardRead::Need(_) => {
                ui.label(
                    RichText::new("100 × straight-line grid distance")
                        .font(mono(13.0))
                        .color(TAN),
                );
            }
            BoardRead::Fault(_) => {
                ui.label(
                    RichText::new("Use a plain number, like 12 or -3.5")
                        .font(mono(13.0))
                        .color(BRICK),
                );
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add(egui::Button::new(
                    RichText::new("CLEAR BOARD").font(mono(13.0)).color(TAN),
                ))
                .clicked()
            {
                clear();
            }
        });
    });
}

fn delta(ui: &mut egui::Ui, label: &str, value: String) {
    ui.label(RichText::new(label).font(mono(12.0)).color(OLIVE));
    ui.label(RichText::new(value).font(mono(16.0)).color(INK));
    ui.add_space(12.0);
}
