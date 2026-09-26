use eframe::egui;
use egui::{Color32, CornerRadius, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ink {
    Green,
    Red,
    Blue,
    White,
    Amber,
    // RGB as a color scheme was KingPredict's idea.
    Rainbow,
    BlueWave,
    PurpleWave,
    RedWave,
}

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub(crate) center: f32,
    pub(crate) span: f32,
    pub(crate) text: Color32,
    pub(crate) bands: [Color32; 4],
}

impl Ink {
    pub(crate) const ALL: [Self; 9] = [
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

    pub(crate) fn palette(self) -> Option<Palette> {
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

    pub(crate) fn color(self) -> Color32 {
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

pub(crate) fn ink_squares(ui: &mut egui::Ui, ink: &mut Ink) {
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
