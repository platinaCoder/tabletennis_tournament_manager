use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchFormat {
    BestOfThree,
    BestOfFive,
}

impl MatchFormat {
    pub const fn maximum_games(self) -> usize {
        match self {
            Self::BestOfThree => 3,
            Self::BestOfFive => 5,
        }
    }

    pub const fn games_required_to_win(self) -> u8 {
        match self {
            Self::BestOfThree => 2,
            Self::BestOfFive => 3,
        }
    }
}
