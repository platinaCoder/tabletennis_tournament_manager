use crate::pairing::EloRating;
use crate::pairing::algorithms::blossom_v1::PerformanceScore;
use crate::results::MatchSide;

const PERFORMANCE_SCALE: i64 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MatchPerformanceDelta {
    pub home: PerformanceScore,
    pub away: PerformanceScore,
}

/// V1 performance policy: actual match outcome minus pre-match Elo
/// expectation, scaled to one million integer units.
pub struct EloExpectationDeltaV1;

impl EloExpectationDeltaV1 {
    pub fn calculate(
        home_elo: EloRating,
        away_elo: EloRating,
        winner: MatchSide,
    ) -> MatchPerformanceDelta {
        let elo_difference = f64::from(away_elo.value()) - f64::from(home_elo.value());
        let expected_home = 1.0 / (1.0 + 10_f64.powf(elo_difference / 400.0));
        let expected_home_scaled = (expected_home * PERFORMANCE_SCALE as f64).round() as i64;
        let actual_home = match winner {
            MatchSide::Home => PERFORMANCE_SCALE,
            MatchSide::Away => 0,
        };
        let home = actual_home - expected_home_scaled;

        MatchPerformanceDelta {
            home: PerformanceScore::from_scaled(home),
            away: PerformanceScore::from_scaled(-home),
        }
    }
}
