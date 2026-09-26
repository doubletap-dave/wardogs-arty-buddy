# Wardogs Arty Buddy

Range in meters between two map points.

```
meters = 100 × √((enemy X − your X)² + (enemy Y − your Y)²)
```

That number is flat ground distance. The row under the coordinates adds height from a 10 m elevation grid: gun, target, and DZ (target minus gun). Pick the map first (BAK, OZE, ZES). Heights are meters above the lowest ground on that map, not sea level. A point past the surveyed edge, mostly a thin strip on Ozeti, shows a dash. The grids come from community terrain data published by [wardogs-calculator](https://github.com/apollyon-sys/wardogs-calculator).

## Download

Current release: **[v0.7.0](https://github.com/doubletap-dave/wardogs-arty-buddy/releases/tag/v0.7.0)**

- [Windows](https://github.com/doubletap-dave/wardogs-arty-buddy/releases/download/v0.7.0/wardogs-arty-buddy-windows-x86_64.exe)
- [Linux](https://github.com/doubletap-dave/wardogs-arty-buddy/releases/download/v0.7.0/wardogs-arty-buddy-linux-x86_64)

Older builds are on the [releases page](https://github.com/doubletap-dave/wardogs-arty-buddy/releases).

Windows may show SmartScreen the first time you open the download. Choose **More info**, then **Run anyway**.

The Linux build is made on Ubuntu 22.04. It needs Vulkan plus the usual X11 or Wayland libraries (`libvulkan1`, `libxkbcommon0`, `libxcb1`).

## How to use

The app is one small window. It opens transparent and stays above other windows, so you can leave it over the game. The colored squares change the ink. The solid squares are flat colors. The striped squares are RGB waves: a full rainbow, plus blue, purple, and red. RGB as a color scheme was KingPredict's idea.

It watches the clipboard.

1. Right-click where you are and choose get coordinates.
2. Press Ctrl+A so the whole coordinate line is selected, then Ctrl+C.
3. The first copy fills **OWN**. That is your gun. It stays there.
4. Right-click the place you want to shoot, get coordinates, then Ctrl+A and Ctrl+C again.
5. That copy fills **TGT** only. The range updates, shown as `500m`, with the compass bearing centered under it (`090°` is east). The height row shows gun, target, and DZ.
6. Keep copying new targets the same way. **OWN** does not move.
7. Click **CLEAR** when you want a new gun position. The next copy sets **OWN** again, and copies after that are targets.

A copied point looks like `x12.34, y56.78`. After the board accepts it, the clipboard is cleared, so a missed copy does not apply the same point twice. Copy the next point with Ctrl+A, Ctrl+C.

## From source

```
cargo run
```

Release builds run only when a `v*` tag is pushed. Every push to `main` still runs the tests.

A release build checks GitHub a few seconds after it opens, then about every six hours. If a newer tag is up, it downloads that build, replaces itself, and starts again. A `cargo run` debug build does not do this.

## Test

```
cargo test
```

## Not the game

WARDOGS, its maps, and its name belong to their owners. This is an unofficial fan tool. It is not affiliated with them, and the MIT license covers this project only, not the game.

## License

[MIT](LICENSE). Copyright (c) 2026 doubletap-dave.

This is open source. You can use it, change it, and ship it. Copies and anything derived from it must keep the copyright notice and this license, so the original work stays cited.

The bundled fonts in `assets/fonts` stay under the SIL Open Font License. Their license files are next to the font files.
