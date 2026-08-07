mod format;
mod game;
mod match_result;

pub use crate::identity::{EntrantId, MatchId};
pub use crate::scheduling::{MatchPublicationStatus, RoundActivity, ScheduledMatch};
pub use format::MatchFormat;
pub use game::{GameNumber, GameNumberError, GamePoints, GamePointsError, GameScore, MatchSide};
pub use match_result::{
    GamesWon, MatchProgress, MatchProgressStatus, MatchResult, MatchResultError,
    MatchResultRevision, evaluate_match_progress, validate_and_complete_match,
};

#[cfg(test)]
mod game_tests;
#[cfg(test)]
mod match_result_tests;
