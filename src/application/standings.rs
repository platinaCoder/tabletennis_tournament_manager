use std::collections::HashMap;

use crate::identity::EntrantId;
use crate::pairing::algorithms::blossom_v1::PerformanceScore;
use crate::results::MatchSide;

use super::scoring::EloExpectationDeltaV1;
use super::standing_accumulator::StandingAccumulator;
use super::{CompletedRound, TournamentApplicationError, TournamentEntrant};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContestantStanding {
    pub entrant_id: EntrantId,
    pub performance_score: PerformanceScore,
    pub matches_played: u32,
    pub matches_won: u32,
    pub matches_lost: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub game_difference: i32,
    pub points_won: u32,
    pub points_lost: u32,
    pub point_difference: i32,
    pub opponent_score_sum: PerformanceScore,
    pub bye_count: u32,
}

pub(super) fn calculate_standings(
    entrants: &[TournamentEntrant],
    rounds: &[CompletedRound],
) -> Result<Vec<ContestantStanding>, TournamentApplicationError> {
    let entrant_by_id = entrants
        .iter()
        .map(|entrant| (&entrant.entrant_id, entrant))
        .collect::<HashMap<_, _>>();
    let mut accumulated = entrants
        .iter()
        .map(|entrant| {
            (
                entrant.entrant_id.clone(),
                StandingAccumulator::new(entrant.entrant_id.clone()),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut opponents = Vec::new();

    for round in rounds {
        if let Some(bye) = &round.bye {
            accumulator(&mut accumulated, bye)?.record_bye()?;
        }
        for result in &round.results {
            let scheduled = round
                .scheduled_matches
                .iter()
                .find(|scheduled| scheduled.match_id == *result.match_id())
                .ok_or_else(|| TournamentApplicationError::InvalidRoundHistory {
                    match_id: result.match_id().clone(),
                })?;
            let home = entrant_by_id
                .get(&scheduled.home_entrant_id)
                .ok_or_else(|| unknown_entrant(&scheduled.home_entrant_id))?;
            let away = entrant_by_id
                .get(&scheduled.away_entrant_id)
                .ok_or_else(|| unknown_entrant(&scheduled.away_entrant_id))?;
            let winner = winner_side(result.winner_id(), scheduled)?;
            let delta =
                EloExpectationDeltaV1::calculate(home.starting_elo, away.starting_elo, winner);
            accumulator(&mut accumulated, &scheduled.home_entrant_id)?.record_result(
                result.home_games_won().value(),
                result.away_games_won().value(),
                result.games().iter().map(|game| game.home_points.value()),
                result.games().iter().map(|game| game.away_points.value()),
                winner == MatchSide::Home,
                delta.home,
            )?;
            accumulator(&mut accumulated, &scheduled.away_entrant_id)?.record_result(
                result.away_games_won().value(),
                result.home_games_won().value(),
                result.games().iter().map(|game| game.away_points.value()),
                result.games().iter().map(|game| game.home_points.value()),
                winner == MatchSide::Away,
                delta.away,
            )?;
            opponents.push((
                scheduled.home_entrant_id.clone(),
                scheduled.away_entrant_id.clone(),
            ));
        }
    }

    add_opponent_strength(&mut accumulated, opponents)?;
    finish_and_rank(entrants, accumulated)
}

fn winner_side(
    winner: &EntrantId,
    scheduled: &crate::scheduling::ScheduledMatch,
) -> Result<MatchSide, TournamentApplicationError> {
    if winner == &scheduled.home_entrant_id {
        Ok(MatchSide::Home)
    } else if winner == &scheduled.away_entrant_id {
        Ok(MatchSide::Away)
    } else {
        Err(unknown_entrant(winner))
    }
}

fn add_opponent_strength(
    standings: &mut HashMap<EntrantId, StandingAccumulator>,
    opponents: Vec<(EntrantId, EntrantId)>,
) -> Result<(), TournamentApplicationError> {
    let scores = standings
        .iter()
        .map(|(id, standing)| (id.clone(), standing.performance_score()))
        .collect::<HashMap<_, _>>();
    for (first, second) in opponents {
        accumulator(standings, &first)?.add_opponent_score(scores[&second])?;
        accumulator(standings, &second)?.add_opponent_score(scores[&first])?;
    }
    Ok(())
}

fn finish_and_rank(
    entrants: &[TournamentEntrant],
    accumulated: HashMap<EntrantId, StandingAccumulator>,
) -> Result<Vec<ContestantStanding>, TournamentApplicationError> {
    let elo_by_id = entrants
        .iter()
        .map(|entrant| (&entrant.entrant_id, entrant.starting_elo))
        .collect::<HashMap<_, _>>();
    let mut standings = accumulated
        .into_values()
        .map(StandingAccumulator::finish)
        .collect::<Result<Vec<_>, _>>()?;
    standings.sort_by(|first, second| {
        second
            .performance_score
            .cmp(&first.performance_score)
            .then_with(|| second.matches_won.cmp(&first.matches_won))
            .then_with(|| second.opponent_score_sum.cmp(&first.opponent_score_sum))
            .then_with(|| elo_by_id[&second.entrant_id].cmp(&elo_by_id[&first.entrant_id]))
            .then_with(|| first.entrant_id.as_str().cmp(second.entrant_id.as_str()))
    });
    Ok(standings)
}

fn accumulator<'a>(
    standings: &'a mut HashMap<EntrantId, StandingAccumulator>,
    entrant_id: &EntrantId,
) -> Result<&'a mut StandingAccumulator, TournamentApplicationError> {
    standings
        .get_mut(entrant_id)
        .ok_or_else(|| unknown_entrant(entrant_id))
}

fn unknown_entrant(entrant_id: &EntrantId) -> TournamentApplicationError {
    TournamentApplicationError::UnknownEntrantInRound {
        entrant_id: entrant_id.clone(),
    }
}
