use crate::identity::MatchId;
use crate::pairing::algorithms::blossom_v1::{PairingProposal, RoundNumber};
use crate::pairing::algorithms::{PairingPolicy, propose_pairings};
use crate::pairing::{
    MatchPublication, TableAssignmentEntrant, assign_tables, publish_scheduled_matches,
};
use crate::results::{GameScore, MatchResult, RoundActivity, validate_and_complete_match};

use super::pairing_snapshot::build_pairing_request;
use super::standings::calculate_standings;
use super::tournament::PendingPairing;
use super::{ActiveRound, CompletedRound, TournamentApplication, TournamentApplicationError};

impl TournamentApplication {
    /// Calculates and stores an unpublished preview. Recalculation safely
    /// replaces an older preview because no match identifiers exist yet.
    pub fn calculate_pairings(
        &mut self,
        policy: impl Into<PairingPolicy>,
    ) -> Result<PairingProposal, TournamentApplicationError> {
        self.ensure_started()?;
        if self.active_round.is_some() {
            return Err(TournamentApplicationError::ActiveRoundExists);
        }
        let maximum_round_count = self.tournament.maximum_round_count().value();
        if self.completed_rounds.len() >= usize::from(maximum_round_count) {
            return Err(TournamentApplicationError::MaximumRoundsCompleted {
                maximum_round_count,
            });
        }
        let active_entrants = self.active_entrants().cloned().collect::<Vec<_>>();
        let request = build_pairing_request(
            &active_entrants,
            &self.standings,
            &self.completed_rounds,
            self.next_round_number()?,
            policy.into(),
        )?;
        let proposal = propose_pairings(&request)?;
        self.pending_pairing = Some(PendingPairing {
            request,
            proposal: proposal.clone(),
        });
        Ok(proposal)
    }

    pub fn publish_pairings(&mut self) -> Result<&ActiveRound, TournamentApplicationError> {
        self.ensure_started()?;
        if self.active_round.is_some() {
            return Err(TournamentApplicationError::ActiveRoundExists);
        }
        let pending = self
            .pending_pairing
            .take()
            .ok_or(TournamentApplicationError::NoPairingPreview)?;
        let proposal = pending.proposal;
        let round_number = self.next_round_number()?;
        let publications = proposal
            .matches
            .iter()
            .enumerate()
            .map(|(index, pairing)| MatchPublication {
                match_id: self.match_id(round_number, index),
                first_entrant_id: pairing.first_entrant_id.clone(),
                second_entrant_id: pairing.second_entrant_id.clone(),
            })
            .collect();
        let published = publish_scheduled_matches(publications, RoundActivity::Active);
        let table_entrants = self
            .entrants
            .iter()
            .map(|entrant| TableAssignmentEntrant {
                entrant_id: entrant.entrant_id.clone(),
                starting_elo: entrant.starting_elo,
            })
            .collect::<Vec<_>>();
        let scheduled_matches =
            assign_tables(self.tournament.table_count(), published, &table_entrants)?;
        let bye = proposal.bye.as_ref().map(|bye| bye.entrant_id.clone());
        self.active_round = Some(ActiveRound {
            round_number,
            pairing_request: pending.request,
            proposal,
            scheduled_matches,
            results: Vec::new(),
            bye,
        });
        self.active_round
            .as_ref()
            .ok_or(TournamentApplicationError::NoActiveRound)
    }

    pub fn enter_match_result(
        &mut self,
        match_id: &MatchId,
        games: Vec<GameScore>,
    ) -> Result<MatchResult, TournamentApplicationError> {
        let match_format = self.tournament.match_format();
        let round = self
            .active_round
            .as_mut()
            .ok_or(TournamentApplicationError::NoActiveRound)?;
        if round
            .results
            .iter()
            .any(|result| result.match_id() == match_id)
        {
            return Err(TournamentApplicationError::ResultAlreadyEntered {
                match_id: match_id.clone(),
            });
        }
        let scheduled_match = round
            .scheduled_matches
            .iter()
            .find(|scheduled| &scheduled.match_id == match_id)
            .ok_or_else(|| TournamentApplicationError::UnknownMatch {
                match_id: match_id.clone(),
            })?;
        let released_table = scheduled_match.table_number().ok_or_else(|| {
            TournamentApplicationError::MatchAwaitingTable {
                match_id: match_id.clone(),
            }
        })?;
        let result = validate_and_complete_match(scheduled_match, match_format, games)?;
        round.results.push(result.clone());
        round
            .results
            .sort_by(|first, second| first.match_id().as_str().cmp(second.match_id().as_str()));
        assign_released_table(round, released_table);
        Ok(result)
    }

    pub fn complete_round(&mut self) -> Result<&CompletedRound, TournamentApplicationError> {
        let round = self
            .active_round
            .as_ref()
            .ok_or(TournamentApplicationError::NoActiveRound)?;
        let missing_result_count = round
            .scheduled_matches
            .len()
            .saturating_sub(round.results.len());
        if missing_result_count != 0 {
            return Err(TournamentApplicationError::RoundIncomplete {
                missing_result_count,
            });
        }

        let mut round = self
            .active_round
            .take()
            .ok_or(TournamentApplicationError::NoActiveRound)?;
        for scheduled_match in &mut round.scheduled_matches {
            scheduled_match.round_activity = RoundActivity::Inactive;
        }
        self.completed_rounds.push(CompletedRound {
            round_number: round.round_number,
            pairing_request: round.pairing_request,
            proposal: round.proposal,
            scheduled_matches: round.scheduled_matches,
            results: round.results,
            bye: round.bye,
        });
        self.standings = calculate_standings(&self.entrants, &self.completed_rounds)?;
        self.completed_rounds
            .last()
            .ok_or(TournamentApplicationError::NoActiveRound)
    }

    fn next_round_number(&self) -> Result<RoundNumber, TournamentApplicationError> {
        let next = self
            .completed_rounds
            .len()
            .checked_add(1)
            .and_then(|value| i64::try_from(value).ok())
            .ok_or(TournamentApplicationError::RoundNumberOverflow)?;
        RoundNumber::try_from(next).map_err(|_| TournamentApplicationError::RoundNumberOverflow)
    }

    fn match_id(&self, round_number: RoundNumber, index: usize) -> MatchId {
        MatchId::new(format!(
            "{}-round-{}-match-{}",
            self.tournament.id().as_str(),
            round_number.value(),
            index + 1
        ))
    }
}

fn assign_released_table(round: &mut ActiveRound, table_number: crate::table::TableNumber) {
    let Some(waiting_index) = round
        .scheduled_matches
        .iter()
        .position(|scheduled| scheduled.table_number().is_none())
    else {
        return;
    };
    round.scheduled_matches[waiting_index] = round.scheduled_matches[waiting_index]
        .clone()
        .with_table_number(Some(table_number));
}
