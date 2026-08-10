use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::platform_time::system_time_now;

use super::match_correction::validate_restored_audit_metadata;

use super::{
    EntrantId, GameScore, MatchFormat, MatchId, MatchPublicationStatus, MatchSide, RoundActivity,
    ScheduledMatch,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GamesWon(u8);

impl GamesWon {
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchProgressStatus {
    Incomplete,
    Complete { winner: MatchSide },
}

/// The domain-derived state of the game scores currently entered for a match.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MatchProgress {
    home_games_won: GamesWon,
    away_games_won: GamesWon,
    status: MatchProgressStatus,
}

impl MatchProgress {
    pub const fn home_games_won(self) -> GamesWon {
        self.home_games_won
    }

    pub const fn away_games_won(self) -> GamesWon {
        self.away_games_won
    }

    pub const fn status(self) -> MatchProgressStatus {
        self.status
    }

    pub const fn winner(self) -> Option<MatchSide> {
        match self.status {
            MatchProgressStatus::Incomplete => None,
            MatchProgressStatus::Complete { winner } => Some(winner),
        }
    }

    pub const fn is_complete(self) -> bool {
        matches!(self.status, MatchProgressStatus::Complete { .. })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MatchResultRevision(u32);

impl MatchResultRevision {
    pub const fn value(self) -> u32 {
        self.0
    }

    pub fn try_from_value(value: u32) -> Result<Self, MatchResultRevisionError> {
        if value == 0 {
            Err(MatchResultRevisionError)
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MatchResultRevisionError;

impl Display for MatchResultRevisionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("match result revision must be greater than zero")
    }
}

impl Error for MatchResultRevisionError {}

/// A completed match whose summary fields are derived from its games.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MatchResult {
    pub(super) match_id: MatchId,
    pub(super) games: Vec<GameScore>,
    pub(super) home_games_won: GamesWon,
    pub(super) away_games_won: GamesWon,
    pub(super) winner_id: EntrantId,
    pub(super) entered_at: SystemTime,
    pub(super) corrected_at: Option<SystemTime>,
    pub(super) revision: MatchResultRevision,
    #[serde(default)]
    pub(super) correction_reason: Option<String>,
}

impl MatchResult {
    pub fn match_id(&self) -> &MatchId {
        &self.match_id
    }

    pub fn games(&self) -> &[GameScore] {
        &self.games
    }

    pub const fn home_games_won(&self) -> GamesWon {
        self.home_games_won
    }

    pub const fn away_games_won(&self) -> GamesWon {
        self.away_games_won
    }

    pub fn winner_id(&self) -> &EntrantId {
        &self.winner_id
    }

    pub const fn entered_at(&self) -> SystemTime {
        self.entered_at
    }

    pub const fn corrected_at(&self) -> Option<SystemTime> {
        self.corrected_at
    }

    pub const fn revision(&self) -> MatchResultRevision {
        self.revision
    }

    pub fn correction_reason(&self) -> Option<&str> {
        self.correction_reason.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MatchResultError {
    InvalidGameScore {
        game_number: u8,
        home_points: u16,
        away_points: u16,
    },
    GameNumbersNotSequential {
        expected: u8,
        actual: u8,
    },
    MatchNotComplete {
        home_games_won: u8,
        away_games_won: u8,
    },
    TooManyGames {
        maximum: usize,
        submitted: usize,
    },
    GamesRecordedAfterMatchCompletion {
        winning_game_number: u8,
    },
    MatchNotPublished,
    RoundNotActive,
    ResultDoesNotBelongToMatch,
    CorrectionReasonTooLong {
        maximum: usize,
    },
    UnexpectedCorrectionReason,
    CorrectionTimestampRequired,
    MatchResultRevisionOverflow,
}

impl MatchResultError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidGameScore { .. } => "invalid_game_score",
            Self::GameNumbersNotSequential { .. } => "game_numbers_not_sequential",
            Self::MatchNotComplete { .. } => "match_not_complete",
            Self::TooManyGames { .. } => "too_many_games",
            Self::GamesRecordedAfterMatchCompletion { .. } => {
                "games_recorded_after_match_completion"
            }
            Self::MatchNotPublished => "match_not_published",
            Self::RoundNotActive => "round_not_active",
            Self::ResultDoesNotBelongToMatch => "result_does_not_belong_to_match",
            Self::CorrectionReasonTooLong { .. } => "correction_reason_too_long",
            Self::UnexpectedCorrectionReason => "unexpected_correction_reason",
            Self::CorrectionTimestampRequired => "correction_timestamp_required",
            Self::MatchResultRevisionOverflow => "match_result_revision_overflow",
        }
    }
}

impl Display for MatchResultError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGameScore {
                game_number,
                home_points,
                away_points,
            } => write!(
                formatter,
                "game {game_number} has an invalid score: {home_points}-{away_points}"
            ),
            Self::GameNumbersNotSequential { expected, actual } => write!(
                formatter,
                "game numbers must be sequential: expected {expected}, got {actual}"
            ),
            Self::MatchNotComplete {
                home_games_won,
                away_games_won,
            } => write!(
                formatter,
                "match is incomplete at {home_games_won}-{away_games_won}"
            ),
            Self::TooManyGames { maximum, submitted } => write!(
                formatter,
                "match allows at most {maximum} games, but {submitted} were submitted"
            ),
            Self::GamesRecordedAfterMatchCompletion {
                winning_game_number,
            } => write!(
                formatter,
                "games were recorded after the match ended at game {winning_game_number}"
            ),
            Self::MatchNotPublished => formatter.write_str("match is not published"),
            Self::RoundNotActive => formatter.write_str("match is not in the active round"),
            Self::ResultDoesNotBelongToMatch => {
                formatter.write_str("the existing result belongs to another match")
            }
            Self::CorrectionReasonTooLong { maximum } => {
                write!(
                    formatter,
                    "correction reason may contain at most {maximum} bytes"
                )
            }
            Self::UnexpectedCorrectionReason => {
                formatter.write_str("an initial result cannot contain a correction reason")
            }
            Self::CorrectionTimestampRequired => {
                formatter.write_str("a corrected result requires a correction timestamp")
            }
            Self::MatchResultRevisionOverflow => {
                formatter.write_str("match result revision exceeds its limit")
            }
        }
    }
}

