# timer_data vision

`timer_data` is a small Rust crate that gives Bevy's `Timer` a
serializer-independent form. `Timer` does not implement the common
serialization traits, so this crate provides `TimerData` and `TimerModeData` as
plain data types, with optional conversions to and from the Bevy types and
optional serde and rkyv derives. It is used where game state containing timers
must be saved, loaded, or sent over the wire.

## Problem

A Bevy component holding a `Timer` cannot be serialized directly. Wrapping the
Bevy type in an adapter at each call site duplicates the same shape and drags
`bevy_time` into crates that only need the data. The data also needs a stable,
portable representation that does not depend on Bevy's internals.

## Principles

- Serializer-independent: the data types are the contract, not any particular
  format. Format support enters through feature flags (`serde`, `rkyv`) and the
  conversions enter separately through `bevy`.
- Plain data only. `TimerData` stores duration and elapsed as `u64` nanoseconds,
  a finished flag, and `TimerModeData` (`Once` or `Repeating`).
- Usable without Bevy at all: `--no-default-features` builds the types with no
  engine dependency.
- `no_std` compatible, using only `core`.
- Invalid states are rejected, not silently accepted: `Once` mode rejects
  elapsed greater than duration with `TimerDataError::ElapsedExceedsDuration`,
  while `Repeating` allows overflow.
- Convenience is explicit: with `bevy` and `serde` together the crate offers
  `#[serde(with = "timer_data")]` helpers, so a field can keep its Bevy type in
  memory.

## Scope

- `TimerData`, `TimerModeData`, `TimerDataError`.
- `From`/`Into` conversions for `bevy_time::Timer` and `TimerMode` behind the
  `bevy` feature.
- serde and rkyv derives behind their features.
- Bevy 0.18 compatibility for the 0.1 line.

## Non-goals

- It does not drive, tick, or schedule timers; that remains Bevy's job.
- It does not provide a serialization format of its own.

## Direction

Not yet documented. The repository has no roadmap or open plan; recent commits
are build, CI, and metadata maintenance.

## Place in the fleet

No canix consumer was found. Canix records the project only in the atlas
`simit-updater` registry.
