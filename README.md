# Wardogs Arty Buddy

Range in meters between two map points.

```
meters = 100 × √((enemy X − your X)² + (enemy Y − your Y)²)
```

## Run

Prompt for coordinates. A blank line on Your X quits.

```
cargo run
```

Or pass all four numbers:

```
cargo run -- 0 0 3 4
```

```
Range: 500 m
```

## Test

```
cargo test
```
