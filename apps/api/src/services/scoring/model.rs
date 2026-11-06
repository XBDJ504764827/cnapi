//! Domain enums for scoring. These are independent of API DTOs and SQL rows.

use std::fmt;

/// The three independent scoring modes. `classic` is not scored in phase 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoringMode {
    Kzt,
    Skz,
    Vnl,
}

impl ScoringMode {
    pub const ALL: [Self; 3] = [Self::Kzt, Self::Skz, Self::Vnl];

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "KZT" => Some(Self::Kzt),
            "SKZ" => Some(Self::Skz),
            "VNL" => Some(Self::Vnl),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Kzt => "KZT",
            Self::Skz => "SKZ",
            Self::Vnl => "VNL",
        }
    }

    /// Database `movement_modes.code`. Configured in `scoring_modes`.
    pub const fn mode_code(self) -> &'static str {
        match self {
            Self::Kzt => "kztimer",
            Self::Skz => "simple",
            Self::Vnl => "vanilla",
        }
    }

    pub const fn supports_t9(self) -> bool {
        matches!(self, Self::Vnl)
    }
}

impl fmt::Display for ScoringMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// NUB = teleports > 0, PRO = teleports = 0 (enforced by a DB CHECK).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordType {
    Nub,
    Pro,
}

impl RecordType {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "nub" => Some(Self::Nub),
            "pro" => Some(Self::Pro),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nub => "nub",
            Self::Pro => "pro",
        }
    }
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// T1-T8 are real tiers. T9 is a historical first-completion label with T8
/// points; it is never stored as a course tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tier(u8);

impl Tier {
    pub const MAX_POINTS: i64 = 1000;

    pub fn new(value: u8) -> Result<Self, ScoringTierError> {
        if (1..=9).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ScoringTierError(value))
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    /// T9 uses T8 for base and distribution calculation.
    pub const fn effective(self) -> u8 {
        if self.0 == 9 { 8 } else { self.0 }
    }

    pub const fn is_t9(self) -> bool {
        self.0 == 9
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid tier {0}; expected 1..=9")]
pub struct ScoringTierError(pub u8);
