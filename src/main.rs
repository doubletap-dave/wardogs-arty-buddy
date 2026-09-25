#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod range;

use std::sync::Arc;

use eframe::egui;
use egui::{
    Align, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame,
    Layout, Margin, Rect, RichText, Sense, Stroke, StrokeKind, TextEdit, pos2, vec2,
};

use range::{BoardRead, format_number, read_board};

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
            .with_title("Wardogs Arty Buddy"),
        ..Default::default()
    };

    eframe::run_native(
        "Wardogs Arty Buddy",
        options,
        Box::new(|cc| Ok(Box::new(ArtyBuddy::new(cc)))),
    )
}

struct ArtyBuddy {
    you_x: String,
    you_y: String,
    enemy_x: String,
    enemy_y: String,
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
        CANVAS.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let reading = read_board(&self.you_x, &self.you_y, &self.enemy_x, &self.enemy_y);
        egui::CentralPanel::default().show(ui, |ui| {
            header(ui);
            ui.add_space(12.0);
            ui.columns(2, |cols| {
                station(
                    cols,
                    0,
                    "OWN STATION",
                    OLIVE,
                    &mut self.you_x,
                    &mut self.you_y,
                );
                station(
                    cols,
                    1,
                    "TARGET",
                    BRICK,
                    &mut self.enemy_x,
                    &mut self.enemy_y,
                );
            });
            ui.add_space(12.0);
            range_well(ui, &reading);
            ui.add_space(10.0);
            footer(ui, &reading, &mut || self.clear());
        });
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

fn header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("WARDOGS").font(stencil(28.0)).color(TAN));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
}

fn station(
    cols: &mut [egui::Ui],
    index: usize,
    title: &str,
    accent: Color32,
    x: &mut String,
    y: &mut String,
) {
    let ui = &mut cols[index];
    Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, LINE))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (mark, _) = ui.allocate_exact_size(vec2(4.0, 14.0), Sense::hover());
                ui.painter().rect_filled(mark, CornerRadius::ZERO, accent);
                ui.label(RichText::new(title).font(mono(13.0)).color(TAN));
            });
            ui.add_space(10.0);
            coord_field(ui, "X", x);
            ui.add_space(8.0);
            coord_field(ui, "Y", y);
        });
}

fn coord_field(ui: &mut egui::Ui, axis: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [16.0, 28.0],
            egui::Label::new(RichText::new(axis).font(mono(16.0)).color(AMBER)),
        );
        ui.add(
            TextEdit::singleline(value)
                .font(mono(20.0))
                .desired_width(ui.available_width())
                .hint_text("0")
                .margin(Margin::symmetric(8, 6)),
        );
    });
}

fn range_well(ui: &mut egui::Ui, reading: &BoardRead) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 188.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, WELL);
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, LINE),
        StrokeKind::Inside,
    );
    corner_ticks(painter, rect);

    let center = rect.center();
    let gap = 150.0;
    let hairline = Stroke::new(1.0, LINE);
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
        BoardRead::Ready(fix) => (format_number(fix.meters), "METERS".to_owned(), AMBER),
        BoardRead::Need(field) => ("STANDBY".to_owned(), field.label().to_owned(), TAN),
        BoardRead::Fault(field) => (
            "FAULT".to_owned(),
            format!("{} is not a number", field.label()),
            BRICK,
        ),
    };
    let numeric = number
        .chars()
        .all(|ch| ch.is_ascii_digit() || ch == '.' || ch == '-');
    let number_size = if numeric {
        match number.chars().count() {
            0..=5 => 72.0,
            6..=8 => 56.0,
            _ => 40.0,
        }
    } else {
        34.0
    };
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
            BRICK
        } else {
            TAN
        },
    );
}

fn corner_ticks(painter: &egui::Painter, rect: Rect) {
    let inset = 8.0;
    let arm = 16.0;
    let stroke = Stroke::new(2.0, AMBER);
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
