use eframe::egui;
use egui::{Align2, Color32, CornerRadius, Rect, RichText, Sense, Stroke, StrokeKind, pos2, vec2};

use crate::ink::{Ink, Palette};
use crate::range::{BoardRead, format_bearing, format_number};
use crate::terrain::{self, Map};
use crate::theme::{mono, stencil};
use crate::wave::{border_wave, paint_complement_line, paint_wave};

pub(crate) fn readout(ui: &mut egui::Ui, label: &str, x: &str, y: &str, color: Color32) {
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

pub(crate) fn map_row(ui: &mut egui::Ui, map: &mut Map, color: Color32) {
    ui.horizontal(|ui| {
        let button_width = 52.0;
        let gaps = ui.spacing().item_spacing.x * 2.0;
        let spare = (ui.available_width() - button_width * 3.0 - gaps).max(0.0);
        ui.add_space(spare * 0.5);
        for choice in Map::ALL {
            let swatch = if *map == choice {
                color
            } else {
                Color32::from_rgb(0x7D, 0x8A, 0x52)
            };
            let button =
                egui::Button::new(RichText::new(choice.short()).font(mono(12.0)).color(swatch));
            if ui
                .add_sized(vec2(button_width, 22.0), button)
                .on_hover_text(choice.name())
                .clicked()
            {
                *map = choice;
            }
        }
    });
}

pub(crate) fn elevation_row(
    ui: &mut egui::Ui,
    map: Map,
    you_x: &str,
    you_y: &str,
    enemy_x: &str,
    enemy_y: &str,
    color: Color32,
) {
    ui.horizontal(|ui| {
        let gun = terrain::elevation_of(map, you_x, you_y);
        let target = terrain::elevation_of(map, enemy_x, enemy_y);
        let gun_text = gun.map_or("—".to_owned(), |meters| {
            format!("{}m", format_number(meters))
        });
        let target_text = target.map_or("—".to_owned(), |meters| {
            format!("{}m", format_number(meters))
        });
        let delta_text = match (gun, target) {
            (Some(gun), Some(target)) => format!("{}m", format_delta(target - gun)),
            _ => "—".to_owned(),
        };
        height_pair(ui, "G", &gun_text, color);
        height_pair(ui, "T", &target_text, color);
        height_pair(ui, "DZ", &delta_text, color);
    });
}

fn height_pair(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    ui.label(RichText::new(label).font(mono(12.0)).color(color));
    ui.label(RichText::new(value).font(mono(16.0)).color(color));
}

fn format_delta(meters: f64) -> String {
    let text = format_number(meters.abs());
    if meters >= 0.0 {
        format!("+{text}")
    } else {
        format!("-{text}")
    }
}

pub(crate) fn range_well(ui: &mut egui::Ui, reading: &BoardRead, ink: Ink) {
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
    let (label, bearing, color) = match reading {
        BoardRead::Ready(fix) => (
            format!("{}m", format_number(fix.meters)),
            Some(format_bearing(fix.bearing)),
            color,
        ),
        BoardRead::Need(_) => ("STANDBY".to_owned(), None, color),
        BoardRead::Fault(_) => ("FAULT".to_owned(), None, fault),
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
    let max_width = (rect.width() - 36.0).max(24.0);
    let (font, text_width, text_height, range_ink) =
        fit_line(painter, &label, number_size, max_width, color);
    let bearing_line = bearing.map(|text| {
        let size = (number_size * 0.5).clamp(14.0, 22.0);
        let (bearing_font, bearing_width, bearing_height, bearing_ink) =
            fit_line(painter, &text, size, max_width, color);
        (
            text,
            bearing_font,
            bearing_width,
            bearing_height,
            bearing_ink,
        )
    });
    let bar_y = center.y;
    let bearing_height = bearing_line
        .as_ref()
        .map_or(0.0, |(_, _, _, height, _)| *height);
    let separation = if bearing_line.is_some() {
        text_height * 0.44 + bearing_height * 0.12
    } else {
        0.0
    };
    let ink_shift = bearing_line
        .as_ref()
        .map_or(range_ink, |(_, _, _, _, bearing_ink)| {
            (range_ink + bearing_ink) * 0.5
        });
    let range_y = center.y - separation * 0.5 - ink_shift;
    let half = text_width * 0.5 + 14.0;
    let left = [
        pos2(rect.left() + 18.0, bar_y),
        pos2(center.x - half, bar_y),
    ];
    let right = [
        pos2(center.x + half, bar_y),
        pos2(rect.right() - 18.0, bar_y),
    ];
    if let Some((phase, palette)) = wave {
        if left[0].x < left[1].x {
            paint_complement_line(painter, left[0], left[1], phase, palette);
        }
        if right[0].x < right[1].x {
            paint_complement_line(painter, right[1], right[0], phase, palette);
        }
    } else {
        let hairline = Stroke::new(1.0, color);
        if left[0].x < left[1].x {
            painter.line_segment(left, hairline);
        }
        if right[0].x < right[1].x {
            painter.line_segment(right, hairline);
        }
    }
    painter.text(
        pos2(center.x, range_y),
        Align2::CENTER_CENTER,
        label,
        font,
        color,
    );
    if let Some((bearing, bearing_font, _, _, _)) = bearing_line {
        painter.text(
            pos2(center.x, center.y + separation * 0.5 - ink_shift),
            Align2::CENTER_CENTER,
            bearing,
            bearing_font,
            color,
        );
    }
}

fn fit_line(
    painter: &egui::Painter,
    text: &str,
    mut size: f32,
    max_width: f32,
    color: Color32,
) -> (egui::FontId, f32, f32, f32) {
    let mut font = stencil(size);
    let mut galley = painter.layout_no_wrap(text.to_owned(), font.clone(), color);
    for _ in 0..2 {
        let width = galley.size().x;
        if width <= max_width || width <= 1.0 {
            break;
        }
        size = (size * max_width / width).max(10.0);
        font = stencil(size);
        galley = painter.layout_no_wrap(text.to_owned(), font.clone(), color);
    }
    let ink_below_center = galley.mesh_bounds.center().y - galley.rect.center().y;
    (font, galley.size().x, galley.size().y, ink_below_center)
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
