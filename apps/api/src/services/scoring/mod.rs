//! Pure scoring functions for the CNGOKZAPI Points & Ranking System.
//!
//! There is no database, cache, HTTP or task logic in this module. Every
//! function is deterministic and takes explicit inputs so the algorithm can be
//! unit tested and reused by incremental and full recalculation.
//!
//! A Filter is `(map, course, mode, NUB/PRO)`. One player has one PB score per
//! Filter. T9 is only a historical first-completion label; its points use the
//! T8 base and distribution tier.

pub mod formula;
pub mod model;
pub mod recalc;

pub use formula::{
    ScoringError, ScoringInput, ScoringOutput, base_points, calculate_points, distribution_points,
    rank_factor,
};
pub use model::{RecordType, ScoringMode, Tier};
