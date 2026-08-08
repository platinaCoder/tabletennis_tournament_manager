use std::collections::{HashMap, HashSet};

use tabletennis_tournament::simulation::simulate_match_games;

use crate::app::App;

impl App {
    pub(crate) fn simulate_remaining_results(&mut self) -> Result<(), String> {
        let language = self.language;
        let run_seed = self
            .simulation_run_seed
            .ok_or_else(|| language.simulation_seed_error().to_owned())?;
        let application = self
            .application
            .as_ref()
            .ok_or_else(|| language.create_tournament_first_error().to_owned())?;
        let round = application
            .active_round()
            .ok_or_else(|| language.no_active_round_error().to_owned())?
            .clone();
        let match_format = application.tournament().match_format();
        let elo_by_entrant = application
            .entrants()
            .iter()
            .map(|entrant| (entrant.entrant_id.clone(), entrant.starting_elo))
            .collect::<HashMap<_, _>>();
        let completed = round
            .results
            .iter()
            .map(|result| result.match_id().clone())
            .collect::<HashSet<_>>();
        let mut simulated = Vec::new();

        for scheduled in &round.scheduled_matches {
            if completed.contains(&scheduled.match_id) {
                continue;
            }
            let home_elo = elo_by_entrant
                .get(&scheduled.home_entrant_id)
                .copied()
                .ok_or_else(|| unknown_contestant(language))?;
            let away_elo = elo_by_entrant
                .get(&scheduled.away_entrant_id)
                .copied()
                .ok_or_else(|| unknown_contestant(language))?;
            let games = simulate_match_games(
                match_format,
                home_elo,
                away_elo,
                crate::simulation_seed::match_simulation_seed(
                    run_seed,
                    scheduled.match_id.as_str(),
                ),
            )
            .map_err(error)?;
            simulated.push((scheduled.match_id.clone(), games));
        }

        let application = self
            .application
            .as_mut()
            .ok_or_else(|| language.create_tournament_first_error().to_owned())?;
        for (match_id, games) in simulated {
            application
                .enter_match_result(&match_id, games)
                .map_err(error)?;
        }
        Ok(())
    }
}

fn unknown_contestant(language: crate::language::Language) -> String {
    language.unknown_simulated_contestant_error().to_owned()
}

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
