# timer_data

<!-- simit:badges:start -->

[![CI](https://img.shields.io/badge/CI-drift-2088ff)](.forgejo/workflows/ci.yaml) [![crates.io](https://img.shields.io/badge/crates.io-ready-f46623)](https://crates.io/crates/timer_data)

<!-- simit:badges:end -->

Serializer-independent data types for Bevy's `Timer`.

Bevy's `Timer` doesn't implement common serialization traits. This crate provides `TimerData` and `TimerModeData` as plain data types with optional `From`/`Into` conversions to Bevy types, plus optional serde and rkyv derives behind feature flags.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `bevy`  | Yes | `bevy_time` dependency + `From`/`Into` conversions for `Timer`/`TimerMode` |
| `serde` | Yes | `Serialize`/`Deserialize` derives. With `bevy`, also `#[serde(with)]` helpers |
| `rkyv`  | No  | `Archive`/`Serialize`/`Deserialize` derives (rkyv 0.8) |

## Usage

### Serde (`#[serde(with)]`) — requires `bevy` + `serde`

```rust
use serde::{Serialize, Deserialize};
use bevy_time::Timer;

#[derive(Serialize, Deserialize)]
struct MyComponent {
    #[serde(with = "timer_data")]
    timer: Timer,
}
```

### Direct conversion — requires `bevy`

```rust
use bevy_time::{Timer, TimerMode};
use timer_data::TimerData;

let timer = Timer::from_seconds(5.0, TimerMode::Once);
let data: TimerData = (&timer).into();
let restored: Timer = data.into();
```

### Without Bevy

```rust
use timer_data::{TimerData, TimerModeData};

let data = TimerData::new(5_000_000_000, 0, false, TimerModeData::Once);
assert_eq!(data.duration(), std::time::Duration::from_secs(5));
```

## Bevy Compatibility

| timer_data | Bevy  |
|-----------------|-------|
| 0.1             | 0.18  |

## CI

Woodpecker CI on Codeberg runs `cargo build`, `cargo test`, `cargo clippy`, and `cargo fmt --check` on every push and pull request.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
