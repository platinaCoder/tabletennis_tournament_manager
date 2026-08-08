use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::query::query;
use sqlx::row::Row;
use sqlx::transaction::Transaction;
use sqlx_postgres::Postgres;
use uuid::Uuid;

use crate::application::{ActiveRound, CompletedRound};
use crate::identity::EntrantId;
use crate::pairing::algorithms::PairingSnapshot;
use crate::pairing::algorithms::blossom_v1::{
    PairingPolicyVersion, PairingProposal, RelaxationTier, RoundNumber,
};
use crate::results::MatchResult;
use crate::scheduling::ScheduledMatch;

use super::match_writer::save_match;
use super::{StoredTournament, TournamentRepositoryError};

pub(super) async fn save_rounds(
    transaction: &mut Transaction<'_, Postgres>,
    stored: &StoredTournament,
    now: DateTime<Utc>,
) -> Result<(), TournamentRepositoryError> {
    let snapshot = stored.application.snapshot();
    query::<Postgres>("DELETE FROM rounds WHERE tournament_id = $1 AND status = 'preview'")
        .bind(stored.id)
        .execute(&mut **transaction)
        .await?;
    for round in stored.application.completed_rounds() {
        save_published_round(transaction, stored.id, round.into(), "completed", now).await?;
    }
    if let Some(round) = stored.application.active_round() {
        save_published_round(transaction, stored.id, round.into(), "active", now).await?;
    }
    if let Some(pending) = snapshot.pending_pairing.as_ref() {
        save_round_row(
            transaction,
            stored.id,
            RoundRowWrite {
                round_number: request_round_number(&pending.request),
                status: "preview",
                request: &pending.request,
                proposal: &pending.proposal,
                bye: pending.proposal.bye.as_ref().map(|bye| &bye.entrant_id),
            },
            now,
        )
        .await?;
    }
    Ok(())
}

struct PublishedRoundRef<'a> {
    round_number: RoundNumber,
    request: &'a PairingSnapshot,
    proposal: &'a PairingProposal,
    matches: &'a [ScheduledMatch],
    results: &'a [MatchResult],
    bye: Option<&'a EntrantId>,
}

impl<'a> From<&'a CompletedRound> for PublishedRoundRef<'a> {
    fn from(round: &'a CompletedRound) -> Self {
        Self {
            round_number: round.round_number,
            request: &round.pairing_request,
            proposal: &round.proposal,
            matches: &round.scheduled_matches,
            results: &round.results,
            bye: round.bye.as_ref(),
        }
    }
}

impl<'a> From<&'a ActiveRound> for PublishedRoundRef<'a> {
    fn from(round: &'a ActiveRound) -> Self {
        Self {
            round_number: round.round_number,
            request: &round.pairing_request,
            proposal: &round.proposal,
            matches: &round.scheduled_matches,
            results: &round.results,
            bye: round.bye.as_ref(),
        }
    }
}

async fn save_published_round(
    transaction: &mut Transaction<'_, Postgres>,
    tournament_id: Uuid,
    round: PublishedRoundRef<'_>,
    status: &str,
    now: DateTime<Utc>,
) -> Result<(), TournamentRepositoryError> {
    let round_id = save_round_row(
        transaction,
        tournament_id,
        RoundRowWrite {
            round_number: round.round_number,
            status,
            request: round.request,
            proposal: round.proposal,
            bye: round.bye,
        },
        now,
    )
    .await?;
    let results = round
        .results
        .iter()
        .map(|result| (result.match_id().as_str(), result))
        .collect::<HashMap<_, _>>();
    for scheduled in round.matches {
        save_match(
            transaction,
            tournament_id,
            round_id,
            scheduled,
            results.get(scheduled.match_id.as_str()).copied(),
            now,
        )
        .await?;
    }
    Ok(())
}

struct RoundRowWrite<'a> {
    round_number: RoundNumber,
    status: &'a str,
    request: &'a PairingSnapshot,
    proposal: &'a PairingProposal,
    bye: Option<&'a EntrantId>,
}

async fn save_round_row(
    transaction: &mut Transaction<'_, Postgres>,
    tournament_id: Uuid,
    round: RoundRowWrite<'_>,
    now: DateTime<Utc>,
) -> Result<Uuid, TournamentRepositoryError> {
    let row = query::<Postgres>(
        "INSERT INTO rounds (
            id, tournament_id, round_number, status, pairing_policy_version,
            relaxation_tier, pairing_snapshot, pairing_proposal,
            bye_entrant_id, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
         ON CONFLICT (tournament_id, round_number) DO UPDATE SET
            status = EXCLUDED.status,
            pairing_policy_version = EXCLUDED.pairing_policy_version,
            relaxation_tier = EXCLUDED.relaxation_tier,
            pairing_snapshot = EXCLUDED.pairing_snapshot,
            pairing_proposal = EXCLUDED.pairing_proposal,
            bye_entrant_id = EXCLUDED.bye_entrant_id,
            updated_at = EXCLUDED.updated_at
         RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(tournament_id)
    .bind(i32::from(round.round_number.value()))
    .bind(round.status)
    .bind(policy_version(round.proposal.policy_version))
    .bind(relaxation_tier(round.proposal.relaxation_tier))
    .bind(serde_json::to_value(round.request)?)
    .bind(serde_json::to_value(round.proposal)?)
    .bind(round.bye.map(EntrantId::as_str))
    .bind(now)
    .fetch_one(&mut **transaction)
    .await?;
    Ok(row.try_get("id")?)
}

fn request_round_number(request: &PairingSnapshot) -> RoundNumber {
    match request {
        PairingSnapshot::BlossomV1(request) => request.round_number,
        PairingSnapshot::BlossomV2(request) => request.round_number,
    }
}

const fn policy_version(value: PairingPolicyVersion) -> &'static str {
    match value {
        PairingPolicyVersion::BlossomV1 => "blossom_v1",
        PairingPolicyVersion::BlossomV2 => "blossom_v2",
    }
}

const fn relaxation_tier(value: RelaxationTier) -> &'static str {
    match value {
        RelaxationTier::Strict => "strict",
        RelaxationTier::SameClubAllowed => "same_club_allowed",
        RelaxationTier::RematchesAllowed => "rematches_allowed",
    }
}
