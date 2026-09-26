use eframe::egui;
use egui::{Color32, Painter, Pos2, Rect, Stroke, pos2};

use crate::ink::Palette;

pub(crate) fn paint_wave(
    painter: &Painter,
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

pub(crate) fn border_wave(along: f32, phase: f32, palette: Palette) -> Color32 {
    let color = if palette.span >= 0.9 {
        hsv(border_hue(palette, along, phase), 0.95, 1.0, 255)
    } else {
        blend_bands(palette.bands, wave_amount(along, phase))
    };
    with_alpha(color, 235)
}

pub(crate) fn paint_complement_line(
    painter: &Painter,
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

fn border_glow(along: f32, phase: f32, palette: Palette) -> Color32 {
    with_alpha(border_wave(along, phase, palette), 72)
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

fn paint_perimeter(painter: &Painter, rect: Rect, width: f32, color_at: impl Fn(f32) -> Color32) {
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
mod tests {
    use super::*;
    use crate::ink::Ink;

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
