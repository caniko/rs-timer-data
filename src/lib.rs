//! Serializer-independent data types for Bevy's Timer.
//!
//! Bevy's `Timer` doesn't implement common serialization traits.
//! This crate provides [`TimerData`] and [`TimerModeData`] as intermediate
//! representations with optional `From`/`Into` conversions (behind the `bevy`
//! feature), plus optional serde and rkyv derives behind feature flags.
//!
//! # Features
//!
//! - **`bevy`** (default): Enables `bevy_time` dependency and `From`/`Into` conversions
//!   between `Timer`/`TimerMode` and `TimerData`/`TimerModeData`.
//! - **`serde`** (default): Enables `Serialize`/`Deserialize` derives.
//!   With `bevy`, also provides `#[serde(with = "timer_data")]` helpers.
//! - **`rkyv`**: Enables `Archive`/`Serialize`/`Deserialize` derives (rkyv 0.8).
//!
//! # Serde usage
//!
//! With the `bevy` + `serde` features, use `#[serde(with = "timer_data")]`:
//!
//! ```ignore
//! use serde::{Serialize, Deserialize};
//! use bevy_time::Timer;
//!
//! #[derive(Serialize, Deserialize)]
//! pub struct MyComponent {
//!     #[serde(with = "timer_data")]
//!     pub timer: Timer,
//! }
//! ```

#![no_std]

use core::fmt;
use core::hash::{Hash, Hasher};
use core::time::Duration;

/// Intermediate data representation of a Bevy `Timer`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TimerData {
    /// Duration in nanoseconds
    pub duration: u64,
    /// Elapsed time in nanoseconds
    pub elapsed: u64,
    /// Whether the timer has finished
    pub finished: bool,
    /// Timer mode (once or repeating)
    pub mode: TimerModeData,
}

/// Intermediate data representation of Bevy's `TimerMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum TimerModeData {
    #[default]
    Once,
    Repeating,
}

/// Error returned when [`TimerData`] contains invalid state.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimerDataError {
    /// Elapsed time exceeds duration on a non-repeating timer.
    ElapsedExceedsDuration {
        elapsed: u64,
        duration: u64,
    },
}

impl fmt::Display for TimerDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ElapsedExceedsDuration { elapsed, duration } => {
                write!(
                    f,
                    "elapsed ({elapsed} ns) exceeds duration ({duration} ns) on a Once timer"
                )
            }
        }
    }
}

impl TimerData {
    /// Create a new `TimerData`.
    pub fn new(duration_nanos: u64, elapsed_nanos: u64, finished: bool, mode: TimerModeData) -> Self {
        Self {
            duration: duration_nanos,
            elapsed: elapsed_nanos,
            finished,
            mode,
        }
    }

    /// Create a `TimerData` from seconds (like Bevy's `Timer::from_seconds`).
    pub fn from_secs(secs: f32, mode: TimerModeData) -> Self {
        Self {
            duration: (secs as f64 * 1_000_000_000.0) as u64,
            elapsed: 0,
            finished: false,
            mode,
        }
    }

    /// Create a `TimerData` from a [`Duration`].
    pub fn from_duration(duration: Duration, mode: TimerModeData) -> Self {
        Self {
            duration: duration.as_nanos() as u64,
            elapsed: 0,
            finished: false,
            mode,
        }
    }

    /// Validate that this `TimerData` is in a consistent state.
    ///
    /// Returns an error if elapsed exceeds duration on a `Once` timer.
    /// Repeating timers may have elapsed > duration (they wrap around).
    pub fn validate(&self) -> Result<(), TimerDataError> {
        if self.mode == TimerModeData::Once && self.elapsed > self.duration {
            return Err(TimerDataError::ElapsedExceedsDuration {
                elapsed: self.elapsed,
                duration: self.duration,
            });
        }
        Ok(())
    }

    /// Duration as a [`Duration`].
    pub fn duration(&self) -> Duration {
        Duration::from_nanos(self.duration)
    }

    /// Elapsed time as a [`Duration`].
    pub fn elapsed(&self) -> Duration {
        Duration::from_nanos(self.elapsed)
    }

    /// Remaining time as a [`Duration`]. Saturates at zero.
    pub fn remaining(&self) -> Duration {
        Duration::from_nanos(self.duration.saturating_sub(self.elapsed))
    }

