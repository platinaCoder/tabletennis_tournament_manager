use std::collections::HashSet;

use crate::pairing::algorithms::blossom_v1::{PairingWarning, RelaxationTier};

use super::*;

#[test]
fn baseline_runs_the_complete_public_workflow() {
    let report = run_simulation(SimulationConfig::baseline()).unwrap();

    assert_eq!(report.rounds.len(), 5);
    assert_eq!(report.completed_match_count, 40);
    assert_eq!(report.final_standings.len(), 16);
    assert!(report.rounds.iter().all(|round| round.bye.is_none()));
    assert!(
        report
            .final_standings
            .iter()
            .all(|standing| standing.matches_played == 5)
    );
}

#[test]
fn standard_scenarios_cover_the_awkward_cases() {
    let reports = run_standard_scenarios().unwrap();
    let report = |name: &str| reports.iter().find(|report| report.name == name).unwrap();

    assert_eq!(
        report("elo-range-900-1500").minimum_starting_elo.value(),
        900
    );
    assert_eq!(
        report("elo-range-900-1500").maximum_starting_elo.value(),
        1_500
    );

    assert!(
        report("odd-entrant-count")
            .rounds
            .iter()
            .all(|round| round.bye.is_some())
    );
    assert!(report("dominant-club").rounds.iter().any(|round| {
        round.relaxation_tier != RelaxationTier::Strict
            && round
                .warnings
                .iter()
                .any(|warning| matches!(warning, PairingWarning::SameClubPairingRequired { .. }))
    }));
    assert_eq!(
        report("unavoidable-rematches").rounds[3].relaxation_tier,
        RelaxationTier::RematchesAllowed
    );
    assert!(
        report("unavoidable-rematches").rounds[3]
            .warnings
            .iter()
            .any(|warning| matches!(warning, PairingWarning::RematchRequired { .. }))
    );
    assert!(
        report("fewer-tables")
            .rounds
            .iter()
            .all(|round| round.unassigned_match_count == 5)
    );

    let identical_scores = report("identical-elo")
        .final_standings
        .iter()
        .map(|standing| standing.performance_score.scaled_value())
        .collect::<Vec<_>>();
    assert!(
        identical_scores.len()
            > identical_scores
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .len()
    );
    let odd_bye_counts = report("odd-entrant-count")
        .final_standings
        .iter()
        .map(|standing| standing.bye_count)
        .collect::<Vec<_>>();
    assert!(odd_bye_counts.iter().max().unwrap() - odd_bye_counts.iter().min().unwrap() <= 1);

    for report in &reports {
        assert_standing_invariants(report);
    }
}

fn assert_standing_invariants(report: &SimulationReport) {
    let total_wins = report
        .final_standings
        .iter()
        .map(|standing| standing.matches_won)
        .sum::<u32>();
    let total_losses = report
        .final_standings
        .iter()
        .map(|standing| standing.matches_lost)
        .sum::<u32>();
    let total_games_won = report
        .final_standings
        .iter()
        .map(|standing| standing.games_won)
        .sum::<u32>();
    let total_games_lost = report
        .final_standings
        .iter()
        .map(|standing| standing.games_lost)
        .sum::<u32>();
    let total_points_won = report
        .final_standings
        .iter()
        .map(|standing| standing.points_won)
        .sum::<u32>();
    let total_points_lost = report
        .final_standings
        .iter()
        .map(|standing| standing.points_lost)
        .sum::<u32>();
    let total_performance = report
        .final_standings
        .iter()
        .map(|standing| standing.performance_score.scaled_value())
        .sum::<i64>();

    assert_eq!(
        u32::try_from(report.completed_match_count).unwrap(),
        total_wins
    );
    assert_eq!(total_wins, total_losses);
    assert_eq!(total_games_won, total_games_lost);
    assert_eq!(total_points_won, total_points_lost);
    assert_eq!(total_performance, 0);
    assert!(report.final_standings.iter().all(|standing| {
        standing.matches_played + standing.bye_count == u32::from(report.configured_round_count)
    }));
}

