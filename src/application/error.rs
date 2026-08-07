use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::identity::{EntrantId, MatchId};
use crate::pairing::TableAssignmentError;
use crate::pairing::algorithms::blossom_v1::BlossomPairingError;
use crate::results::MatchResultError;
use crate::tournament::TournamentError;

#[derive(Debug)]
pub enum TournamentApplicationError {
    DuplicateEntrant { entrant_id: EntrantId },
    NotEnoughEntrants { entrant_count: usize },
    TournamentNotStarted,
    ActiveRoundExists,
    NoPairingPreview,
    NoActiveRound,
    UnknownMatch { match_id: MatchId },
    MatchAwaitingTable { match_id: MatchId },
    InvalidRoundHistory { match_id: MatchId },
    UnknownEntrantInRound { entrant_id: EntrantId },
    ResultAlreadyEntered { match_id: MatchId },
    RoundIncomplete { missing_result_count: usize },
    MaximumRoundsCompleted { maximum_round_count: u16 },
    RoundNumberOverflow,
    StandingOverflow { component: &'static str },
    Tournament(TournamentError),
    Pairing(BlossomPairingError),
    TableAssignment(TableAssignmentError),
    MatchResult(MatchResultError),
}

impl Display for TournamentApplicationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateEntrant { entrant_id } => {
                write!(
                    formatter,
                    "entrant {} is already registered",
                    entrant_id.as_str()
                )
            }
            Self::NotEnoughEntrants { entrant_count } => write!(
                formatter,
                "at least two entrants are required to start, received {entrant_count}"
            ),
            Self::TournamentNotStarted => formatter.write_str("tournament has not started"),
            Self::ActiveRoundExists => formatter.write_str("a round is already active"),
            Self::NoPairingPreview => formatter.write_str("no pairing preview is available"),
            Self::NoActiveRound => formatter.write_str("there is no active round"),
            Self::UnknownMatch { match_id } => {
                write!(
                    formatter,
                    "match {} is not in the active round",
                    match_id.as_str()
                )
            }
            Self::MatchAwaitingTable { match_id } => write!(
                formatter,
                "match {} is waiting for an available table",
                match_id.as_str()
            ),
            Self::InvalidRoundHistory { match_id } => write!(
                formatter,
                "completed result {} has no corresponding scheduled match",
                match_id.as_str()
            ),
            Self::UnknownEntrantInRound { entrant_id } => write!(
                formatter,
                "round history references unknown entrant {}",
                entrant_id.as_str()
            ),
            Self::ResultAlreadyEntered { match_id } => {
                write!(
                    formatter,
                    "match {} already has a result",
                    match_id.as_str()
                )
            }
            Self::RoundIncomplete {
                missing_result_count,
            } => write!(
                formatter,
                "round cannot be completed while {missing_result_count} results are missing"
            ),
            Self::MaximumRoundsCompleted {
                maximum_round_count,
            } => write!(
                formatter,
                "all {maximum_round_count} configured rounds have been completed"
            ),
            Self::RoundNumberOverflow => formatter.write_str("round number exceeds its limit"),
            Self::StandingOverflow { component } => {
                write!(
                    formatter,
                    "standing value overflowed while calculating {component}"
                )
            }
            Self::Tournament(error) => Display::fmt(error, formatter),
            Self::Pairing(error) => Display::fmt(error, formatter),
            Self::TableAssignment(error) => Display::fmt(error, formatter),
            Self::MatchResult(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for TournamentApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Tournament(error) => Some(error),
            Self::Pairing(error) => Some(error),
            Self::TableAssignment(error) => Some(error),
            Self::MatchResult(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TournamentError> for TournamentApplicationError {
    fn from(value: TournamentError) -> Self {
        Self::Tournament(value)
    }
}

impl From<BlossomPairingError> for TournamentApplicationError {
    fn from(value: BlossomPairingError) -> Self {
        Self::Pairing(value)
    }
}

impl From<TableAssignmentError> for TournamentApplicationError {
    fn from(value: TableAssignmentError) -> Self {
        Self::TableAssignment(value)
    }
}

impl From<MatchResultError> for TournamentApplicationError {
    fn from(value: MatchResultError) -> Self {
        Self::MatchResult(value)
    }
}
