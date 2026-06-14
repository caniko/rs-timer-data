# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## Project

`timer_data` — a Rust library providing serializer-independent data types for Bevy's `Timer`. Bevy's `Timer` lacks serialization traits; this crate bridges that gap with `TimerData` and `TimerModeData` as plain data types with optional conversions and serialization derives.

Bevy compatibility: **0.18**. MSRV: **1.70**.

## Build Commands

```bash
cargo build                        # default features (bevy + serde)
cargo build --no-default-features  # standalone, no bevy or serde
cargo build --all-features         # all features including rkyv
cargo test                         # run all tests (needs dev-deps: bevy_time, serde, serde_json)
cargo test --no-default-features   # test without optional features
```

## Feature Flags

- `bevy` (default) — enables `bevy_time` dep and `From`/`Into` conversions between Bevy Timer types and data types
- `serde` (default) — enables `Serialize`/`Deserialize` derives; combined with `bevy`, enables `#[serde(with = "timer_data")]` helpers
- `rkyv` — enables rkyv 0.8 `Archive`/`Serialize`/`Deserialize` derives

## Architecture

Single-file library: [src/lib.rs](src/lib.rs). No submodules.

**Core types:**
- `TimerData` — stores duration/elapsed as `u64` nanoseconds, `finished` flag, and `TimerModeData`
- `TimerModeData` — enum: `Once` (default) | `Repeating`
- `TimerDataError` — validation error (`ElapsedExceedsDuration`)
- `FnvHasher` — custom FNV-1a hasher for no_std-compatible hashing
- `HashHex` — newtype for hex-formatted hash display

**Feature-gated sections:**
- `#[cfg(feature = "bevy")]` — `From` impls between Bevy types and data types
- `#[cfg(all(feature = "bevy", feature = "serde"))]` — `serialize()`/`deserialize()` fns for `#[serde(with)]` usage

**Design notes:**
- `no_std` compatible (uses `core::` and `alloc` only in tests)
- All time values stored as `u64` nanoseconds internally
- Validation: `Once` mode rejects elapsed > duration; `Repeating` allows overflow

## License

Dual-licensed: MIT OR Apache-2.0.
