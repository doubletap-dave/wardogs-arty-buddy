/// Meters between two map points.
///
/// `100 * sqrt((enemy_x - you_x)^2 + (enemy_y - you_y)^2)`
pub fn range_meters(you_x: f64, you_y: f64, enemy_x: f64, enemy_y: f64) -> f64 {
    let dx = enemy_x - you_x;
    let dy = enemy_y - you_y;
    dx.hypot(dy) * 100.0
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

impl CoordField {
    pub fn label(self) -> &'static str {
        match self {
            Self::YouX => "Your X",
            Self::YouY => "Your Y",
            Self::EnemyX => "Target X",
            Self::EnemyY => "Target Y",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fix {
    pub dx: f64,
    pub dy: f64,
    pub grid: f64,
    pub meters: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoardRead {
    Need(CoordField),
    Fault(CoordField),
    Ready(Fix),
}

pub fn read_board(you_x: &str, you_y: &str, enemy_x: &str, enemy_y: &str) -> BoardRead {
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
    BoardRead::Ready(Fix {
        dx,
        dy,
        grid: dx.hypot(dy),
        meters: range_meters(parsed[0], parsed[1], parsed[2], parsed[3]),
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
}
