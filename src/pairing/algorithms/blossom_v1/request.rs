use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::num::NonZeroU16;

use serde::{Deserialize, Serialize};

pub use crate::identity::ClubId;
use crate::identity::EntrantId;
use crate::pairing::EloRating;

use super::BlossomV1Policy;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RoundNumber(NonZeroU16);

impl RoundNumber {
    pub const fn value(self) -> u16 {
        self.0.get()
    }
}

impl TryFrom<i64> for RoundNumber {
    type Error = RoundNumberError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = u16::try_from(value).map_err(|_| RoundNumberError)?;
        NonZeroU16::new(value).map(Self).ok_or(RoundNumberError)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoundNumberError;

impl Display for RoundNumberError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("round number must be between 1 and 65535")
    }
}

impl Error for RoundNumberError {}

/// Integer representation supplied by the application scoring layer.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct PerformanceScore(i64);

impl PerformanceScore {
    pub const ZERO: Self = Self(0);

    pub const fn from_scaled(value: i64) -> Self {
        Self(value)
    }

    pub const fn scaled_value(self) -> i64 {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PairingRequest {
    pub round_number: RoundNumber,
    pub entrants: Vec<PairingEntrant>,
    pub previous_matches: Vec<PreviousMatch>,
    pub policy: BlossomV1Policy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PairingEntrant {
    pub entrant_id: EntrantId,
    pub club_id: ClubId,
    pub starting_elo: EloRating,
    pub performance_score: PerformanceScore,
    pub matches_won: u16,
    pub opponent_score_sum: PerformanceScore,
    pub bye_count: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PreviousMatch {
    pub first_entrant_id: EntrantId,
    pub second_entrant_id: EntrantId,
    pub round_number: RoundNumber,
}
