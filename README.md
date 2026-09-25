# Wardogs Arty Buddy

Range in meters between two map points.

```
meters = 100 × √((enemy X − your X)² + (enemy Y − your Y)²)
```

## How to use

Open the app, then copy points out of Wardogs. The board is watching the clipboard.

1. Right-click where you are and choose get coordinates.
2. Press Ctrl+A so the whole coordinate line is selected, then Ctrl+C.
3. The first copy fills **Own station**. That is your gun. It stays there.
4. Right-click the place you want to shoot, get coordinates, then Ctrl+A and Ctrl+C again.
5. That copy fills **Target** only. The range updates, shown as `500m`.
6. Keep copying new targets the same way. Own station does not move.
7. Click **Clear board** when you want a new gun position. The next copy sets Own station again, and copies after that are targets.

A copied point looks like `x12.34, y56.78`. After the board accepts it, the clipboard is cleared, so a missed copy does not apply the same point twice. Copy the next point with Ctrl+A, Ctrl+C.

**GAME** shrinks the board, turns it transparent, and keeps it above other windows so it can sit over the game without covering the screen. The colored squares change the ink. **CLEAR** does the same thing as Clear board. **BACK** returns to the full window.

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
