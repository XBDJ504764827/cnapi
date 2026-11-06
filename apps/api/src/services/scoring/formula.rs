//! Deterministic points formula. All times are seconds (`f64`), converted from
//! integer nanoseconds by the caller. No intermediate rounding is performed.

use super::model::{RecordType, Tier};

const MAX_POINTS: i64 = 1000;
const TOP5_BONUS: [f64; 5] = [0.2, 0.12, 0.09, 0.06, 0.02];

/// NUB base points indexed by effective tier (1..=8).
const NUB_BASE: [f64; 8] = [0.0, 50.0, 200.0, 350.0, 500.0, 650.0, 800.0, 950.0];
/// PRO base points indexed by effective tier (1..=8).
const PRO_BASE: [f64; 8] = [100.0, 145.0, 280.0, 415.0, 550.0, 685.0, 820.0, 955.0];

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ScoringError {
    #[error("rank must be >= 1")]
    InvalidRank,
    #[error("time and wr must be finite, positive and time >= wr")]
    InvalidTimes,
    #[error("invalid tier")]
    InvalidTier,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoringInput {
    pub tier: Tier,
    pub record_type: RecordType,
    pub rank: u32,
    /// PB time in seconds.
    pub time: f64,
    /// Current Filter WR time in seconds.
    pub wr: f64,
    /// Effective PB count in the Filter; kept for statistics / future versions.
    /// The first-version distribution formula deliberately does not use it.
    pub pb_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoringOutput {
    pub base: f64,
    pub rank_factor: f64,
    pub dist_points: f64,
    pub points: i64,
}

/// Base points for a tier and record type. T9 == T8.
pub fn base_points(tier: Tier, record_type: RecordType) -> f64 {
    let index = (tier.effective() - 1) as usize;
    match record_type {
        RecordType::Nub => NUB_BASE[index],
        RecordType::Pro => PRO_BASE[index],
    }
}

/// Rank factor for 1-based rank. Rank 1 == 1.0, rank >= 101 == 0.0.
pub fn rank_factor(rank: u32) -> f64 {
    let r = rank.saturating_sub(1);
    let mut factor = 0.0;
    if r < 100 {
        factor += (100 - r) as f64 * 0.004;
    }
    if r < 20 {
        factor += (20 - r) as f64 * 0.02;
    }
    if r <= 4 {
        factor += TOP5_BONUS[r as usize];
    }
    factor.min(1.0)
}

/// Small-leaderboard distribution coefficient from the phase-6 specification.
pub fn distribution_points(tier: Tier, time: f64, wr: f64) -> f64 {
    let x = 2.1 - 0.25 * tier.effective() as f64;
    let y = 1.0 + (-0.5 * x).exp();
    let z = 1.0 + (x * (time / wr - 1.5)).exp();
    (y / z).clamp(0.0, 1.0)
}

/// Final integer points for one player PB in one Filter.
pub fn calculate_points(input: ScoringInput) -> Result<ScoringOutput, ScoringError> {
    if input.rank == 0 {
        return Err(ScoringError::InvalidRank);
    }
    if !input.time.is_finite()
        || !input.wr.is_finite()
        || input.time <= 0.0
        || input.wr <= 0.0
        || input.time < input.wr
    {
        return Err(ScoringError::InvalidTimes);
    }

    let base = base_points(input.tier, input.record_type);
    let rank_factor_value = rank_factor(input.rank);
    // WR is an exact invariant: rank=1 and time==wr must always be 1000,
    // regardless of floating rounding in the distribution formula.
    let dist =
        if input.rank == 1 && (input.time - input.wr).abs() < f64::EPSILON * input.wr.max(1.0) {
            1.0
        } else {
            distribution_points(input.tier, input.time, input.wr)
        };

    let remaining = MAX_POINTS as f64 - base;
    let points = (base + 0.125 * remaining * rank_factor_value + 0.875 * remaining * dist)
        .round()
        .clamp(0.0, MAX_POINTS as f64) as i64;

    Ok(ScoringOutput {
        base,
        rank_factor: rank_factor_value,
        dist_points: dist,
        points,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tier(n: u8) -> Tier {
        Tier::new(n).expect("valid tier")
    }

    #[test]
    fn base_table_matches_specification() {
        let expected_nub = [0.0, 50.0, 200.0, 350.0, 500.0, 650.0, 800.0, 950.0];
        let expected_pro = [100.0, 145.0, 280.0, 415.0, 550.0, 685.0, 820.0, 955.0];
        for (index, value) in expected_nub.iter().enumerate() {
            assert_eq!(base_points(tier(index as u8 + 1), RecordType::Nub), *value);
        }
        for (index, value) in expected_pro.iter().enumerate() {
            assert_eq!(base_points(tier(index as u8 + 1), RecordType::Pro), *value);
        }
        assert_eq!(base_points(tier(9), RecordType::Nub), 950.0);
        assert_eq!(base_points(tier(9), RecordType::Pro), 955.0);
        assert!(Tier::new(0).is_err());
        assert!(Tier::new(10).is_err());
    }

    #[test]
    fn rank_factor_boundaries() {
        assert_eq!(rank_factor(1), 1.0);
        assert!((rank_factor(2) - (0.396 + 0.38 + 0.12)).abs() < 1e-12);
        assert!((rank_factor(5) - (0.384 + 0.32 + 0.02)).abs() < 1e-12);
        assert!((rank_factor(20) - (0.324 + 0.02)).abs() < 1e-12);
        assert!((rank_factor(21) - 0.32).abs() < 1e-12);
        assert!((rank_factor(100) - 0.004).abs() < 1e-12);
        assert_eq!(rank_factor(101), 0.0);
        assert_eq!(rank_factor(500), 0.0);
        assert_eq!(rank_factor(u32::MAX), 0.0);
    }

    #[test]
    fn distribution_points_are_clamped_and_monotonic() {
        for tier_number in 1..=9 {
            let t = tier(tier_number);
            assert_eq!(distribution_points(t, 10.0, 10.0), 1.0);
            assert!((0.0..=1.0).contains(&distribution_points(t, 12.0, 10.0)));
            assert!(distribution_points(t, 10.0, 10.0) >= distribution_points(t, 12.0, 10.0));
            assert!(distribution_points(t, 12.0, 10.0) >= distribution_points(t, 100.0, 10.0));
            assert!((0.0..=1.0).contains(&distribution_points(t, 1e9, 10.0)));
        }
    }

    #[test]
    fn distribution_uses_t8_for_t9() {
        assert_eq!(
            distribution_points(tier(9), 12.0, 10.0),
            distribution_points(tier(8), 12.0, 10.0)
        );
    }

    #[test]
    fn wr_is_always_1000_regardless_of_tier_or_type() {
        for tier_number in 1..=9 {
            for record_type in [RecordType::Nub, RecordType::Pro] {
                let output = calculate_points(ScoringInput {
                    tier: tier(tier_number),
                    record_type,
                    rank: 1,
                    time: 42.5,
                    wr: 42.5,
                    pb_count: 1,
                })
                .expect("valid input");
                assert_eq!(output.points, 1000, "tier={tier_number} type={record_type}");
                assert_eq!(output.rank_factor, 1.0);
                assert_eq!(output.dist_points, 1.0);
            }
        }
    }

    #[test]
    fn representative_scores_are_stable_and_bounded() {
        for tier_number in 1..=9 {
            for record_type in [RecordType::Nub, RecordType::Pro] {
                for rank in [1, 2, 5, 20, 21, 100, 101, 500] {
                    let output = calculate_points(ScoringInput {
                        tier: tier(tier_number),
                        record_type,
                        rank,
                        time: 12.0,
                        wr: 10.0,
                        pb_count: 200,
                    })
                    .expect("valid input");
                    assert!((0..=1000).contains(&output.points));
                }
            }
        }
    }

    #[test]
    fn pb_count_does_not_change_first_version_points() {
        let a = calculate_points(ScoringInput {
            tier: tier(5),
            record_type: RecordType::Pro,
            rank: 3,
            time: 12.0,
            wr: 10.0,
            pb_count: 1,
        })
        .unwrap();
        let b = calculate_points(ScoringInput {
            tier: tier(5),
            record_type: RecordType::Pro,
            rank: 3,
            time: 12.0,
            wr: 10.0,
            pb_count: 1000,
        })
        .unwrap();
        assert_eq!(a.points, b.points);
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        let base = ScoringInput {
            tier: tier(5),
            record_type: RecordType::Pro,
            rank: 2,
            time: 12.0,
            wr: 10.0,
            pb_count: 1,
        };
        assert_eq!(
            calculate_points(ScoringInput { rank: 0, ..base }),
            Err(ScoringError::InvalidRank)
        );
        assert_eq!(
            calculate_points(ScoringInput { time: 9.0, ..base }),
            Err(ScoringError::InvalidTimes)
        );
        assert_eq!(
            calculate_points(ScoringInput { wr: 0.0, ..base }),
            Err(ScoringError::InvalidTimes)
        );
        assert_eq!(
            calculate_points(ScoringInput {
                time: f64::NAN,
                ..base
            }),
            Err(ScoringError::InvalidTimes)
        );
        assert_eq!(
            calculate_points(ScoringInput {
                wr: f64::INFINITY,
                ..base
            }),
            Err(ScoringError::InvalidTimes)
        );
    }
}