impl Error for MatchResultError {}

/// Validates entered game rows and derives current match progress.
pub fn evaluate_match_progress(
    match_format: MatchFormat,
    submitted_games: &[GameScore],
) -> Result<MatchProgress, MatchResultError> {
    validate_game_count(match_format, submitted_games)?;

    let required_wins = match_format.games_required_to_win();
    let mut home_games_won = 0_u8;
    let mut away_games_won = 0_u8;
    let mut winning_game_number = None;

    for (index, game) in submitted_games.iter().copied().enumerate() {
        validate_game_number(index, game)?;

        if let Some(winning_game_number) = winning_game_number {
            return Err(MatchResultError::GamesRecordedAfterMatchCompletion {
                winning_game_number,
            });
        }

        match game.winner() {
            Some(MatchSide::Home) => home_games_won += 1,
            Some(MatchSide::Away) => away_games_won += 1,
            None => return Err(invalid_game_score(game)),
        }

        if home_games_won == required_wins || away_games_won == required_wins {
            winning_game_number = Some(game.game_number.value());
        }
    }

    Ok(MatchProgress {
        home_games_won: GamesWon(home_games_won),
        away_games_won: GamesWon(away_games_won),
        status: progress_status(home_games_won, away_games_won, required_wins),
    })
}

fn validate_game_count(
    match_format: MatchFormat,
    submitted_games: &[GameScore],
) -> Result<(), MatchResultError> {
    if submitted_games.len() > match_format.maximum_games() {
        return Err(MatchResultError::TooManyGames {
            maximum: match_format.maximum_games(),
            submitted: submitted_games.len(),
        });
    }

    Ok(())
}

