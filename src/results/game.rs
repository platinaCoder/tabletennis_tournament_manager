use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// A checked point total suitable for storage in an unsigned 16-bit column.
///
/// Table-tennis rules do not add an artificial score cap. The upper bound here
/// is solely the storage/input-safety limit.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GamePoints(u16);

impl GamePoints {
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl From<u16> for GamePoints {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl TryFrom<i64> for GamePoints {
    type Error = GamePointsError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = u16::try_from(value).map_err(|_| {
            if value < 0 {
                GamePointsError::Negative
            } else {
                GamePointsError::ExceedsStorageLimit
            }
        })?;

        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GamePointsError {
    Negative,
    ExceedsStorageLimit,
}

impl Display for GamePointsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negative => formatter.write_str("game points cannot be negative"),
            Self::ExceedsStorageLimit => {
                formatter.write_str("game points exceed the supported storage limit")
            }
        }
    }
}

impl Error for GamePointsError {}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GameNumber(u8);

impl GameNumber {
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl TryFrom<i64> for GameNumber {
    type Error = GameNumberError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match u8::try_from(value) {
            Ok(0) => Err(GameNumberError),
            Ok(value) => Ok(Self(value)),
            Err(_) => Err(GameNumberError),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameNumberError;

impl Display for GameNumberError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("game number must be between 1 and 255")
    }
}

impl Error for GameNumberError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameScore {
    pub game_number: GameNumber,
    pub home_points: GamePoints,
    pub away_points: GamePoints,
}

impl GameScore {
    pub fn new(
        game_number: u8,
        home_points: u16,
        away_points: u16,
    ) -> Result<Self, GameNumberError> {
        Ok(Self {
            game_number: GameNumber::try_from(i64::from(game_number))?,
            home_points: GamePoints(home_points),
            away_points: GamePoints(away_points),
        })
    }

    pub(super) fn winner(self) -> Option<MatchSide> {
        let home_wins = is_winning_score(self.home_points, self.away_points);
        let away_wins = is_winning_score(self.away_points, self.home_points);

        match (home_wins, away_wins) {
            (true, false) => Some(MatchSide::Home),
            (false, true) => Some(MatchSide::Away),
            _ => None,
        }
    }
}

fn is_winning_score(candidate: GamePoints, opponent: GamePoints) -> bool {
    candidate.value() >= 11
        && candidate
            .value()
            .checked_sub(opponent.value())
            .is_some_and(|lead| lead >= 2)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchSide {
    Home,
    Away,
}
