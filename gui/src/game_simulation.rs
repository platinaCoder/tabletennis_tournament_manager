use std::collections::{HashMap, HashSet};

use tabletennis_tournament::simulation::simulate_match_games;

use crate::app::App;

impl App {
    pub(crate) fn simulate_remaining_results(&mut self) -> Result<(), String> {
        let application = self
            .application
            .as_ref()
            .ok_or_else(|| "Create a tournament first.".to_owned())?;
        let round = application
            .active_round()
            .ok_or_else(|| "There is no active round to simulate.".to_owned())?
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
                .ok_or_else(unknown_contestant)?;
            let away_elo = elo_by_entrant
                .get(&scheduled.away_entrant_id)
                .copied()
                .ok_or_else(unknown_contestant)?;
            let games = simulate_match_games(
                match_format,
                home_elo,
                away_elo,
                stable_seed(scheduled.match_id.as_str()),
            )
            .map_err(error)?;
            simulated.push((scheduled.match_id.clone(), games));
        }

        let application = self
            .application
            .as_mut()
            .ok_or_else(|| "Create a tournament first.".to_owned())?;
        for (match_id, games) in simulated {
            application
                .enter_match_result(&match_id, games)
                .map_err(error)?;
        }
        Ok(())
    }
}

fn stable_seed(value: &str) -> u64 {
    value
        .bytes()
        .fold(14_695_981_039_346_656_037, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(1_099_511_628_211)
        })
}

fn unknown_contestant() -> String {
    "A simulated match references an unknown contestant.".to_owned()
}

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
