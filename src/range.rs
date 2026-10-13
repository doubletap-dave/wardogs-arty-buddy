/// Meters between two map points.
///
/// `100 * sqrt((enemy_x - you_x)^2 + (enemy_y - you_y)^2)`
pub fn range_meters(you_x: f64, you_y: f64, enemy_x: f64, enemy_y: f64) -> f64 {
    let dx = enemy_x - you_x;
    let dy = enemy_y - you_y;
    dx.hypot(dy) * 100.0
}

/// Compass degrees, clockwise from north. +Y is north and +X is east.
pub fn bearing_degrees(dx: f64, dy: f64) -> f64 {
    dx.atan2(dy).to_degrees().rem_euclid(360.0)
}

pub fn format_bearing(degrees: f64) -> String {
    let rounded = degrees.round() as i64;
    format!("{:03}°", rounded.rem_euclid(360))
}

pub fn format_number(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if (rounded - rounded.round()).abs() < 1e-9 {
        format!("{}", rounded.round() as i64)
    } else {
        format!("{rounded:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordField {
    YouX,
    YouY,
    EnemyX,
    EnemyY,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fix {
    pub dx: f64,
    pub dy: f64,
    pub grid: f64,
    pub flat_meters: f64,
    pub meters: f64,
    pub bearing: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoardRead {
    Need(CoordField),
    Fault(CoordField),
    Ready(Fix),
}

/// A Wardogs copy is `x12.34, y56.78`: one to three digits before the
/// decimal, always two after. Returns the two numeric strings.
pub fn wardogs_pair(text: &str) -> Option<(String, String)> {
    let rest = strip_axis(text.trim(), 'x')?;
    let (x, rest) = take_fixed(rest)?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(',')?.trim_start();
    let rest = strip_axis(rest, 'y')?;
    let (y, rest) = take_fixed(rest)?;
    if rest.trim().is_empty() {
        Some((x, y))
    } else {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipUpdate {
    Own { x: String, y: String },
    Target { x: String, y: String },
}

/// First Wardogs copy is your station. Later copies are the target until the board is cleared.
pub fn update_from_clip(own_locked: bool, text: &str) -> Option<ClipUpdate> {
    let (x, y) = wardogs_pair(text)?;
    if own_locked {
        Some(ClipUpdate::Target { x, y })
    } else {
        Some(ClipUpdate::Own { x, y })
    }
}

/// `x12.34` or `y56.78` on its own.
pub fn wardogs_single(text: &str, axis: char) -> Option<String> {
    let rest = strip_axis(text.trim(), axis)?;
    let (value, rest) = take_fixed(rest)?;
    if rest.trim().is_empty() {
        Some(value)
    } else {
        None
    }
}

/// If either box holds a full Wardogs copy, fill both. A lone `x` or `y`
/// value is stripped down to its number.
pub fn absorb_wardogs(x: &mut String, y: &mut String) {
    if let Some((next_x, next_y)) = wardogs_pair(x).or_else(|| wardogs_pair(y)) {
        *x = next_x;
        *y = next_y;
        return;
    }
    if let Some(next_x) = wardogs_single(x, 'x') {
        *x = next_x;
    }
    if let Some(next_y) = wardogs_single(y, 'y') {
        *y = next_y;
    }
}

fn strip_axis(text: &str, axis: char) -> Option<&str> {
    let mut chars = text.chars();
    let first = chars.next()?;
    if !first.eq_ignore_ascii_case(&axis) {
        return None;
    }
    Some(chars.as_str().trim_start())
}

fn take_fixed(text: &str) -> Option<(String, &str)> {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut digits = 0;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        digits += 1;
        if digits > 3 {
            return None;
        }
        index += 1;
    }
    if digits < 1 || index >= bytes.len() || bytes[index] != b'.' {
        return None;
    }
    index += 1;
    let frac = index;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index - frac != 2 {
        return None;
    }
    Some((text[..index].to_owned(), &text[index..]))
}

const SPH_LOW_TABLE: &[(f64, f64)] = &[
    (1181.0, 20.0),
    (1232.0, 30.0),
    (1283.0, 40.0),
    (1334.0, 50.0),
    (1384.0, 60.0),
    (1433.0, 70.0),
    (1482.0, 80.0),
    (1529.0, 90.0),
    (1576.0, 100.0),
    (1622.0, 110.0),
    (1666.0, 120.0),
    (1709.0, 130.0),
    (1751.0, 140.0),
    (1792.0, 150.0),
    (1832.0, 160.0),
    (1870.0, 170.0),
    (1907.0, 180.0),
    (1944.0, 190.0),
    (1979.0, 200.0),
    (2014.0, 210.0),
    (2046.0, 220.0),
    (2079.0, 230.0),
    (2110.0, 240.0),
    (2139.0, 250.0),
    (2168.0, 260.0),
    (2196.0, 270.0),
    (2223.0, 280.0),
    (2249.0, 290.0),
    (2273.0, 300.0),
    (2296.0, 310.0),
    (2319.0, 320.0),
    (2341.0, 330.0),
    (2362.0, 340.0),
    (2383.0, 350.0),
    (2403.0, 360.0),
    (2422.0, 370.0),
    (2439.0, 380.0),
    (2456.0, 390.0),
    (2471.0, 400.0),
    (2485.0, 410.0),
    (2499.0, 420.0),
    (2513.0, 430.0),
    (2526.0, 440.0),
    (2538.0, 450.0),
    (2550.0, 460.0),
    (2561.0, 470.0),
    (2570.0, 480.0),
    (2579.0, 490.0),
    (2586.0, 500.0),
    (2593.0, 510.0),
    (2599.0, 520.0),
    (2605.0, 530.0),
    (2610.0, 540.0),
    (2615.0, 550.0),
    (2620.0, 560.0),
    (2623.0, 570.0),
    (2626.0, 580.0),
    (2628.0, 590.0),
    (2629.0, 600.0),
];

const GC_X: &[f64] = &[
    0.0, 30.0, 61.0, 91.0, 122.0, 150.0, 183.0, 213.0, 244.0, 274.0, 300.0, 335.0, 366.0, 396.0,
    427.0, 450.0, 488.0, 518.0, 549.0, 579.0, 600.0,
];
const GC_Y: &[f64] = &[
    0.71, 0.69, 0.66, 0.64, 0.65, 0.66, 0.68, 0.72, 0.80, 0.92, 1.10, 1.25, 1.33, 1.39, 1.44, 1.48,
    1.50, 1.54, 1.57, 1.58, 1.60,
];

const SPH_SPEED: f64 = 300.0;
const SPH_GRAVITY: f64 = 8.85397505384648;
const SPH_DRAG: f64 = 0.0007351626163587786;
const SPH_OFFSET: f64 = 95.03019643400405;

fn drag_coefficient(speed: f64) -> f64 {
    if speed <= GC_X[0] {
        return GC_Y[0];
    }
    let last = GC_X.len() - 1;
    if speed >= GC_X[last] {
        return GC_Y[last];
    }
    let mut lo = 0;
    let mut hi = last;
    while lo + 1 < hi {
        let mid = (lo + hi) >> 1;
        if GC_X[mid] <= speed {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let fraction = (speed - GC_X[lo]) / (GC_X[hi] - GC_X[lo]);
    GC_Y[lo] + (GC_Y[hi] - GC_Y[lo]) * fraction
}

fn drag_factor(vx: f64, vz: f64) -> f64 {
    let speed = vx.hypot(vz);
    SPH_DRAG * drag_coefficient(speed) * speed
}

fn height_at_distance(angle: f64, distance: f64) -> Option<f64> {
    if !angle.is_finite() || angle <= -0.5 || angle >= 1.55 || !distance.is_finite() || distance <= 0.0 {
        return None;
    }
    let dt = 0.02;
    let mut x = 0.0;
    let mut z = 0.0;
    let mut vx = SPH_SPEED * angle.cos();
    let mut vz = SPH_SPEED * angle.sin();
    for _ in 0..6000 {
        let q1 = drag_factor(vx, vz);
        let (a1x, a1z) = (-q1 * vx, -SPH_GRAVITY - q1 * vz);
        let (v2x, v2z) = (vx + a1x * dt * 0.5, vz + a1z * dt * 0.5);
        let q2 = drag_factor(v2x, v2z);
        let (a2x, a2z) = (-q2 * v2x, -SPH_GRAVITY - q2 * v2z);
        let (v3x, v3z) = (vx + a2x * dt * 0.5, vz + a2z * dt * 0.5);
        let q3 = drag_factor(v3x, v3z);
        let (a3x, a3z) = (-q3 * v3x, -SPH_GRAVITY - q3 * v3z);
        let (v4x, v4z) = (vx + a3x * dt, vz + a3z * dt);
        let q4 = drag_factor(v4x, v4z);

        let next_x = x + dt * (vx + 2.0 * v2x + 2.0 * v3x + v4x) / 6.0;
        let next_z = z + dt * (vz + 2.0 * v2z + 2.0 * v3z + v4z) / 6.0;
        if !next_x.is_finite() || !next_z.is_finite() || !v4x.is_finite() || !v4z.is_finite() {
            return None;
        }
        if next_x >= distance {
            return Some(z + (next_z - z) * (distance - x) / (next_x - x) + SPH_OFFSET);
        }
        vx += dt * (a1x + 2.0 * a2x + 2.0 * a3x - q4 * v4x) / 6.0;
        vz += dt * (a1z + 2.0 * a2z + 2.0 * a3z - SPH_GRAVITY - q4 * v4z) / 6.0;
        x = next_x;
        z = next_z;
        if vx <= 1e-7 {
            return None;
        }
    }
    None
}

fn bisect(mut left: f64, mut right: f64, tol: f64, f: impl Fn(f64) -> Option<f64>) -> Option<f64> {
    let mut fl = f(left)?;
    let fr = f(right)?;
    if fl * fr > 0.0 {
        return None;
    }
    if fl.abs() < 1e-8 {
        return Some(left);
    }
    if fr.abs() < 1e-8 {
        return Some(right);
    }
    for _ in 0..40 {
        if right - left <= tol {
            break;
        }
        let mid = (left + right) * 0.5;
        let fm = f(mid)?;
        if fm.abs() < 1e-8 {
            return Some(mid);
        }
        if fl * fm <= 0.0 {
            right = mid;
        } else {
            left = mid;
            fl = fm;
        }
    }
    Some((left + right) * 0.5)
}

fn dist_to_flat_mil(distance: f64) -> Option<f64> {
    if distance < SPH_LOW_TABLE[0].0 {
        let (d0, m0) = SPH_LOW_TABLE[0];
        let (d1, m1) = SPH_LOW_TABLE[1];
        return Some(m0 + (distance - d0) * (m1 - m0) / (d1 - d0));
    }
    for window in SPH_LOW_TABLE.windows(2) {
        let (d0, m0) = window[0];
        let (d1, m1) = window[1];
        if distance >= d0 && distance <= d1 {
            let t = (distance - d0) / (d1 - d0);
            return Some(m0 + t * (m1 - m0));
        }
    }
    None
}

fn mil_to_flat_dist(mil: f64) -> Option<f64> {
    if mil < SPH_LOW_TABLE[0].1 {
        let (d0, m0) = SPH_LOW_TABLE[0];
        let (d1, m1) = SPH_LOW_TABLE[1];
        return Some((d0 + (mil - m0) * (d1 - d0) / (m1 - m0)).max(0.0));
    }
    for window in SPH_LOW_TABLE.windows(2) {
        let (d0, m0) = window[0];
        let (d1, m1) = window[1];
        if mil >= m0 && mil <= m1 {
            let t = (mil - m0) / (m1 - m0);
            return Some(d0 + t * (d1 - d0));
        }
    }
    None
}

/// Terrain-adjusted effective firing distance in meters for a given flat range and DZ (target - gun).
pub fn ballistic_range_meters(flat_meters: f64, dz_meters: f64) -> f64 {
    if !flat_meters.is_finite() || !dz_meters.is_finite() || dz_meters.abs() < 1e-6 || flat_meters <= 0.0 {
        return flat_meters;
    }
    let Some(flat_mil) = dist_to_flat_mil(flat_meters) else {
        return (flat_meters + dz_meters).max(0.0);
    };
    let center = flat_mil / 1000.0;
    let ref_angle = bisect(
        (center - 0.08).max(-0.35),
        (center + 0.08).min(0.61),
        1e-7,
        |a| height_at_distance(a, flat_meters),
    )
    .unwrap_or(center);

    let cmd_mil = bisect(-300.0, 600.0, 0.025, |cmd| {
        let angle = ref_angle + (cmd - flat_mil) / 1000.0;
        Some(height_at_distance(angle, flat_meters)? - dz_meters)
    });

    cmd_mil
        .and_then(mil_to_flat_dist)
        .unwrap_or_else(|| (flat_meters + dz_meters).max(0.0))
}

#[cfg(test)]
pub fn read_board(you_x: &str, you_y: &str, enemy_x: &str, enemy_y: &str) -> BoardRead {
    read_board_with_dz(you_x, you_y, enemy_x, enemy_y, None)
}

pub fn read_board_with_dz(
    you_x: &str,
    you_y: &str,
    enemy_x: &str,
    enemy_y: &str,
    dz: Option<f64>,
) -> BoardRead {
    let fields = [
        (CoordField::YouX, you_x),
        (CoordField::YouY, you_y),
        (CoordField::EnemyX, enemy_x),
        (CoordField::EnemyY, enemy_y),
    ];

    let mut parsed = [0.0; 4];
    let mut empty = None;

    for (index, (field, text)) in fields.iter().copied().enumerate() {
        let text = text.trim();
        if text.is_empty() {
            if empty.is_none() {
                empty = Some(field);
            }
            continue;
        }
        match text.parse::<f64>() {
            Ok(value) if value.is_finite() => parsed[index] = value,
            _ => return BoardRead::Fault(field),
        }
    }

    if let Some(field) = empty {
        return BoardRead::Need(field);
    }

    let dx = parsed[2] - parsed[0];
    let dy = parsed[3] - parsed[1];
    let flat_meters = range_meters(parsed[0], parsed[1], parsed[2], parsed[3]);
    let meters = dz.map_or(flat_meters, |dz| ballistic_range_meters(flat_meters, dz));
    BoardRead::Ready(Fix {
        dx,
        dy,
        grid: dx.hypot(dy),
        flat_meters,
        meters,
        bearing: bearing_degrees(dx, dy),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f64, expected: f64) {
        let tol = 1e-9_f64.max(expected.abs() * 1e-9);
        assert!(
            (actual - expected).abs() <= tol,
            "actual {actual} expected {expected}"
        );
    }

    #[test]
    fn same_point_is_zero() {
        close(range_meters(12.0, -4.0, 12.0, -4.0), 0.0);
    }

    #[test]
    fn three_four_five_is_five_hundred_meters() {
        close(range_meters(0.0, 0.0, 3.0, 4.0), 500.0);
        close(range_meters(10.0, 20.0, 13.0, 24.0), 500.0);
        close(range_meters(5.0, 5.0, 2.0, 1.0), 500.0);
    }

    #[test]
    fn order_of_the_two_points_does_not_matter() {
        close(
            range_meters(1.5, -2.0, 8.0, 4.25),
            range_meters(8.0, 4.25, 1.5, -2.0),
        );
    }

    #[test]
    fn one_grid_step_on_x_is_one_hundred_meters() {
        close(range_meters(0.0, 0.0, 1.0, 0.0), 100.0);
    }

    #[test]
    fn bearing_matches_the_compass() {
        close(bearing_degrees(0.0, 1.0), 0.0);
        close(bearing_degrees(1.0, 0.0), 90.0);
        close(bearing_degrees(0.0, -1.0), 180.0);
        close(bearing_degrees(-1.0, 0.0), 270.0);
        close(bearing_degrees(1.0, 1.0), 45.0);
        close(bearing_degrees(-1.0, 1.0), 315.0);
        assert_eq!(format_bearing(359.6), "000°");
        assert_eq!(format_bearing(90.2), "090°");
        assert_eq!(format_bearing(270.0), "270°");
    }

    #[test]
    fn diagonal_unit_step_is_one_hundred_root_two() {
        close(range_meters(0.0, 0.0, 1.0, 1.0), 100.0 * 2.0_f64.sqrt());
    }

    #[test]
    fn format_drops_a_trailing_decimal_on_whole_numbers() {
        assert_eq!(format_number(500.0), "500");
        assert_eq!(format_number(141.421356), "141.42");
    }

    #[test]
    fn board_waits_for_the_first_empty_field() {
        assert_eq!(
            read_board("", "", "", ""),
            BoardRead::Need(CoordField::YouX)
        );
        assert_eq!(
            read_board("1", "2", "", "4"),
            BoardRead::Need(CoordField::EnemyX)
        );
    }

    #[test]
    fn board_reports_the_first_bad_field() {
        assert_eq!(
            read_board("1", "nope", "3", "4"),
            BoardRead::Fault(CoordField::YouY)
        );
    }

    #[test]
    fn board_solves_a_three_four_five() {
        match read_board("0", "0", "3", "4") {
            BoardRead::Ready(fix) => {
                close(fix.meters, 500.0);
                close(fix.grid, 5.0);
                close(fix.dx, 3.0);
                close(fix.dy, 4.0);
            }
            other => panic!("expected a fix, got {other:?}"),
        }
    }

    #[test]
    fn wardogs_copy_accepts_one_to_three_digits() {
        assert_eq!(
            wardogs_pair("x1.00, y2.50"),
            Some(("1.00".into(), "2.50".into()))
        );
        assert_eq!(
            wardogs_pair("x12.34, y5.00"),
            Some(("12.34".into(), "5.00".into()))
        );
        assert_eq!(
            wardogs_pair("X123.45, Y67.89"),
            Some(("123.45".into(), "67.89".into()))
        );
        assert_eq!(
            wardogs_pair("x12.34,y67.80"),
            Some(("12.34".into(), "67.80".into()))
        );
    }

    #[test]
    fn wardogs_copy_rejects_the_wrong_shape() {
        assert_eq!(wardogs_pair("x12.3, y1.00"), None);
        assert_eq!(wardogs_pair("x1234.00, y1.00"), None);
        assert_eq!(wardogs_pair("x1.00 y2.00"), None);
        assert_eq!(wardogs_pair("12.00, 34.00"), None);
    }

    #[test]
    fn absorb_splits_a_pasted_pair_and_keeps_plain_numbers() {
        let mut x = "x0.00, y0.00".to_owned();
        let mut y = String::new();
        absorb_wardogs(&mut x, &mut y);
        assert_eq!((x.as_str(), y.as_str()), ("0.00", "0.00"));

        let mut x = "3".to_owned();
        let mut y = "4".to_owned();
        absorb_wardogs(&mut x, &mut y);
        assert_eq!((x.as_str(), y.as_str()), ("3", "4"));

        let mut x = String::new();
        let mut y = "x3.00, y4.00".to_owned();
        absorb_wardogs(&mut x, &mut y);
        assert_eq!((x.as_str(), y.as_str()), ("3.00", "4.00"));
    }

    #[test]
    fn first_copy_locks_own_and_later_copies_move_the_target() {
        assert_eq!(
            update_from_clip(false, "x10.00, y20.00"),
            Some(ClipUpdate::Own {
                x: "10.00".into(),
                y: "20.00".into(),
            })
        );
        assert_eq!(
            update_from_clip(true, "x13.00, y24.00"),
            Some(ClipUpdate::Target {
                x: "13.00".into(),
                y: "24.00".into(),
            })
        );
        assert_eq!(update_from_clip(true, "hello"), None);
    }

    #[test]
    fn pasted_wardogs_copies_solve_to_meters() {
        let mut you_x = "x10.00, y20.00".to_owned();
        let mut you_y = String::new();
        let mut enemy_x = "x13.00, y24.00".to_owned();
        let mut enemy_y = String::new();
        absorb_wardogs(&mut you_x, &mut you_y);
        absorb_wardogs(&mut enemy_x, &mut enemy_y);
        match read_board(&you_x, &you_y, &enemy_x, &enemy_y) {
            BoardRead::Ready(fix) => close(fix.meters, 500.0),
            other => panic!("expected a fix, got {other:?}"),
        }
    }

    #[test]
    fn ballistic_range_preserves_flat_distance_when_dz_is_zero() {
        close(ballistic_range_meters(1400.0, 0.0), 1400.0);
        close(ballistic_range_meters(500.0, 0.0), 500.0);
    }

    #[test]
    fn paladin_ballistic_range_compensates_uphill_and_downhill_shots() {
        let uphill = ballistic_range_meters(1400.0, 40.0);
        let downhill = ballistic_range_meters(1400.0, -40.0);
        // At 1400m on the Paladin (SPH-2), +40m DZ adds ~139m and -40m DZ subtracts ~145m.
        assert!(
            (uphill - 1539.4).abs() < 2.0,
            "expected uphill ~1539.4m, got {uphill}"
        );
        assert!(
            (downhill - 1254.9).abs() < 2.0,
            "expected downhill ~1254.9m, got {downhill}"
        );
    }

    #[test]
    fn read_board_with_dz_adjusts_meters_and_keeps_flat_meters() {
        match read_board_with_dz("0", "0", "0", "14", Some(-40.0)) {
            BoardRead::Ready(fix) => {
                close(fix.flat_meters, 1400.0);
                assert!(
                    (fix.meters - 1254.9).abs() < 2.0,
                    "expected ~1254.9m, got {}",
                    fix.meters
                );
            }
            other => panic!("expected a fix, got {other:?}"),
        }
    }
}
