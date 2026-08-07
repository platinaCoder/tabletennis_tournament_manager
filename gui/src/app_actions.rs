use std::collections::HashSet;

use tabletennis_tournament::application::{TournamentApplication, TournamentEntrant};
use tabletennis_tournament::identity::{ClubId, EntrantId};
use tabletennis_tournament::pairing::EloRating;
use tabletennis_tournament::pairing::algorithms::blossom_v1::BlossomV1Policy;
use tabletennis_tournament::tournament::{MaximumRoundCount, TableCount, Tournament, TournamentId};

use crate::app::{App, Msg, RosterAction};
use crate::model::{CreateTournamentCommand, RosterEntryCommand};

impl App {
    pub(crate) fn handle(&mut self, message: Msg) -> Result<RosterAction, String> {
        match message {
            Msg::CreateTournament(command) => self.create_tournament(command),
            Msg::StartTournament(roster) => {
                self.save_roster(roster)?;
                self.application_mut()?.start_tournament().map_err(error)?;
                Ok(RosterAction::None)
            }
            Msg::SaveRoster(roster) => {
                self.save_roster(roster)?;
                Ok(RosterAction::CloseEditor)
            }
            Msg::CalculatePairings => self
                .application_mut()?
                .calculate_pairings(BlossomV1Policy::default())
                .map(|_| RosterAction::None)
                .map_err(error),
            Msg::PublishPairings => self
                .application_mut()?
                .publish_pairings()
                .map(|_| RosterAction::None)
                .map_err(error),
            Msg::SubmitResult(submission) => self
                .application_mut()?
                .enter_match_result(&submission.match_id, submission.games)
                .map(|_| RosterAction::None)
                .map_err(error),
            Msg::SimulateRemainingResults => {
                if !self.development_tools_enabled {
                    return Err(self.language.simulation_route_error().to_owned());
                }
                self.simulate_remaining_results()?;
                Ok(RosterAction::None)
            }
            Msg::CompleteRound => self
                .application_mut()?
                .complete_round()
                .map(|_| RosterAction::None)
                .map_err(error),
            Msg::DismissError | Msg::ToggleRoster | Msg::ToggleDarkMode | Msg::ToggleLanguage => {
                Ok(RosterAction::None)
            }
        }
    }

    fn create_tournament(
        &mut self,
        command: CreateTournamentCommand,
    ) -> Result<RosterAction, String> {
        if command.tournament_id.trim().is_empty() {
            return Err(self.language.tournament_identifier_error().to_owned());
        }
        if !(2..=64).contains(&command.contestant_count) {
            return Err(self.language.contestant_range_error().to_owned());
        }
        let table_count = TableCount::try_from(command.table_count).map_err(error)?;
        let maximum_round_count =
            MaximumRoundCount::try_from(command.maximum_round_count).map_err(error)?;
        self.initial_contestant_count = command.contestant_count;
        self.application = Some(TournamentApplication::new(Tournament::new(
            TournamentId::new(command.tournament_id.trim()),
            command.match_format,
            table_count,
            maximum_round_count,
        )));
        Ok(RosterAction::None)
    }

    fn save_roster(&mut self, commands: Vec<RosterEntryCommand>) -> Result<(), String> {
        let known_entrants = self
            .application
            .as_ref()
            .ok_or_else(|| self.language.create_tournament_first_error().to_owned())?
            .entrants()
            .to_vec();
        let mut used_ids = HashSet::with_capacity(commands.len());
        let mut replacements = Vec::with_capacity(commands.len());

        for command in commands {
            if command.name.trim().is_empty() || command.club_name.trim().is_empty() {
                return Err(self.language.roster_fields_error().to_owned());
            }
            let entrant_id = match command.entrant_id {
                Some(id) => EntrantId::new(id),
                None => self.next_entrant_id(&known_entrants, &used_ids),
            };
            if !used_ids.insert(entrant_id.clone()) {
                return Err(self.language.duplicate_roster_error().to_owned());
            }
            let club_id =
                self.club_id_for_name(command.club_name.trim(), &known_entrants, &replacements);
            replacements.push(TournamentEntrant {
                entrant_id,
                name: command.name.trim().to_owned(),
                club_id,
                club_name: command.club_name.trim().to_owned(),
                starting_elo: EloRating::new(command.starting_elo),
            });
        }

        self.application_mut()?
            .replace_active_roster(replacements)
            .map_err(error)
    }

    fn next_entrant_id(
        &mut self,
        known_entrants: &[TournamentEntrant],
        used_ids: &HashSet<EntrantId>,
    ) -> EntrantId {
        loop {
            let id = EntrantId::new(format!("entrant-{:06}", self.next_entrant_number));
            self.next_entrant_number += 1;
            if !used_ids.contains(&id)
                && !known_entrants
                    .iter()
                    .any(|entrant| entrant.entrant_id == id)
            {
                return id;
            }
        }
    }

    fn club_id_for_name(
        &mut self,
        club_name: &str,
        known_entrants: &[TournamentEntrant],
        replacements: &[TournamentEntrant],
    ) -> ClubId {
        if let Some(entrant) = replacements
            .iter()
            .chain(known_entrants)
            .find(|entrant| entrant.club_name.eq_ignore_ascii_case(club_name))
        {
            return entrant.club_id.clone();
        }
        let club_id = ClubId::new(format!("club-{:06}", self.next_club_number));
        self.next_club_number += 1;
        club_id
    }

    fn application_mut(&mut self) -> Result<&mut TournamentApplication, String> {
        self.application
            .as_mut()
            .ok_or_else(|| self.language.create_tournament_first_error().to_owned())
    }
}

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