#[test]
fn identical_configuration_and_seed_produce_identical_sporting_output() {
    let first = run_simulation(SimulationConfig::baseline()).unwrap();
    let second = run_simulation(SimulationConfig::baseline()).unwrap();

    assert_eq!(first.final_standings, second.final_standings);
    assert_eq!(first.rounds.len(), second.rounds.len());
    for (first_round, second_round) in first.rounds.iter().zip(&second.rounds) {
        assert_eq!(first_round.pairings, second_round.pairings);
        assert_eq!(first_round.bye, second_round.bye);
        assert_eq!(first_round.relaxation_tier, second_round.relaxation_tier);
        assert_eq!(first_round.total_cost, second_round.total_cost);
    }
}

#[test]
fn table_count_changes_only_table_availability() {
    let reports = run_standard_scenarios().unwrap();
    let baseline = reports
        .iter()
        .find(|report| report.name == "baseline")
        .unwrap();
    let fewer_tables = reports
        .iter()
        .find(|report| report.name == "fewer-tables")
        .unwrap();

    assert_eq!(baseline.final_standings, fewer_tables.final_standings);
    assert_eq!(
        baseline.completed_match_count,
        fewer_tables.completed_match_count
    );
    for (baseline_round, fewer_tables_round) in baseline.rounds.iter().zip(&fewer_tables.rounds) {
        assert_eq!(baseline_round.pairings, fewer_tables_round.pairings);
        assert_eq!(baseline_round.bye, fewer_tables_round.bye);
        assert_eq!(
            baseline_round.relaxation_tier,
            fewer_tables_round.relaxation_tier
        );
        assert_eq!(baseline_round.total_cost, fewer_tables_round.total_cost);
        assert_eq!(baseline_round.unassigned_match_count, 0);
        assert_eq!(fewer_tables_round.unassigned_match_count, 5);
    }
}

#[test]
fn public_match_simulation_is_deterministic_and_domain_valid() {
    let first = simulate_match_games(
        crate::results::MatchFormat::BestOfFive,
        crate::pairing::EloRating::new(1_500),
        crate::pairing::EloRating::new(1_200),
        42,
    )
    .unwrap();
    let second = simulate_match_games(
        crate::results::MatchFormat::BestOfFive,
        crate::pairing::EloRating::new(1_500),
        crate::pairing::EloRating::new(1_200),
        42,
    )
    .unwrap();
    let different_seed = simulate_match_games(
        crate::results::MatchFormat::BestOfFive,
        crate::pairing::EloRating::new(1_500),
        crate::pairing::EloRating::new(1_200),
        43,
    )
    .unwrap();

    assert_eq!(first, second);
    assert_ne!(first, different_seed);
    assert!(
        crate::results::evaluate_match_progress(crate::results::MatchFormat::BestOfFive, &first,)
            .unwrap()
            .is_complete()
    );
}

#[test]
fn simulated_match_winners_follow_the_match_level_elo_expectation() {
    use crate::results::{MatchFormat, MatchSide, evaluate_match_progress};

    let mut random = super::result_generator::DeterministicRandom::new(0x51_7a_7e);
    let mut underdog_wins = 0_u32;
    let sample_count = 20_000_u32;

    for _ in 0..sample_count {
        let games = super::result_generator::simulate_games(
            MatchFormat::BestOfFive,
            crate::pairing::EloRating::new(900),
            crate::pairing::EloRating::new(1_500),
            &mut random,
        )
        .unwrap();
        let progress = evaluate_match_progress(MatchFormat::BestOfFive, &games).unwrap();
        underdog_wins += u32::from(progress.winner() == Some(MatchSide::Home));
    }

    assert!(
        (450..=800).contains(&underdog_wins),
        "observed {underdog_wins} underdog wins in {sample_count} matches"
    );
}
