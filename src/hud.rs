use eframe::egui;
use egui::{Align2, Color32, CornerRadius, Rect, RichText, Sense, Stroke, StrokeKind, pos2, vec2};

use crate::ink::{Ink, Palette};
use crate::range::{BoardRead, format_number};
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
