use std::collections::HashSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::identity::{EntrantId, MatchId};
use crate::pairing::algorithms::PairingSnapshot;
use crate::pairing::algorithms::blossom_v1::PairingProposal;
use crate::results::{RoundActivity, validate_and_complete_match};
use crate::scheduling::ScheduledMatch;
use crate::tournament::{Tournament, TournamentState};

use super::standings::calculate_standings;
use super::tournament::PendingPairing;
use super::{ActiveRound, CompletedRound, TournamentApplication, TournamentEntrant};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PairingPreviewSnapshot {
    pub request: PairingSnapshot,
    pub proposal: PairingProposal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TournamentApplicationSnapshot {
    pub schema_version: u16,
    pub tournament: Tournament,
    pub entrants: Vec<TournamentEntrant>,
    pub completed_rounds: Vec<CompletedRound>,
    pub active_round: Option<ActiveRound>,
    pub pending_pairing: Option<PairingPreviewSnapshot>,
    pub active_entrant_ids: Vec<EntrantId>,
}

impl TournamentApplication {
    pub fn snapshot(&self) -> TournamentApplicationSnapshot {
        let mut active_entrant_ids = self.active_entrant_ids.iter().cloned().collect::<Vec<_>>();
        active_entrant_ids.sort_by(|first, second| first.as_str().cmp(second.as_str()));
        TournamentApplicationSnapshot {
            schema_version: 1,
            tournament: self.tournament.clone(),
            entrants: self.entrants.clone(),
            completed_rounds: self.completed_rounds.clone(),
            active_round: self.active_round.clone(),
            pending_pairing: self
                .pending_pairing
                .as_ref()
                .map(|pending| PairingPreviewSnapshot {
                    request: pending.request.clone(),
                    proposal: pending.proposal.clone(),
                }),
            active_entrant_ids,
        }
    }

    pub fn restore(
        snapshot: TournamentApplicationSnapshot,
    ) -> Result<Self, TournamentSnapshotError> {
        if snapshot.schema_version != 1 {
            return Err(TournamentSnapshotError::UnsupportedSchemaVersion {
                version: snapshot.schema_version,
            });
        }
        validate_entrants(&snapshot.entrants, &snapshot.active_entrant_ids)?;
        validate_lifecycle(&snapshot)?;
        validate_rounds(&snapshot)?;

        let standings = calculate_standings(&snapshot.entrants, &snapshot.completed_rounds)
            .map_err(|_| TournamentSnapshotError::InvalidStandings)?;
        Ok(Self {
            tournament: snapshot.tournament,
            entrants: snapshot.entrants,
            standings,
            completed_rounds: snapshot.completed_rounds,
            active_round: snapshot.active_round,
            pending_pairing: snapshot.pending_pairing.map(|pending| PendingPairing {
                request: pending.request,
                proposal: pending.proposal,
            }),
            active_entrant_ids: snapshot.active_entrant_ids.into_iter().collect(),
        })
    }
}

fn validate_entrants(
    entrants: &[TournamentEntrant],
    active_entrant_ids: &[EntrantId],
) -> Result<(), TournamentSnapshotError> {
    let mut known = HashSet::with_capacity(entrants.len());
    for entrant in entrants {
        if !known.insert(entrant.entrant_id.clone()) {
            return Err(TournamentSnapshotError::DuplicateEntrant {
                entrant_id: entrant.entrant_id.clone(),
            });
        }
    }
    let mut active = HashSet::with_capacity(active_entrant_ids.len());
    for entrant_id in active_entrant_ids {
        if !known.contains(entrant_id) {
            return Err(TournamentSnapshotError::UnknownActiveEntrant {
                entrant_id: entrant_id.clone(),
            });
        }
        if !active.insert(entrant_id.clone()) {
            return Err(TournamentSnapshotError::DuplicateActiveEntrant {
                entrant_id: entrant_id.clone(),
            });
        }
    }
    Ok(())
}

fn validate_lifecycle(
    snapshot: &TournamentApplicationSnapshot,
) -> Result<(), TournamentSnapshotError> {
    if snapshot.active_round.is_some() && snapshot.pending_pairing.is_some() {
        return Err(TournamentSnapshotError::PreviewAndActiveRound);
    }
    if snapshot.tournament.state() == TournamentState::Draft
        && (!snapshot.completed_rounds.is_empty()
            || snapshot.active_round.is_some()
            || snapshot.pending_pairing.is_some())
    {
        return Err(TournamentSnapshotError::DraftContainsRounds);
    }
    Ok(())
}

fn validate_rounds(
    snapshot: &TournamentApplicationSnapshot,
) -> Result<(), TournamentSnapshotError> {
    let known_entrants = snapshot
        .entrants
        .iter()
        .map(|entrant| entrant.entrant_id.clone())
        .collect::<HashSet<_>>();
    let mut known_matches = HashSet::new();

    for (index, round) in snapshot.completed_rounds.iter().enumerate() {
        let expected = u16::try_from(index + 1).map_err(|_| TournamentSnapshotError::RoundGap)?;
        if round.round_number.value() != expected {
            return Err(TournamentSnapshotError::RoundGap);
        }
        validate_round(
            &round.scheduled_matches,
            &round.results,
            &known_entrants,
            &mut known_matches,
            snapshot.tournament.match_format(),
            true,
        )?;
    }
    if let Some(round) = &snapshot.active_round {
        let expected = u16::try_from(snapshot.completed_rounds.len() + 1)
            .map_err(|_| TournamentSnapshotError::RoundGap)?;
        if round.round_number.value() != expected {
            return Err(TournamentSnapshotError::RoundGap);
        }
        validate_round(
            &round.scheduled_matches,
            &round.results,
            &known_entrants,
            &mut known_matches,
            snapshot.tournament.match_format(),
            false,
        )?;
    }
    Ok(())
}

fn validate_round(
    scheduled_matches: &[ScheduledMatch],
    results: &[crate::results::MatchResult],
    known_entrants: &HashSet<EntrantId>,
    known_matches: &mut HashSet<MatchId>,
    match_format: crate::results::MatchFormat,
    must_be_complete: bool,
) -> Result<(), TournamentSnapshotError> {
    for scheduled in scheduled_matches {
        if !known_entrants.contains(&scheduled.home_entrant_id)
            || !known_entrants.contains(&scheduled.away_entrant_id)
        {
            return Err(TournamentSnapshotError::UnknownRoundEntrant);
        }
        if !known_matches.insert(scheduled.match_id.clone()) {
            return Err(TournamentSnapshotError::DuplicateMatch {
                match_id: scheduled.match_id.clone(),
            });
        }
    }
    if must_be_complete && results.len() != scheduled_matches.len() {
        return Err(TournamentSnapshotError::IncompleteHistoricalRound);
    }
    let mut result_ids = HashSet::with_capacity(results.len());
    for result in results {
        if !result_ids.insert(result.match_id().clone()) {
            return Err(TournamentSnapshotError::DuplicateResult {
                match_id: result.match_id().clone(),
            });
        }
        let scheduled = scheduled_matches
            .iter()
            .find(|scheduled| scheduled.match_id == *result.match_id())
            .ok_or_else(|| TournamentSnapshotError::UnknownResultMatch {
                match_id: result.match_id().clone(),
            })?;
        let active_scheduled = ScheduledMatch::published(
            scheduled.match_id.clone(),
            scheduled.home_entrant_id.clone(),
            scheduled.away_entrant_id.clone(),
            scheduled.table_number(),
            RoundActivity::Active,
        );
        let derived =
            validate_and_complete_match(&active_scheduled, match_format, result.games().to_vec())
                .map_err(|_| TournamentSnapshotError::InvalidResult {
                match_id: result.match_id().clone(),
            })?;
        if derived.home_games_won() != result.home_games_won()
            || derived.away_games_won() != result.away_games_won()
            || derived.winner_id() != result.winner_id()
        {
            return Err(TournamentSnapshotError::InvalidResult {
                match_id: result.match_id().clone(),
            });
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TournamentSnapshotError {
    UnsupportedSchemaVersion { version: u16 },
    DuplicateEntrant { entrant_id: EntrantId },
    UnknownActiveEntrant { entrant_id: EntrantId },
    DuplicateActiveEntrant { entrant_id: EntrantId },
    PreviewAndActiveRound,
    DraftContainsRounds,
    RoundGap,
    UnknownRoundEntrant,
    DuplicateMatch { match_id: MatchId },
    IncompleteHistoricalRound,
    DuplicateResult { match_id: MatchId },
    UnknownResultMatch { match_id: MatchId },
    InvalidResult { match_id: MatchId },
    InvalidStandings,
}

impl Display for TournamentSnapshotError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid tournament snapshot: {self:?}")
    }
}

impl Error for TournamentSnapshotError {}

#[cfg(test)]
mod tests {
    use crate::results::MatchFormat;
    use crate::tournament::{MaximumRoundCount, TableCount, TournamentId};

    use super::*;

    #[test]
    fn empty_draft_round_trips() {
        let tournament = Tournament::new(
            TournamentId::new("snapshot"),
            MatchFormat::BestOfThree,
            TableCount::try_from(4).unwrap(),
            MaximumRoundCount::try_from(5).unwrap(),
        );
        let application = TournamentApplication::new(tournament);
        let restored = TournamentApplication::restore(application.snapshot()).unwrap();
        assert_eq!(restored.snapshot(), application.snapshot());
    }
}
