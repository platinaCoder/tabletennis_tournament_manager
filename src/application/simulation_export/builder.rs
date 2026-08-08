use std::time::{SystemTime, UNIX_EPOCH};

use crate::results::{MatchFormat, MatchResult};
use crate::scheduling::{MatchPublicationStatus, RoundActivity, ScheduledMatch};
use crate::tournament::TournamentState;

use super::SimulationTraceError;
use super::model::{
    EntrantTrace, GameScoreTrace, MatchResultTrace, RoundTrace, ScheduledMatchTrace,
    SimulationMetadataTrace, SimulationTrace, StandingTrace, TournamentTrace,
};
use super::pairing_trace::pairing_calculation;
use crate::application::standings::calculate_standings;
use crate::application::{
    ActiveRound, CompletedRound, ContestantStanding, TournamentApplication, TournamentEntrant,
};

impl TournamentApplication {
    /// Creates a full developer trace of the current in-memory tournament.
    ///
    /// The trace includes exact historical pairing requests, all relaxation
    /// graphs, published matches, individual game scores and standings.
    pub fn simulation_trace(&self) -> Result<SimulationTrace, SimulationTraceError> {
        self.build_simulation_trace(None)
    }

    /// Creates a developer trace for results generated from `run_seed`.
    pub fn simulation_trace_with_result_seed(
        &self,
        run_seed: u64,
    ) -> Result<SimulationTrace, SimulationTraceError> {
        self.build_simulation_trace(Some(run_seed))
    }

    fn build_simulation_trace(
        &self,
        run_seed: Option<u64>,
    ) -> Result<SimulationTrace, SimulationTraceError> {
        let completed_rounds = self
            .completed_rounds
            .iter()
            .enumerate()
            .map(|(index, round)| {
                let before = calculate_standings(&self.entrants, &self.completed_rounds[..index])?;
                let after = calculate_standings(&self.entrants, &self.completed_rounds[..=index])?;
                completed_round(round, &before, &after)
            })
            .collect::<Result<Vec<_>, SimulationTraceError>>()?;
        let active_round = self
            .active_round
            .as_ref()
            .map(|round| active_round(round, &self.standings))
            .transpose()?;
        let pending_pairing = self
            .pending_pairing
            .as_ref()
            .map(|pending| pairing_calculation(&pending.request, &pending.proposal))
            .transpose()?;

        Ok(SimulationTrace {
            schema_version: 2,
            simulation: SimulationMetadataTrace {
                result_generator: run_seed
                    .map(|_| "elo_match_outcome_with_generated_games_v2".to_owned()),
                run_seed,
            },
            tournament: TournamentTrace {
                tournament_id: self.tournament.id().as_str().to_owned(),
                state: tournament_state(self.tournament.state()).to_owned(),
                match_format: match_format(self.tournament.match_format()).to_owned(),
                table_count: self.tournament.table_count().value(),
                maximum_round_count: self.tournament.maximum_round_count().value(),
            },
            entrants: self
                .entrants
                .iter()
                .map(|entrant| entrant_trace(self, entrant))
                .collect(),
            completed_rounds,
            active_round,
            pending_pairing,
            current_standings: standings(&self.standings),
        })
    }
}

fn completed_round(
    round: &CompletedRound,
    standings_before: &[ContestantStanding],
    standings_after: &[ContestantStanding],
) -> Result<RoundTrace, SimulationTraceError> {
    Ok(RoundTrace {
        round_number: round.round_number.value(),
        pairing: pairing_calculation(&round.pairing_request, &round.proposal)?,
        scheduled_matches: round
            .scheduled_matches
            .iter()
            .map(scheduled_match)
            .collect(),
        results: round.results.iter().map(match_result).collect(),
        bye_entrant_id: round.bye.as_ref().map(|id| id.as_str().to_owned()),
        standings_before_round: standings(standings_before),
        standings_after_round: Some(standings(standings_after)),
    })
}

