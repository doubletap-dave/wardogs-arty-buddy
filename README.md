# Wardogs Arty Buddy

Range in meters between two map points.

```
meters = 100 × √((enemy X − your X)² + (enemy Y − your Y)²)
```

## Run

```
cargo run
```

Copy a Wardogs point (`x12.34, y56.78`) and the window picks it up. The first copy is your station and stays there. Every copy after that moves the target and updates the range. Each copy is cleared off the clipboard so a missed copy does not reuse the old point. Clear board unlocks your station, and the next copy sets it again.

GAME shrinks the board, keeps it above the game, and makes it translucent. The colored squares switch the ink. CLEAR resets your station. BACK returns to the full window.

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
