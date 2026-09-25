# Wardogs Arty Buddy

Range in meters between two map points.

```
meters = 100 × √((enemy X − your X)² + (enemy Y − your Y)²)
```

## How to use

The app is one small window. It opens transparent and stays above other windows, so you can leave it over the game. The colored squares change the ink.

It watches the clipboard.

1. Right-click where you are and choose get coordinates.
2. Press Ctrl+A so the whole coordinate line is selected, then Ctrl+C.
3. The first copy fills **OWN**. That is your gun. It stays there.
4. Right-click the place you want to shoot, get coordinates, then Ctrl+A and Ctrl+C again.
5. That copy fills **TGT** only. The range updates, shown as `500m`.
6. Keep copying new targets the same way. **OWN** does not move.
7. Click **CLEAR** when you want a new gun position. The next copy sets **OWN** again, and copies after that are targets.

A copied point looks like `x12.34, y56.78`. After the board accepts it, the clipboard is cleared, so a missed copy does not apply the same point twice. Copy the next point with Ctrl+A, Ctrl+C.

From a source checkout:

```
cargo run
```

## Downloads

Tagged releases attach two builds:

- `wardogs-arty-buddy-windows-x86_64.exe`
- `wardogs-arty-buddy-linux-x86_64`

The Linux build is made on Ubuntu 22.04. It needs Vulkan plus the usual X11 or Wayland libraries (`libvulkan1`, `libxkbcommon0`, `libxcb1`).

Release builds run only when a `v*` tag is pushed. Every push to `main` still runs the tests.

## Test

```
cargo test
```

## License

[MIT](LICENSE). Copyright (c) 2026 doubletap-dave.

This is open source. You can use it, change it, and ship it. Copies and anything derived from it must keep the copyright notice and this license, so the original work stays cited.

The bundled fonts in `assets/fonts` stay under the SIL Open Font License. Their license files are next to the font files.
