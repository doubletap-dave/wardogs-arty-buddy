const SPACING: f64 = 0.1;
const NO_DATA: i16 = i16::MIN;

struct Grid {
    x_min: f64,
    y_min: f64,
    nx: usize,
    ny: usize,
    samples: &'static [u8],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Map {
    Bakurani,
    Ozeti,
    Zestafona,
}

impl Map {
    pub(crate) const ALL: [Self; 3] = [Self::Bakurani, Self::Ozeti, Self::Zestafona];

    pub(crate) fn short(self) -> &'static str {
        match self {
            Self::Bakurani => "BAK",
            Self::Ozeti => "OZE",
            Self::Zestafona => "ZES",
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Bakurani => "Bakurani",
            Self::Ozeti => "Ozeti",
            Self::Zestafona => "Zestafona",
        }
    }

    fn grid(self) -> Grid {
        match self {
            Self::Bakurani => Grid {
                x_min: 0.0,
                y_min: 0.0,
                nx: 1633,
                ny: 1633,
                samples: include_bytes!("lut/bakurani.bin"),
            },
            Self::Ozeti => Grid {
                x_min: 0.0,
                y_min: 1.62,
                nx: 1617,
                ny: 1617,
                samples: include_bytes!("lut/ozeti.bin"),
            },
            Self::Zestafona => Grid {
                x_min: 0.0,
                y_min: 0.0,
                nx: 1633,
                ny: 1633,
                samples: include_bytes!("lut/zestafona.bin"),
            },
        }
    }

    /// Meters on this map's own datum, or nothing if the point is off the grid.
    pub(crate) fn elevation(self, x: f64, y: f64) -> Option<f64> {
        self.grid().elevation(x, y)
    }
}

impl Grid {
    fn elevation(&self, gx: f64, gy: f64) -> Option<f64> {
        let fx = (gx - self.x_min) / SPACING;
        let fy = (gy - self.y_min) / SPACING;
        if fx < 0.0 || fy < 0.0 || fx > (self.nx - 1) as f64 || fy > (self.ny - 1) as f64 {
            return None;
        }
        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(self.nx - 1);
        let y1 = (y0 + 1).min(self.ny - 1);
        let tx = fx - x0 as f64;
        let ty = fy - y0 as f64;
        let z00 = self.meters_at(x0, y0)?;
        let z10 = self.meters_at(x1, y0)?;
        let z01 = self.meters_at(x0, y1)?;
        let z11 = self.meters_at(x1, y1)?;
        let top = z00 + (z10 - z00) * tx;
        let bot = z01 + (z11 - z01) * tx;
        Some(top + (bot - top) * ty)
    }

    fn meters_at(&self, x: usize, y: usize) -> Option<f64> {
        let index = (y * self.nx + x) * 2;
        let bytes = self.samples.get(index..index + 2)?;
        let sample = i16::from_le_bytes([bytes[0], bytes[1]]);
        (sample != NO_DATA).then_some(sample as f64 / 10.0)
    }
}

pub(crate) fn elevation_of(map: Map, x: &str, y: &str) -> Option<f64> {
    let x = x.trim().parse::<f64>().ok()?;
    let y = y.trim().parse::<f64>().ok()?;
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    map.elevation(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(samples: &'static [u8], nx: usize, ny: usize) -> Grid {
        Grid {
            x_min: 0.0,
            y_min: 0.0,
            nx,
            ny,
            samples,
        }
    }

    #[test]
    fn bilinear_midpoint_is_the_average() {
        // 2x2, 10 m cells. Values are decimeters: 0, 100, 0, 0.
        let raw: &'static [u8] = &[0, 0, 100, 0, 0, 0, 0, 0];
        let height = grid(raw, 2, 2).elevation(0.05, 0.0).unwrap();
        assert!((height - 5.0).abs() < 1e-9, "{height}");
    }

    #[test]
    fn sentinel_is_no_data() {
        let raw: &'static [u8] = &[0, 0, 0, 128, 0, 0, 0, 0];
        assert!(grid(raw, 2, 2).elevation(0.05, 0.0).is_none());
    }

    #[test]
    fn outside_the_grid_is_none() {
        assert!(Map::Bakurani.elevation(-1.0, 10.0).is_none());
        assert!(Map::Ozeti.elevation(10.0, 0.0).is_none());
    }

    #[test]
    fn bakurani_center_has_a_height() {
        let height = Map::Bakurani.elevation(80.0, 80.0).unwrap();
        assert!(height.is_finite());
    }
}
