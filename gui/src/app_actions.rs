use std::collections::{HashMap, HashSet};

use tabletennis_tournament::api_contract::{
    EntrantInput, GameScoreInput, RecordMatchResultRequest, ReplaceRosterRequest, TournamentView,
};
use tabletennis_tournament::application::TournamentApplication;
use tabletennis_tournament::simulation::simulate_match_games;

use crate::app::App;
use crate::components::SubmittedResult;
use crate::model::{RosterEntryCommand, WorkspacePage};

impl App {
    pub(crate) fn roster_request(
        &self,
        commands: Vec<RosterEntryCommand>,
    ) -> Result<ReplaceRosterRequest, String> {
        let revision = self
            .tournament_revision
            .ok_or_else(|| self.language.create_tournament_first_error().to_owned())?;
        let mut ids = HashSet::new();
        let entrants = commands
            .into_iter()
            .map(|command| {
                if command.name.trim().is_empty() || command.club_name.trim().is_empty() {
                    return Err(self.language.roster_fields_error().to_owned());
                }
                if let Some(id) = &command.entrant_id
                    && !ids.insert(id.clone())
                {
                    return Err(self.language.duplicate_roster_error().to_owned());
                }
                Ok(EntrantInput {
                    entrant_id: command.entrant_id,
                    display_name: command.name,
                    club_id: None,
                    club_name: command.club_name,
                    starting_elo: i64::from(command.starting_elo),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(ReplaceRosterRequest {
            expected_tournament_revision: revision,
            entrants,
        })
    }

    pub(crate) fn install_tournament_view(&mut self, view: TournamentView) -> Result<(), String> {
        self.tournament_record_id = Some(view.id);
        self.tournament_revision = Some(view.revision);
        self.tournament_access_role = Some(view.access_role);
        self.page = WorkspacePage::Tournament;
        self.application = Some(
            TournamentApplication::restore(view.application).map_err(|error| error.to_string())?,
        );
        Ok(())
    }
}

pub(crate) async fn simulate_remaining_results(
    tournament_id: &str,
    run_seed: u64,
) -> Result<TournamentView, String> {
    let mut view = crate::api_client::load_tournament(tournament_id).await?;
    loop {
        let application = TournamentApplication::restore(view.application.clone())
            .map_err(|error| error.to_string())?;
        let round = application
            .active_round()
            .ok_or_else(|| "There is no active round to simulate.".to_owned())?;
        let completed = round
            .results
            .iter()
            .map(|result| result.match_id().clone())
            .collect::<HashSet<_>>();
        if completed.len() == round.scheduled_matches.len() {
            return Ok(view);
        }
        let scheduled = round
            .scheduled_matches
            .iter()
            .find(|scheduled| {
                scheduled.table_number().is_some() && !completed.contains(&scheduled.match_id)
            })
            .ok_or_else(|| "No table is available for the next simulated match.".to_owned())?;
        let entrants = application
            .entrants()
            .iter()
            .map(|entrant| (&entrant.entrant_id, entrant.starting_elo))
            .collect::<HashMap<_, _>>();
        let home_elo = entrants
            .get(&scheduled.home_entrant_id)
            .copied()
            .ok_or_else(|| "The simulated home contestant is unknown.".to_owned())?;
        let away_elo = entrants
            .get(&scheduled.away_entrant_id)
            .copied()
            .ok_or_else(|| "The simulated away contestant is unknown.".to_owned())?;
        let submission = SubmittedResult {
            match_id: scheduled.match_id.clone(),
            games: simulate_match_games(
                application.tournament().match_format(),
                home_elo,
                away_elo,
                crate::simulation_seed::match_simulation_seed(
                    run_seed,
                    scheduled.match_id.as_str(),
                ),
            )
            .map_err(|error| error.to_string())?,
            expected_revision: 0,
        };
        view = crate::api_client::record_result(
            tournament_id,
            submission.match_id.as_str(),
            &result_request(&submission),
        )
        .await?;
    }
}

pub(crate) fn result_request(submission: &SubmittedResult) -> RecordMatchResultRequest {
    RecordMatchResultRequest {
        expected_revision: submission.expected_revision,
        games: submission
            .games
            .iter()
            .map(|game| GameScoreInput {
                game_number: i64::from(game.game_number.value()),
                home_points: i64::from(game.home_points.value()),
                away_points: i64::from(game.away_points.value()),
            })
            .collect(),
        correction_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use tabletennis_tournament::identity::MatchId;
    use tabletennis_tournament::results::GameScore;

    use super::*;

    #[test]
    fn correction_request_preserves_match_revision() {
        let request = result_request(&SubmittedResult {
            match_id: MatchId::new("match"),
            games: vec![GameScore::new(1, 11, 7).unwrap()],
            expected_revision: 2,
        });

        assert_eq!(request.expected_revision, 2);
        assert_eq!(request.correction_reason, None);
    }
}
