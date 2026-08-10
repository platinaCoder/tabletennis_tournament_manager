use std::time::SystemTime;

use crate::platform_time::system_time_now;

use super::match_result::{completed_winner_id, validate_published_match};
use super::{
    GameScore, MatchFormat, MatchResult, MatchResultError, MatchResultRevision, ScheduledMatch,
    evaluate_match_progress,
};

const MAX_CORRECTION_REASON_LENGTH: usize = 500;

/// Revalidates replacement games and creates the next immutable result revision.
pub fn validate_and_correct_match(
    scheduled_match: &ScheduledMatch,
    match_format: MatchFormat,
    existing_result: &MatchResult,
    submitted_games: Vec<GameScore>,
    correction_reason: Option<String>,
) -> Result<MatchResult, MatchResultError> {
    validate_and_correct_match_at(
        scheduled_match,
        match_format,
        existing_result,
        submitted_games,
        correction_reason,
        system_time_now(),
    )
}

pub(super) fn validate_and_correct_match_at(
    scheduled_match: &ScheduledMatch,
    match_format: MatchFormat,
    existing_result: &MatchResult,
    submitted_games: Vec<GameScore>,
    correction_reason: Option<String>,
    corrected_at: SystemTime,
) -> Result<MatchResult, MatchResultError> {
    if existing_result.match_id() != &scheduled_match.match_id {
        return Err(MatchResultError::ResultDoesNotBelongToMatch);
    }
    validate_published_match(scheduled_match)?;
    let correction_reason = normalize_correction_reason(correction_reason)?;
    let revision = existing_result
        .revision()
        .value()
        .checked_add(1)
        .ok_or(MatchResultError::MatchResultRevisionOverflow)?;
    let progress = evaluate_match_progress(match_format, &submitted_games)?;
    let winner_id = completed_winner_id(scheduled_match, progress)?;
    Ok(MatchResult {
        match_id: scheduled_match.match_id.clone(),
        games: submitted_games,
        home_games_won: progress.home_games_won(),
        away_games_won: progress.away_games_won(),
        winner_id,
        entered_at: existing_result.entered_at(),
        corrected_at: Some(corrected_at),
        revision: MatchResultRevision::try_from_value(revision)
            .map_err(|_| MatchResultError::MatchResultRevisionOverflow)?,
        correction_reason,
    })
}

pub(super) fn validate_restored_audit_metadata(
    revision: MatchResultRevision,
    corrected_at: Option<SystemTime>,
    correction_reason: Option<&str>,
) -> Result<(), MatchResultError> {
    if revision.value() == 1 {
        if corrected_at.is_some() || correction_reason.is_some() {
            return Err(MatchResultError::UnexpectedCorrectionReason);
        }
        return Ok(());
    }
    if corrected_at.is_none() {
        return Err(MatchResultError::CorrectionTimestampRequired);
    }
    normalize_correction_reason(correction_reason.map(str::to_owned)).map(|_| ())
}

fn normalize_correction_reason(reason: Option<String>) -> Result<Option<String>, MatchResultError> {
    let Some(reason) = reason else {
        return Ok(None);
    };
    let reason = reason.trim();
    if reason.is_empty() {
        return Ok(None);
    }
    if reason.len() > MAX_CORRECTION_REASON_LENGTH {
        return Err(MatchResultError::CorrectionReasonTooLong {
            maximum: MAX_CORRECTION_REASON_LENGTH,
        });
    }
    Ok(Some(reason.to_owned()))
}
