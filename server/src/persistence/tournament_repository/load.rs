use std::time::SystemTime;

use sqlx::query_as::query_as;
use uuid::Uuid;

use crate::application::{
    ActiveRound, CompletedRound, PairingPreviewSnapshot, TournamentApplication,
    TournamentApplicationSnapshot, TournamentEntrant,
};
use crate::identity::{ClubId, EntrantId, MatchId};
use crate::pairing::EloRating;
use crate::pairing::algorithms::PairingSnapshot;
use crate::pairing::algorithms::blossom_v1::{PairingProposal, RoundNumber};
use crate::results::{
    GamePoints, GameScore, MatchFormat, MatchResult, MatchResultRevision, RoundActivity,
    ScheduledMatch, restore_match_result,
};
use crate::scheduling::TableNumber;
use crate::tournament::{MaximumRoundCount, TableCount, Tournament, TournamentId};

use super::row::{EntrantRow, GameRow, MatchRow, ResultRow, RoundRow, TournamentRow};
use super::{StoredTournament, TournamentRepository, TournamentRepositoryError};

impl TournamentRepository {
    pub async fn load(
        &self,
        tournament_id: Uuid,
    ) -> Result<Option<StoredTournament>, TournamentRepositoryError> {
        let Some(row) = query_as::<sqlx_postgres::Postgres, TournamentRow>(
            "SELECT id, domain_id, status, match_format,
                    table_count, maximum_round_count, revision
             FROM tournaments WHERE id = $1",
        )
        .bind(tournament_id)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let match_format = parse_match_format(&row.match_format)?;
        let mut tournament = Tournament::new(
            TournamentId::new(row.domain_id),
            match_format,
            TableCount::try_from(i64::from(row.table_count)).map_err(invalid)?,
            MaximumRoundCount::try_from(i64::from(row.maximum_round_count)).map_err(invalid)?,
        );
        if row.status == "started" {
            tournament.start().map_err(invalid)?;
        } else if row.status != "draft" {
            return Err(invalid("unknown tournament status"));
        }

        let (entrants, active_entrant_ids) = self.load_entrants(row.id).await?;
        let round_rows = query_as::<sqlx_postgres::Postgres, RoundRow>(
            "SELECT id, round_number, status, pairing_snapshot,
                    pairing_proposal, bye_entrant_id
             FROM rounds
             WHERE tournament_id = $1
             ORDER BY round_number",
        )
        .bind(row.id)
        .fetch_all(&self.pool)
        .await?;
        let mut completed_rounds = Vec::new();
        let mut active_round = None;
        let mut pending_pairing = None;
        for round_row in round_rows {
            let request: PairingSnapshot = serde_json::from_value(round_row.pairing_snapshot)?;
            let proposal: PairingProposal = serde_json::from_value(round_row.pairing_proposal)?;
            let round_number =
                RoundNumber::try_from(i64::from(round_row.round_number)).map_err(invalid)?;
            let bye = round_row.bye_entrant_id.map(EntrantId::new);
            match round_row.status.as_str() {
                "preview" => {
                    if pending_pairing
                        .replace(PairingPreviewSnapshot { request, proposal })
                        .is_some()
                    {
                        return Err(invalid("multiple pairing previews"));
                    }
                }
                "active" => {
                    let (scheduled_matches, results) =
                        self.load_matches(round_row.id, match_format).await?;
                    if active_round
                        .replace(ActiveRound {
                            round_number,
                            pairing_request: request,
                            proposal,
                            scheduled_matches,
                            results,
                            bye,
                        })
                        .is_some()
                    {
                        return Err(invalid("multiple active rounds"));
                    }
                }
                "completed" => {
                    let (scheduled_matches, results) =
                        self.load_matches(round_row.id, match_format).await?;
                    completed_rounds.push(CompletedRound {
                        round_number,
                        pairing_request: request,
                        proposal,
                        scheduled_matches,
                        results,
                        bye,
                    });
                }
                _ => return Err(invalid("unknown round status")),
            }
        }
        let application = TournamentApplication::restore(TournamentApplicationSnapshot {
            schema_version: 1,
            tournament,
            entrants,
            completed_rounds,
            active_round,
            pending_pairing,
            active_entrant_ids,
        })?;
        let revision = u64::try_from(row.revision).map_err(invalid)?;
        Ok(Some(StoredTournament {
            id: row.id,
            revision,
            application,
        }))
    }

    async fn load_entrants(
        &self,
        tournament_id: Uuid,
    ) -> Result<(Vec<TournamentEntrant>, Vec<EntrantId>), TournamentRepositoryError> {
        let rows = query_as::<sqlx_postgres::Postgres, EntrantRow>(
            "SELECT entrant_id, display_name, club_id, club_name,
                    starting_elo, is_active
             FROM entrants
             WHERE tournament_id = $1
             ORDER BY entrant_id",
        )
        .bind(tournament_id)
        .fetch_all(&self.pool)
        .await?;
        let mut entrants = Vec::with_capacity(rows.len());
        let mut active = Vec::new();
        for row in rows {
            let entrant_id = EntrantId::new(row.entrant_id);
            if row.is_active {
                active.push(entrant_id.clone());
            }
            entrants.push(TournamentEntrant {
                entrant_id,
                name: row.display_name,
                club_id: ClubId::new(row.club_id),
                club_name: row.club_name,
                starting_elo: EloRating::new(u32::try_from(row.starting_elo).map_err(invalid)?),
            });
        }
        Ok((entrants, active))
    }

