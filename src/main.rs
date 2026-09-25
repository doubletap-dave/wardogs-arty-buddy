use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

/// Meters between two map points.
///
/// `100 * sqrt((enemy_x - you_x)^2 + (enemy_y - you_y)^2)`
fn range_meters(you_x: f64, you_y: f64, enemy_x: f64, enemy_y: f64) -> f64 {
    let dx = enemy_x - you_x;
    let dy = enemy_y - you_y;
    dx.hypot(dy) * 100.0
}

fn format_meters(meters: f64) -> String {
    let rounded = (meters * 100.0).round() / 100.0;
    if (rounded - rounded.round()).abs() < 1e-9 {
        format!("{}", rounded.round() as i64)
    } else {
        format!("{rounded:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn print_usage() {
    println!(
        "\
Wardogs Arty Buddy

  wardogs-arty-buddy
      Prompt for your X Y, then the enemy X Y. Blank on Your X quits.

  wardogs-arty-buddy <your X> <your Y> <enemy X> <enemy Y>
      Print the range once and exit.

Meters = 100 × √((enemy X − your X)² + (enemy Y − your Y)²)"
    );
}

fn read_number(prompt: &str, blank_quits: bool) -> io::Result<Option<f64>> {
    loop {
        print!("{prompt}");
        io::stdout().flush()?;

        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            return Ok(None);
        }

        let text = line.trim();
        if text.is_empty() {
            if blank_quits {
                return Ok(None);
            }
            println!("Enter a number.");
            continue;
        }

        match text.parse::<f64>() {
            Ok(value) if value.is_finite() => return Ok(Some(value)),
            _ => println!("Enter a number."),
        }
    }
}

fn run_interactive() -> io::Result<()> {
    println!("Wardogs Arty Buddy");
    println!("Meters = 100 × distance between your point and the enemy point.");
    println!();

    loop {
        let Some(you_x) = read_number("Your X (blank to quit): ", true)? else {
            break;
        };
        let Some(you_y) = read_number("Your Y: ", false)? else {
            break;
        };
        let Some(enemy_x) = read_number("Enemy X: ", false)? else {
            break;
        };
        let Some(enemy_y) = read_number("Enemy Y: ", false)? else {
            break;
        };

        let meters = range_meters(you_x, you_y, enemy_x, enemy_y);
        println!();
        println!("Range: {} m", format_meters(meters));
        println!();
    }

    Ok(())
}

fn run_once(args: &[String]) -> Result<(), String> {
    if args.len() != 4 {
        return Err(
            "need four numbers: <your X> <your Y> <enemy X> <enemy Y>\n\
             run with no arguments to be prompted, or --help for usage"
                .to_string(),
        );
    }

    let mut values = [0.0; 4];
    for (slot, raw) in values.iter_mut().zip(args) {
        *slot = raw
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(|| format!("not a number: {raw}"))?;
    }

    let meters = range_meters(values[0], values[1], values[2], values[3]);
    println!("Range: {} m", format_meters(meters));
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let result = if args.is_empty() {
        run_interactive().map_err(|err| err.to_string())
    } else if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        print_usage();
        Ok(())
    } else {
        run_once(&args)
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("wardogs-arty-buddy: {err}");
            ExitCode::from(1)
        }
    }
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
    fn format_drops_a_trailing_decimal_on_whole_meters() {
        assert_eq!(format_meters(500.0), "500");
        assert_eq!(format_meters(141.421356), "141.42");
    }
}
