use std::collections::HashSet;

use crate::identity::EntrantId;
use crate::pairing::algorithms::PairingSnapshot;
use crate::pairing::algorithms::blossom_v1::PairingProposal;
use crate::tournament::{Tournament, TournamentState};

use super::standings::calculate_standings;
use super::{
    ActiveRound, CompletedRound, ContestantStanding, TournamentApplicationError, TournamentEntrant,
};

pub struct TournamentApplication {
    pub(super) tournament: Tournament,
    pub(super) entrants: Vec<TournamentEntrant>,
    pub(super) standings: Vec<ContestantStanding>,
    pub(super) completed_rounds: Vec<CompletedRound>,
    pub(super) active_round: Option<ActiveRound>,
    pub(super) pending_pairing: Option<PendingPairing>,
    pub(super) active_entrant_ids: HashSet<EntrantId>,
}

impl TournamentApplication {
    pub fn new(tournament: Tournament) -> Self {
        Self {
            tournament,
            entrants: Vec::new(),
            standings: Vec::new(),
            completed_rounds: Vec::new(),
            active_round: None,
            pending_pairing: None,
            active_entrant_ids: HashSet::new(),
        }
    }

    pub fn tournament(&self) -> &Tournament {
        &self.tournament
    }

    pub fn entrants(&self) -> &[TournamentEntrant] {
        &self.entrants
    }

    pub fn active_entrants(&self) -> impl Iterator<Item = &TournamentEntrant> {
        self.entrants
            .iter()
            .filter(|entrant| self.active_entrant_ids.contains(&entrant.entrant_id))
    }

    pub fn is_entrant_active(&self, entrant_id: &EntrantId) -> bool {
        self.active_entrant_ids.contains(entrant_id)
    }

    pub fn standings(&self) -> &[ContestantStanding] {
        &self.standings
    }

    pub fn active_round(&self) -> Option<&ActiveRound> {
        self.active_round.as_ref()
    }

    pub fn completed_rounds(&self) -> &[CompletedRound] {
        &self.completed_rounds
    }

    pub fn pending_pairing(&self) -> Option<&PairingProposal> {
        self.pending_pairing
            .as_ref()
            .map(|pending| &pending.proposal)
    }

    pub fn register_entrant(
        &mut self,
        entrant: TournamentEntrant,
    ) -> Result<(), TournamentApplicationError> {
        if self
            .entrants
            .iter()
            .any(|registered| registered.entrant_id == entrant.entrant_id)
        {
            return Err(TournamentApplicationError::DuplicateEntrant {
                entrant_id: entrant.entrant_id,
            });
        }
        let entrant_id = entrant.entrant_id.clone();
        let mut entrants = self.entrants.clone();
        entrants.push(entrant);
        entrants.sort_by(|first, second| first.entrant_id.as_str().cmp(second.entrant_id.as_str()));
        let standings = self.standings_for_roster(&entrants)?;
        self.entrants = entrants;
        self.active_entrant_ids.insert(entrant_id);
        self.standings = standings;
        self.pending_pairing = None;
        Ok(())
    }

    pub fn update_entrant(
        &mut self,
        replacement: TournamentEntrant,
    ) -> Result<(), TournamentApplicationError> {
        let mut entrants = self.entrants.clone();
        let entrant = entrants
            .iter_mut()
            .find(|entrant| entrant.entrant_id == replacement.entrant_id)
            .ok_or_else(|| TournamentApplicationError::UnknownEntrantInRound {
                entrant_id: replacement.entrant_id.clone(),
            })?;
        *entrant = replacement;
        let standings = self.standings_for_roster(&entrants)?;
        self.entrants = entrants;
        self.standings = standings;
        self.pending_pairing = None;
        Ok(())
    }

    pub fn replace_active_roster(
        &mut self,
        replacements: Vec<TournamentEntrant>,
    ) -> Result<(), TournamentApplicationError> {
        let mut active_entrant_ids = HashSet::with_capacity(replacements.len());
        let mut entrants = self.entrants.clone();
        for replacement in replacements {
            if !active_entrant_ids.insert(replacement.entrant_id.clone()) {
                return Err(TournamentApplicationError::DuplicateEntrant {
                    entrant_id: replacement.entrant_id,
                });
            }
            if let Some(existing) = entrants
                .iter_mut()
                .find(|entrant| entrant.entrant_id == replacement.entrant_id)
            {
                *existing = replacement;
            } else {
                entrants.push(replacement);
            }
        }
        entrants.sort_by(|first, second| first.entrant_id.as_str().cmp(second.entrant_id.as_str()));
        let standings = self.standings_for_roster(&entrants)?;
        self.entrants = entrants;
        self.active_entrant_ids = active_entrant_ids;
        self.standings = standings;
        self.pending_pairing = None;
        Ok(())
    }

    pub fn withdraw_entrant(
        &mut self,
        entrant_id: &EntrantId,
    ) -> Result<(), TournamentApplicationError> {
        if !self
            .entrants
            .iter()
            .any(|entrant| &entrant.entrant_id == entrant_id)
        {
            return Err(TournamentApplicationError::UnknownEntrantInRound {
                entrant_id: entrant_id.clone(),
            });
        }
        self.active_entrant_ids.remove(entrant_id);
        self.pending_pairing = None;
        Ok(())
    }

    pub fn start_tournament(&mut self) -> Result<(), TournamentApplicationError> {
        let entrant_count = self.active_entrant_ids.len();
        if entrant_count < 2 {
            return Err(TournamentApplicationError::NotEnoughEntrants { entrant_count });
        }
        self.tournament.start()?;
        self.standings = calculate_standings(&self.entrants, &self.completed_rounds)?;
        Ok(())
    }

    pub fn discard_pairing_preview(&mut self) {
        self.pending_pairing = None;
    }

    pub(super) fn ensure_started(&self) -> Result<(), TournamentApplicationError> {
        if self.tournament.state() == TournamentState::Started {
            Ok(())
        } else {
            Err(TournamentApplicationError::TournamentNotStarted)
        }
    }

    fn standings_for_roster(
        &self,
        entrants: &[TournamentEntrant],
    ) -> Result<Vec<ContestantStanding>, TournamentApplicationError> {
        if self.tournament.state() == TournamentState::Started {
            calculate_standings(entrants, &self.completed_rounds)
        } else {
            Ok(self.standings.clone())
        }
    }
}

pub(super) struct PendingPairing {
    pub request: PairingSnapshot,
    pub proposal: PairingProposal,
}