    async fn load_matches(
        &self,
        round_id: Uuid,
        match_format: MatchFormat,
    ) -> Result<(Vec<ScheduledMatch>, Vec<MatchResult>), TournamentRepositoryError> {
        let rows = query_as::<sqlx_postgres::Postgres, MatchRow>(
            "SELECT id, match_id, home_entrant_id, away_entrant_id, table_number,
                    publication_status, round_activity, winner_entrant_id,
                    home_games_won, away_games_won, revision
             FROM matches WHERE round_id = $1 ORDER BY match_id",
        )
        .bind(round_id)
        .fetch_all(&self.pool)
        .await?;
        let mut scheduled_matches = Vec::with_capacity(rows.len());
        let mut results = Vec::new();
        for row in rows {
            let table_number = row
                .table_number
                .map(|value| TableNumber::try_from(i64::from(value)).map_err(invalid))
                .transpose()?;
            let scheduled = ScheduledMatch::published(
                MatchId::new(row.match_id.clone()),
                EntrantId::new(row.home_entrant_id.clone()),
                EntrantId::new(row.away_entrant_id.clone()),
                table_number,
                parse_round_activity(&row.round_activity)?,
            );
            if row.publication_status != "published" {
                return Err(invalid("non-published stored match"));
            }
            if row.revision > 0 {
                let result = self.load_result(&row, &scheduled, match_format).await?;
                results.push(result);
            } else if row.winner_entrant_id.is_some()
                || row.home_games_won.is_some()
                || row.away_games_won.is_some()
            {
                return Err(invalid("result summary without a result revision"));
            }
            scheduled_matches.push(scheduled);
        }
        Ok((scheduled_matches, results))
    }

    async fn load_result(
        &self,
        match_row: &MatchRow,
        scheduled: &ScheduledMatch,
        match_format: MatchFormat,
    ) -> Result<MatchResult, TournamentRepositoryError> {
        let result_row = query_as::<sqlx_postgres::Postgres, ResultRow>(
            "SELECT winner_entrant_id, home_games_won, away_games_won,
                    entered_at, corrected_at, correction_reason
             FROM match_result_revisions
             WHERE match_id = $1 AND revision = $2",
        )
        .bind(match_row.id)
        .bind(match_row.revision)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| invalid("match summary has no result revision"))?;
        let game_rows = query_as::<sqlx_postgres::Postgres, GameRow>(
            "SELECT game_number, home_points, away_points
             FROM game_scores
             WHERE match_id = $1 AND result_revision = $2
             ORDER BY game_number",
        )
        .bind(match_row.id)
        .bind(match_row.revision)
        .fetch_all(&self.pool)
        .await?;
        let games = game_rows
            .into_iter()
            .map(|game| {
                Ok(GameScore {
                    game_number: crate::results::GameNumber::try_from(i64::from(game.game_number))
                        .map_err(invalid)?,
                    home_points: GamePoints::try_from(i64::from(game.home_points))
                        .map_err(invalid)?,
                    away_points: GamePoints::try_from(i64::from(game.away_points))
                        .map_err(invalid)?,
                })
            })
            .collect::<Result<Vec<_>, TournamentRepositoryError>>()?;
        let active_scheduled = ScheduledMatch::published(
            scheduled.match_id.clone(),
            scheduled.home_entrant_id.clone(),
            scheduled.away_entrant_id.clone(),
            scheduled.table_number(),
            RoundActivity::Active,
        );
        let revision_value = u32::try_from(match_row.revision).map_err(invalid)?;
        let result = restore_match_result(
            &active_scheduled,
            match_format,
            games,
            SystemTime::from(result_row.entered_at),
            result_row.corrected_at.map(SystemTime::from),
            MatchResultRevision::try_from_value(revision_value).map_err(invalid)?,
            result_row.correction_reason,
        )
        .map_err(invalid)?;
        if result.winner_id().as_str() != result_row.winner_entrant_id
            || i32::from(result.home_games_won().value()) != result_row.home_games_won
            || i32::from(result.away_games_won().value()) != result_row.away_games_won
            || match_row.winner_entrant_id.as_deref() != Some(result_row.winner_entrant_id.as_str())
            || match_row.home_games_won != Some(result_row.home_games_won)
            || match_row.away_games_won != Some(result_row.away_games_won)
        {
            return Err(invalid("stored result summary does not match its games"));
        }
        Ok(result)
    }
}

fn parse_match_format(value: &str) -> Result<MatchFormat, TournamentRepositoryError> {
    match value {
        "best_of_three" => Ok(MatchFormat::BestOfThree),
        "best_of_five" => Ok(MatchFormat::BestOfFive),
        _ => Err(invalid("unknown match format")),
    }
}

fn parse_round_activity(value: &str) -> Result<RoundActivity, TournamentRepositoryError> {
    match value {
        "active" => Ok(RoundActivity::Active),
        "inactive" => Ok(RoundActivity::Inactive),
        _ => Err(invalid("unknown round activity")),
    }
}

fn invalid(error: impl std::fmt::Display) -> TournamentRepositoryError {
    TournamentRepositoryError::InvalidStoredData(error.to_string())
}
