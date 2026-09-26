use std::sync::Arc;

use eframe::egui;
use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke, vec2,
};

pub(crate) const CANVAS: Color32 = Color32::from_rgb(0x1C, 0x24, 0x16);
pub(crate) const PANEL: Color32 = Color32::from_rgb(0x24, 0x2C, 0x1A);
pub(crate) const WELL: Color32 = Color32::from_rgb(0x10, 0x14, 0x0B);
pub(crate) const INK: Color32 = Color32::from_rgb(0xD8, 0xD0, 0xB8);
pub(crate) const TAN: Color32 = Color32::from_rgb(0xC4, 0xB4, 0x8A);
pub(crate) const AMBER: Color32 = Color32::from_rgb(0xE2, 0xA2, 0x2A);
pub(crate) const OLIVE: Color32 = Color32::from_rgb(0x7D, 0x8A, 0x52);
pub(crate) const LINE: Color32 = Color32::from_rgb(0x4E, 0x5A, 0x38);
pub(crate) const TITLE: &str = concat!("Wardogs Arty Buddy ", env!("CARGO_PKG_VERSION"));
pub(crate) const PAD: i8 = 10;
pub(crate) const GUTTER: i8 = 18;
pub(crate) const WINDOW_SIZE: egui::Vec2 =
    egui::vec2(400.0 + GUTTER as f32 * 2.0, 196.0 + GUTTER as f32 * 2.0);

pub(crate) fn install_fonts(ctx: &egui::Context) {
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

pub(crate) fn install_style(ctx: &egui::Context) {
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

pub(crate) fn stencil(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("stencil".into()))
}

pub(crate) fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}
