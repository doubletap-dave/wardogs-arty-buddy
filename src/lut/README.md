# WARDOGS terrain elevation lookup tables

Downsampled elevation grids for all three WARDOGS maps, built for the
wardogs-arty-buddy calculator.

## Source

Community-extracted Unreal Engine 5 Landscape collision heightfields from
[apollyon-sys/wardogs-calculator](https://github.com/apollyon-sys/wardogs-calculator)
(`docs/terrain.md`), fetched from their Cloudflare R2 release hosting:

`https://assets.wardogs-artillery.com/releases/assets-v1/data/terrain/<map-id>/`

Every one of the 768 source chunks (256 per map) was SHA-256 verified against
its `manifest.json` before decoding. Heights were decoded with the project's
exact formula (`worldZ = worldZOffsetMeters + localZ * worldZScaleMetersPerLocalUnit`,
bilinear over the 511x511 collision vertices) and resampled to a 10 m grid in
**game coordinates** (the same units the game copies to your clipboard:
1 unit = 100 m, +Y = north).

## Files

| file | contents |
|---|---|
| `<map>.bin` | raw little-endian `int16`, row-major (`samplesY` rows x `samplesX` columns) |
| `<map>.json` | metadata: origin, spacing, dimensions, quad mapping, datum note |

Each sample is **decimeters** (divide by 10 for meters). ` -32768` (`i16::MIN`)
means no data (outside surveyed coverage).

| map | grid | size | relief |
|---|---|---|---|
| bakurani | 1633 x 1633 | 5.3 MB | ~1192 m |
| ozeti | 1617 x 1617 | 5.2 MB | ~761 m |
| zestafona | 1633 x 1633 | 5.3 MB | ~682 m |

## Indexing

```text
ix = round((gameX - gameXMin) / 0.1)   # 0.1 game units = 10 m
iy = round((gameY - gameYMin) / 0.1)
sample = data[iy * samplesX + ix]      # decimeters; i16::MIN = no data
```

Row 0 is `gameYMin` (south edge); column 0 is `gameXMin` (west edge).

## Datum

Decoded heights sit on the source dataset's **offset datum** (hundreds of
meters off absolute). Only **differences** between samples are meaningful:

```text
dZ = targetElevation - gunElevation
```

That is all artillery needs. Do not display these as altitudes.

## Validation

Re-decoded the four corners + center of each map straight from the source
chunks and compared against the stored grid: max error 0.046 m, i.e. pure
decimeter-rounding noise. Ozeti has 3233 sentinel samples where the 10 m grid
steps just past the surveyed coverage edge; the other maps have none.

## Rust sketch

```rust
use std::fs;

pub struct TerrainLut {
    pub game_x_min: f64,
    pub game_y_min: f64,
    pub nx: usize,
    pub ny: usize,
    pub data: Vec<i16>, // decimeters, row-major; i16::MIN = no data
}

impl TerrainLut {
    pub fn load(bin_path: &str) -> std::io::Result<Self> {
        let meta_path = bin_path.replace(".bin", ".json");
        let meta: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&meta_path)?).unwrap();
        let raw = fs::read(bin_path)?;
        let data = raw
            .chunks_exact(2)
            .map(|c| i16::from_le_bytes([c[0], c[1]]))
            .collect();
        Ok(Self {
            game_x_min: meta["gameXMin"].as_f64().unwrap(),
            game_y_min: meta["gameYMin"].as_f64().unwrap(),
            nx: meta["samplesX"].as_u64().unwrap() as usize,
            ny: meta["samplesY"].as_u64().unwrap() as usize,
            data,
        })
    }

    /// Bilinear elevation in meters at game coords, or None outside coverage.
    pub fn elevation(&self, gx: f64, gy: f64) -> Option<f64> {
        let fx = (gx - self.game_x_min) / 0.1;
        let fy = (gy - self.game_y_min) / 0.1;
        if fx < 0.0 || fy < 0.0 || fx > (self.nx - 1) as f64 || fy > (self.ny - 1) as f64 {
            return None;
        }
        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(self.nx - 1);
        let y1 = (y0 + 1).min(self.ny - 1);
        let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
        let at = |x: usize, y: usize| {
            let v = self.data[y * self.nx + x];
            (v != i16::MIN).then_some(v as f64 / 10.0)
        };
        let (z00, z10, z01, z11) = (at(x0, y0)?, at(x1, y0)?, at(x0, y1)?, at(x1, y1)?);
        let top = z00 + (z10 - z00) * tx;
        let bot = z01 + (z11 - z01) * tx;
        Some(top + (bot - top) * ty)
    }
}
```
