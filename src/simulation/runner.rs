use std::collections::HashMap;

use crate::application::{TournamentApplication, TournamentEntrant};
use crate::identity::{ClubId, EntrantId};
use crate::pairing::EloRating;
use crate::tournament::{MaximumRoundCount, Tournament};

use super::result_generator::{DeterministicRandom, simulate_games};
use super::{
    SimulationConfig, SimulationEntrantPattern, SimulationError, SimulationReport,
    SimulationRoundReport, standard_scenarios,
};

pub fn run_simulation(config: SimulationConfig) -> Result<SimulationReport, SimulationError> {
    validate_config(&config)?;
    let maximum_round_count =
        MaximumRoundCount::try_from(i64::from(config.round_count)).map_err(|_| {
            SimulationError::InvalidConfiguration {
                reason: "configured round count exceeds the tournament limit",
            }
        })?;
    let tournament = Tournament::new(
        config.tournament_id.clone(),
        config.match_format,
        config.table_count,
        maximum_round_count,
    );
    let mut application = TournamentApplication::new(tournament);
    for entrant in generate_entrants(&config) {
        application.register_entrant(entrant)?;
    }
    application.start_tournament()?;

    let elo_by_entrant = application
        .entrants()
        .iter()
        .map(|entrant| (entrant.entrant_id.clone(), entrant.starting_elo))
        .collect::<HashMap<_, _>>();
    let minimum_starting_elo =
        elo_by_entrant
            .values()
            .copied()
            .min()
            .ok_or(SimulationError::InvalidConfiguration {
                reason: "simulation generated no entrants",
            })?;
    let maximum_starting_elo =
        elo_by_entrant
            .values()
            .copied()
            .max()
            .ok_or(SimulationError::InvalidConfiguration {
                reason: "simulation generated no entrants",
            })?;
    let mut random = DeterministicRandom::new(config.random_seed);
    let mut rounds = Vec::with_capacity(usize::from(config.round_count));

    for _ in 0..config.round_count {
        let proposal = application.calculate_pairings(config.pairing_policy.clone())?;
        let scheduled_matches = application.publish_pairings()?.scheduled_matches.clone();
        let initially_unassigned_match_count = scheduled_matches
            .iter()
            .filter(|scheduled| scheduled.table_number().is_none())
            .count();
        for scheduled_match in &scheduled_matches {
            let home_elo = entrant_elo(&elo_by_entrant, &scheduled_match.home_entrant_id)?;
            let away_elo = entrant_elo(&elo_by_entrant, &scheduled_match.away_entrant_id)?;
            let games = simulate_games(config.match_format, home_elo, away_elo, &mut random)?;
            application.enter_match_result(&scheduled_match.match_id, games)?;
        }
        let completed = application.complete_round()?;
        rounds.push(SimulationRoundReport {
            round_number: completed.round_number,
            relaxation_tier: proposal.relaxation_tier,
            total_cost: proposal.total_cost,
            pairings: proposal.matches,
            warnings: proposal.warnings,
            diagnostics: proposal.diagnostics,
            bye: completed.bye.clone(),
            match_count: completed.scheduled_matches.len(),
            unassigned_match_count: initially_unassigned_match_count,
        });
    }

    Ok(SimulationReport {
        name: config.name,
        entrant_count: config.entrant_count,
        configured_round_count: config.round_count,
        minimum_starting_elo,
        maximum_starting_elo,
        completed_match_count: rounds.iter().map(|round| round.match_count).sum(),
        rounds,
        final_standings: application.standings().to_vec(),
    })
}

pub fn run_standard_scenarios() -> Result<Vec<SimulationReport>, SimulationError> {
    standard_scenarios()
        .into_iter()
        .map(run_simulation)
        .collect()
}

fn validate_config(config: &SimulationConfig) -> Result<(), SimulationError> {
    if config.entrant_count < 2 {
        return Err(SimulationError::InvalidConfiguration {
            reason: "at least two entrants are required",
        });
    }
    if config.club_count == 0 {
        return Err(SimulationError::InvalidConfiguration {
            reason: "at least one club is required",
        });
    }
    if config.round_count == 0 {
        return Err(SimulationError::InvalidConfiguration {
            reason: "at least one round is required",
        });
    }
    Ok(())
}

fn generate_entrants(config: &SimulationConfig) -> Vec<TournamentEntrant> {
    (0..config.entrant_count)
        .map(|index| {
            let club_index = club_index(config, index);
            TournamentEntrant {
                entrant_id: EntrantId::new(format!("entrant-{index:02}")),
                name: format!("Contestant {}", index + 1),
                club_id: ClubId::new(format!("club-{club_index:02}")),
                club_name: format!("Club {}", club_index + 1),
                starting_elo: generated_elo(config.entrant_pattern, index, config.entrant_count),
            }
        })
        .collect()
}

fn club_index(config: &SimulationConfig, entrant_index: usize) -> usize {
    match config.entrant_pattern {
        SimulationEntrantPattern::DominantClub
            if entrant_index < config.entrant_count.saturating_mul(3) / 4 =>
        {
            0
        }
        SimulationEntrantPattern::DominantClub if config.club_count > 1 => {
            1 + entrant_index % (config.club_count - 1)
        }
        _ => entrant_index % config.club_count,
    }
}

fn generated_elo(
    pattern: SimulationEntrantPattern,
    entrant_index: usize,
    entrant_count: usize,
) -> EloRating {
    match pattern {
        SimulationEntrantPattern::EloRange900To1500 => {
            let denominator = entrant_count.saturating_sub(1).max(1);
            let offset = entrant_index.saturating_mul(600) / denominator;
            EloRating::new(900 + u32::try_from(offset).unwrap_or(600))
        }
        SimulationEntrantPattern::IdenticalElo => EloRating::new(1_500),
        _ => EloRating::new(
            1_100
                + u32::try_from(entrant_index)
                    .unwrap_or(u32::MAX)
                    .saturating_mul(37),
        ),
    }
}

fn entrant_elo(
    elo_by_entrant: &HashMap<EntrantId, EloRating>,
    entrant_id: &EntrantId,
) -> Result<EloRating, SimulationError> {
    elo_by_entrant
        .get(entrant_id)
        .copied()
        .ok_or(SimulationError::InvalidConfiguration {
            reason: "published match references an unknown generated entrant",
        })
}