    /// Whether the timer has finished.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Fraction of the timer elapsed, from 0.0 to 1.0.
    ///
    /// Returns 0.0 if duration is zero.
    pub fn fraction(&self) -> f32 {
        if self.duration == 0 {
            return 0.0;
        }
        (self.elapsed as f64 / self.duration as f64) as f32
    }

    /// Fraction of the timer remaining, from 1.0 to 0.0.
    ///
    /// Returns 0.0 if duration is zero.
    pub fn fraction_remaining(&self) -> f32 {
        if self.duration == 0 {
            return 0.0;
        }
        1.0 - self.fraction()
    }

    /// Compute a 64-bit hash and return it as a hex string (16 chars).
    pub fn hash_hex(&self) -> HashHex {
        let mut hasher = FnvHasher::default();
        self.hash(&mut hasher);
        HashHex(hasher.finish())
    }
}

/// A 64-bit hash displayed as lowercase hex. Returned by [`TimerData::hash_hex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HashHex(pub u64);

impl fmt::Display for HashHex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl fmt::Display for TimerData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dur_ms = self.duration as f64 / 1_000_000.0;
        let elapsed_ms = self.elapsed as f64 / 1_000_000.0;
        write!(
            f,
            "{elapsed_ms:.1}/{dur_ms:.1}ms ({}, {}) [{}]",
            self.mode,
            if self.finished { "finished" } else { "running" },
            self.hash_hex(),
        )
    }
}

/// FNV-1a 64-bit hasher (no_std compatible).
#[derive(Clone)]
struct FnvHasher(u64);

impl Default for FnvHasher {
    fn default() -> Self {
        Self(0xcbf29ce484222325)
    }
}

impl Hasher for FnvHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for TimerModeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Once => write!(f, "Once"),
            Self::Repeating => write!(f, "Repeating"),
        }
    }
}

// ============================================================================
// Bevy conversions (requires `bevy` feature)
// ============================================================================

#[cfg(feature = "bevy")]
mod bevy_impls {
    use super::*;
    use bevy_time::{Timer, TimerMode};

    impl From<&Timer> for TimerData {
        fn from(timer: &Timer) -> Self {
            Self {
                duration: timer.duration().as_nanos() as u64,
                elapsed: timer.elapsed().as_nanos() as u64,
                finished: timer.is_finished(),
                mode: timer.mode().into(),
            }
        }
    }

    impl From<Timer> for TimerData {
        fn from(timer: Timer) -> Self {
            Self::from(&timer)
        }
    }

    impl From<TimerData> for Timer {
        fn from(data: TimerData) -> Self {
            let mut timer = Timer::new(Duration::from_nanos(data.duration), data.mode.into());
            timer.tick(Duration::from_nanos(data.elapsed));
            timer
        }
    }

    impl From<TimerMode> for TimerModeData {
        fn from(mode: TimerMode) -> Self {
            match mode {
                TimerMode::Once => Self::Once,
                TimerMode::Repeating => Self::Repeating,
            }
        }
    }

    impl From<TimerModeData> for TimerMode {
        fn from(data: TimerModeData) -> Self {
            match data {
                TimerModeData::Once => Self::Once,
                TimerModeData::Repeating => Self::Repeating,
            }
        }
    }
}

// ============================================================================
// Serde helper (for #[serde(with = "...")]) — requires bevy + serde
// ============================================================================

/// Serialize a `Timer` via serde.
///
/// Use with `#[serde(with = "timer_data")]` on a `Timer` field.
#[cfg(all(feature = "bevy", feature = "serde"))]
pub fn serialize<S>(timer: &bevy_time::Timer, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serde::Serialize::serialize(&TimerData::from(timer), serializer)
}

/// Deserialize a `Timer` via serde.
#[cfg(all(feature = "bevy", feature = "serde"))]
pub fn deserialize<'de, D>(deserializer: D) -> Result<bevy_time::Timer, D::Error>
where
    D: serde::Deserializer<'de>,
{
    <TimerData as serde::Deserialize>::deserialize(deserializer).map(bevy_time::Timer::from)
}