fn validate_game_number(index: usize, game: GameScore) -> Result<(), MatchResultError> {
    let expected =
        u8::try_from(index + 1).expect("supported match formats always fit in a u8 game number");
    let actual = game.game_number.value();

    if actual != expected {
        return Err(MatchResultError::GameNumbersNotSequential { expected, actual });
    }

    Ok(())
}

fn invalid_game_score(game: GameScore) -> MatchResultError {
    MatchResultError::InvalidGameScore {
        game_number: game.game_number.value(),
        home_points: game.home_points.value(),
        away_points: game.away_points.value(),
    }
}

fn progress_status(home: u8, away: u8, required: u8) -> MatchProgressStatus {
    match (home == required, away == required) {
        (true, false) => MatchProgressStatus::Complete {
            winner: MatchSide::Home,
        },
        (false, true) => MatchProgressStatus::Complete {
            winner: MatchSide::Away,
        },
        _ => MatchProgressStatus::Incomplete,
    }
}

/// Validates individual games and derives the complete match outcome.
pub fn validate_and_complete_match(
    scheduled_match: &ScheduledMatch,
    match_format: MatchFormat,
    submitted_games: Vec<GameScore>,
) -> Result<MatchResult, MatchResultError> {
    validate_and_complete_match_at(
        scheduled_match,
        match_format,
        submitted_games,
        system_time_now(),
    )
}

/// Reconstructs a persisted result while re-running all game and match rules.
/// Summary fields are always derived from `games`.
pub fn restore_match_result(
    scheduled_match: &ScheduledMatch,
    match_format: MatchFormat,
    games: Vec<GameScore>,
    entered_at: SystemTime,
    corrected_at: Option<SystemTime>,
    revision: MatchResultRevision,
    correction_reason: Option<String>,
) -> Result<MatchResult, MatchResultError> {
    let mut result =
        validate_and_complete_match_at(scheduled_match, match_format, games, entered_at)?;
    validate_restored_audit_metadata(revision, corrected_at, correction_reason.as_deref())?;
    result.corrected_at = corrected_at;
    result.revision = revision;
    result.correction_reason = correction_reason;
    Ok(result)
}

pub(super) fn validate_and_complete_match_at(
    scheduled_match: &ScheduledMatch,
    match_format: MatchFormat,
    submitted_games: Vec<GameScore>,
    entered_at: SystemTime,
) -> Result<MatchResult, MatchResultError> {
    validate_scheduled_match(scheduled_match)?;
    let progress = evaluate_match_progress(match_format, &submitted_games)?;
    let winner_id = completed_winner_id(scheduled_match, progress)?;

    Ok(MatchResult {
        match_id: scheduled_match.match_id.clone(),
        games: submitted_games,
        home_games_won: progress.home_games_won(),
        away_games_won: progress.away_games_won(),
        winner_id,
        entered_at,
        corrected_at: None,
        revision: MatchResultRevision(1),
        correction_reason: None,
    })
}

fn validate_scheduled_match(scheduled_match: &ScheduledMatch) -> Result<(), MatchResultError> {
    validate_published_match(scheduled_match)?;

    if scheduled_match.round_activity != RoundActivity::Active {
        return Err(MatchResultError::RoundNotActive);
    }

    Ok(())
}

pub(super) fn validate_published_match(
    scheduled_match: &ScheduledMatch,
) -> Result<(), MatchResultError> {
    if scheduled_match.publication_status == MatchPublicationStatus::Published {
        Ok(())
    } else {
        Err(MatchResultError::MatchNotPublished)
    }
}

pub(super) fn completed_winner_id(
    scheduled_match: &ScheduledMatch,
    progress: MatchProgress,
) -> Result<EntrantId, MatchResultError> {
    match progress.winner() {
        Some(MatchSide::Home) => Ok(scheduled_match.home_entrant_id.clone()),
        Some(MatchSide::Away) => Ok(scheduled_match.away_entrant_id.clone()),
        None => Err(MatchResultError::MatchNotComplete {
            home_games_won: progress.home_games_won().value(),
            away_games_won: progress.away_games_won().value(),
        }),
    }
}
