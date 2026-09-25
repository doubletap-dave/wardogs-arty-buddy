# Wardogs Arty Buddy

Range in meters between two map points.

```
meters = 100 × √((enemy X − your X)² + (enemy Y − your Y)²)
```

## Run

```
cargo run
```

A window opens. Enter your X and Y, then the target X and Y. The range updates as you type. Clear board wipes the four fields.

A Wardogs copy pastes into either box of a station: `x12.34, y56.78`. One to three digits before the decimal, two after. Plain numbers still work.

GAME shrinks the board, keeps it above other windows, and makes the plate translucent so the game shows through. Pick GREEN, RED, BLUE, WHITE, or AMBER. PASS CLICKS lets the mouse fall through to the game. Alt+Tab back to this window and press Esc to take the mouse again. BOARD returns to the full layout.

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