#[cfg(test)]
extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "bevy")]
    #[test]
    fn test_timer_data_roundtrip() {
        use bevy_time::{Timer, TimerMode};

        let timer = Timer::from_seconds(5.0, TimerMode::Once);
        let data = TimerData::from(&timer);
        let restored = Timer::from(data);
        assert_eq!(timer.duration(), restored.duration());
        assert_eq!(timer.mode(), restored.mode());
    }

    #[cfg(all(feature = "bevy", feature = "serde"))]
    #[test]
    fn test_serde_with_roundtrip() {
        use bevy_time::{Timer, TimerMode};
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct TestStruct {
            #[serde(with = "crate")]
            timer: Timer,
        }

        let original = TestStruct {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: TestStruct = serde_json::from_str(&json).unwrap();

        assert_eq!(original.timer.duration(), restored.timer.duration());
        assert_eq!(original.timer.mode(), restored.timer.mode());
    }

    #[test]
    fn test_timer_data_new() {
        let data = TimerData::new(5_000_000_000, 2_500_000_000, false, TimerModeData::Once);
        assert_eq!(data.duration(), Duration::from_secs(5));
        assert_eq!(data.elapsed(), Duration::from_millis(2500));
        assert!(!data.finished);
        assert_eq!(data.mode, TimerModeData::Once);
    }

    #[test]
    fn test_from_secs() {
        let data = TimerData::from_secs(3.0, TimerModeData::Repeating);
        assert_eq!(data.duration(), Duration::from_secs(3));
        assert_eq!(data.elapsed(), Duration::ZERO);
        assert!(!data.finished);
        assert_eq!(data.mode, TimerModeData::Repeating);
    }

    #[test]
    fn test_from_duration() {
        let data = TimerData::from_duration(Duration::from_millis(500), TimerModeData::Once);
        assert_eq!(data.duration(), Duration::from_millis(500));
        assert_eq!(data.elapsed(), Duration::ZERO);
    }

    #[test]
    fn test_fraction() {
        let data = TimerData::new(4_000_000_000, 1_000_000_000, false, TimerModeData::Once);
        assert!((data.fraction() - 0.25).abs() < f32::EPSILON);
        assert!((data.fraction_remaining() - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fraction_zero_duration() {
        let data = TimerData::new(0, 0, false, TimerModeData::Once);
        assert_eq!(data.fraction(), 0.0);
        assert_eq!(data.fraction_remaining(), 0.0);
    }

    #[test]
    fn test_remaining() {
        let data = TimerData::new(5_000_000_000, 3_000_000_000, false, TimerModeData::Once);
        assert_eq!(data.remaining(), Duration::from_secs(2));
    }

    #[test]
    fn test_remaining_saturates() {
        let data = TimerData::new(1_000_000_000, 5_000_000_000, true, TimerModeData::Repeating);
        assert_eq!(data.remaining(), Duration::ZERO);
    }

    #[test]
    fn test_validate_ok() {
        let data = TimerData::new(5_000_000_000, 3_000_000_000, false, TimerModeData::Once);
        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_validate_elapsed_exceeds_duration() {
        let data = TimerData::new(1_000_000_000, 5_000_000_000, false, TimerModeData::Once);
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_validate_repeating_allows_overflow() {
        let data = TimerData::new(1_000_000_000, 5_000_000_000, false, TimerModeData::Repeating);
        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_display_contains_hash() {
        let data = TimerData::new(5_000_000_000, 2_500_000_000, false, TimerModeData::Once);
        let s = alloc::format!("{data}");
        assert!(s.contains("Once"));
        assert!(s.contains("running"));
        // Display includes a 16-char hex hash in brackets
        assert!(s.contains('['));
        assert!(s.contains(']'));
    }

    #[test]
    fn test_hash_hex_deterministic() {
        let data = TimerData::new(5_000_000_000, 2_500_000_000, false, TimerModeData::Once);
        assert_eq!(data.hash_hex(), data.hash_hex());
    }

    #[test]
    fn test_hash_hex_differs() {
        let a = TimerData::new(5_000_000_000, 0, false, TimerModeData::Once);
        let b = TimerData::new(5_000_000_000, 1, false, TimerModeData::Once);
        assert_ne!(a.hash_hex(), b.hash_hex());
    }

    #[test]
    fn test_hash_hex_format() {
        let data = TimerData::new(1, 0, false, TimerModeData::Once);
        let hex = alloc::format!("{}", data.hash_hex());
        assert_eq!(hex.len(), 16);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_timer_mode_default() {
        assert_eq!(TimerModeData::default(), TimerModeData::Once);
    }
}
