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

## Test

```
cargo test
```