fn active_round(
    round: &ActiveRound,
    standings_before: &[ContestantStanding],
) -> Result<RoundTrace, SimulationTraceError> {
    Ok(RoundTrace {
        round_number: round.round_number.value(),
        pairing: pairing_calculation(&round.pairing_request, &round.proposal)?,
        scheduled_matches: round
            .scheduled_matches
            .iter()
            .map(scheduled_match)
            .collect(),
        results: round.results.iter().map(match_result).collect(),
        bye_entrant_id: round.bye.as_ref().map(|id| id.as_str().to_owned()),
        standings_before_round: standings(standings_before),
        standings_after_round: None,
    })
}

fn entrant_trace(application: &TournamentApplication, entrant: &TournamentEntrant) -> EntrantTrace {
    EntrantTrace {
        entrant_id: entrant.entrant_id.as_str().to_owned(),
        name: entrant.name.clone(),
        club_id: entrant.club_id.as_str().to_owned(),
        club_name: entrant.club_name.clone(),
        starting_elo: entrant.starting_elo.value(),
        active: application.is_entrant_active(&entrant.entrant_id),
    }
}

fn scheduled_match(value: &ScheduledMatch) -> ScheduledMatchTrace {
    ScheduledMatchTrace {
        match_id: value.match_id.as_str().to_owned(),
        home_entrant_id: value.home_entrant_id.as_str().to_owned(),
        away_entrant_id: value.away_entrant_id.as_str().to_owned(),
        table_number: value.table_number().map(|table| table.value()),
        publication_status: publication_status(value.publication_status).to_owned(),
        round_activity: round_activity(value.round_activity).to_owned(),
    }
}

fn match_result(result: &MatchResult) -> MatchResultTrace {
    MatchResultTrace {
        match_id: result.match_id().as_str().to_owned(),
        games: result
            .games()
            .iter()
            .map(|game| GameScoreTrace {
                game_number: game.game_number.value(),
                home_points: game.home_points.value(),
                away_points: game.away_points.value(),
            })
            .collect(),
        home_games_won: result.home_games_won().value(),
        away_games_won: result.away_games_won().value(),
        winner_entrant_id: result.winner_id().as_str().to_owned(),
        entered_at_unix_milliseconds: unix_milliseconds(result.entered_at()),
        corrected_at_unix_milliseconds: result.corrected_at().map(unix_milliseconds),
        revision: result.revision().value(),
    }
}

fn standings(values: &[ContestantStanding]) -> Vec<StandingTrace> {
    values
        .iter()
        .enumerate()
        .map(|(index, standing)| StandingTrace {
            rank: index + 1,
            entrant_id: standing.entrant_id.as_str().to_owned(),
            performance_score_scaled: standing.performance_score.scaled_value(),
            matches_played: standing.matches_played,
            matches_won: standing.matches_won,
            matches_lost: standing.matches_lost,
            games_won: standing.games_won,
            games_lost: standing.games_lost,
            game_difference: standing.game_difference,
            points_won: standing.points_won,
            points_lost: standing.points_lost,
            point_difference: standing.point_difference,
            opponent_score_sum_scaled: standing.opponent_score_sum.scaled_value(),
            bye_count: standing.bye_count,
        })
        .collect()
}

fn unix_milliseconds(time: SystemTime) -> u128 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

const fn tournament_state(state: TournamentState) -> &'static str {
    match state {
        TournamentState::Draft => "draft",
        TournamentState::Started => "started",
    }
}

const fn match_format(format: MatchFormat) -> &'static str {
    match format {
        MatchFormat::BestOfThree => "best_of_three",
        MatchFormat::BestOfFive => "best_of_five",
    }
}

const fn publication_status(status: MatchPublicationStatus) -> &'static str {
    match status {
        MatchPublicationStatus::Draft => "draft",
        MatchPublicationStatus::Published => "published",
    }
}

const fn round_activity(activity: RoundActivity) -> &'static str {
    match activity {
        RoundActivity::Active => "active",
        RoundActivity::Inactive => "inactive",
    }
}
