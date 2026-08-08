use crate::identity::{ClubId, EntrantId};
use crate::pairing::EloRating;
use crate::pairing::algorithms::blossom_v1::{BlossomV1Policy, PairingPolicyVersion};
use crate::pairing::algorithms::blossom_v2::BlossomV2Policy;
use crate::results::{GameScore, MatchFormat, MatchSide};
use crate::tournament::{MaximumRoundCount, TableCount, Tournament, TournamentId};

use super::*;

fn application(match_format: MatchFormat, entrant_count: usize) -> TournamentApplication {
    application_with_maximum_rounds(match_format, entrant_count, 5)
}

fn application_with_maximum_rounds(
    match_format: MatchFormat,
    entrant_count: usize,
    maximum_round_count: i64,
) -> TournamentApplication {
    let tournament = Tournament::new(
        TournamentId::new("tournament"),
        match_format,
        TableCount::try_from(2_i64).unwrap(),
        MaximumRoundCount::try_from(maximum_round_count).unwrap(),
    );
    let mut application = TournamentApplication::new(tournament);
    for index in 0..entrant_count {
        application
            .register_entrant(TournamentEntrant {
                entrant_id: EntrantId::new(format!("entrant-{index}")),
                name: format!("Entrant {index}"),
                club_id: ClubId::new(format!("club-{index}")),
                club_name: format!("Club {index}"),
                starting_elo: EloRating::new(1_400 + u32::try_from(index).unwrap() * 50),
            })
            .unwrap();
    }
    application
}

#[test]
fn pairing_stops_after_the_configured_final_round() {
    let mut application = application_with_maximum_rounds(MatchFormat::BestOfThree, 4, 1);
    application.start_tournament().unwrap();
    application
        .calculate_pairings(BlossomV1Policy::default())
        .unwrap();
    let matches = application
        .publish_pairings()
        .unwrap()
        .scheduled_matches
        .clone();
    for scheduled_match in matches {
        application
            .enter_match_result(&scheduled_match.match_id, two_zero())
            .unwrap();
    }
    application.complete_round().unwrap();

    assert!(matches!(
        application.calculate_pairings(BlossomV1Policy::default()),
        Err(TournamentApplicationError::MaximumRoundsCompleted {
            maximum_round_count: 1
        })
    ));
}

fn two_zero() -> Vec<GameScore> {
    vec![
        GameScore::new(1, 11, 4).unwrap(),
        GameScore::new(2, 11, 8).unwrap(),
    ]
}

fn two_one() -> Vec<GameScore> {
    vec![
        GameScore::new(1, 11, 9).unwrap(),
        GameScore::new(2, 7, 11).unwrap(),
        GameScore::new(3, 13, 11).unwrap(),
    ]
}

fn three_zero() -> Vec<GameScore> {
    vec![
        GameScore::new(1, 11, 2).unwrap(),
        GameScore::new(2, 11, 6).unwrap(),
        GameScore::new(3, 11, 9).unwrap(),
    ]
}

fn three_two() -> Vec<GameScore> {
    vec![
        GameScore::new(1, 12, 10).unwrap(),
        GameScore::new(2, 8, 11).unwrap(),
        GameScore::new(3, 11, 7).unwrap(),
        GameScore::new(4, 9, 11).unwrap(),
        GameScore::new(5, 14, 12).unwrap(),
    ]
}

#[test]
fn public_workflow_publishes_results_and_updates_complete_standings() {
    let mut application = application(MatchFormat::BestOfThree, 4);
    application.start_tournament().unwrap();

    let proposal = application
        .calculate_pairings(BlossomV1Policy::default())
        .unwrap();
    assert_eq!(proposal.matches.len(), 2);
    assert!(application.pending_pairing().is_some());

    let scheduled = application
        .publish_pairings()
        .unwrap()
        .scheduled_matches
        .clone();
    assert!(application.pending_pairing().is_none());
    assert_eq!(
        scheduled
            .iter()
            .filter_map(|game| game.table_number())
            .count(),
        2
    );
    for scheduled_match in scheduled {
        application
            .enter_match_result(&scheduled_match.match_id, two_zero())
            .unwrap();
    }
    application.complete_round().unwrap();

    assert!(application.active_round().is_none());
    assert_eq!(application.completed_rounds().len(), 1);
    assert!(application.standings().iter().all(|standing| {
        standing.matches_played == 1
            && standing.matches_won + standing.matches_lost == 1
            && standing.games_won + standing.games_lost == 2
            && standing.points_won > 0
    }));
}

#[test]
fn simulation_trace_retains_pairing_graphs_results_and_standing_totals() {
    let application = completed_one_round(MatchFormat::BestOfThree, two_one());

    let trace = application.simulation_trace().unwrap();
    let round = &trace.completed_rounds[0];

    assert_eq!(trace.schema_version, 2);
    assert_eq!(trace.simulation.run_seed, None);
    assert_eq!(trace.tournament.match_format, "best_of_three");
    assert_eq!(round.pairing.request.round_number, 1);
    assert_eq!(round.pairing.request.entrants.len(), 4);
    assert_eq!(round.pairing.relaxation_graphs.len(), 3);
    assert_eq!(
        round
            .pairing
            .relaxation_graphs
            .iter()
            .flat_map(|graph| &graph.edges)
            .filter(|edge| edge.selected)
            .count(),
        2
    );
    assert_eq!(round.results.len(), 2);
    assert!(round.results.iter().all(|result| {
        result.games.len() == 3
            && result.home_games_won == 2
            && result.away_games_won == 1
            && result.revision == 1
    }));
    let after = round.standings_after_round.as_ref().unwrap();
    assert!(after.iter().all(|standing| {
        standing.matches_played == 1
            && standing.games_won + standing.games_lost == 3
            && standing.points_won + standing.points_lost > 0
    }));
    assert_eq!(trace.current_standings, *after);

    let seeded_trace = application.simulation_trace_with_result_seed(42).unwrap();
    assert_eq!(seeded_trace.simulation.run_seed, Some(42));
    assert_eq!(
        seeded_trace.simulation.result_generator.as_deref(),
        Some("elo_match_outcome_with_generated_games_v2")
    );
}

#[test]
fn application_and_trace_preserve_the_selected_v2_policy() {
    let mut application = application(MatchFormat::BestOfThree, 4);
    application.start_tournament().unwrap();

    let proposal = application
        .calculate_pairings(BlossomV2Policy::default())
        .unwrap();
    let trace = application.simulation_trace().unwrap();
    let policy = &trace.pending_pairing.unwrap().request.policy;

    assert_eq!(proposal.policy_version, PairingPolicyVersion::BlossomV2);
    assert_eq!(policy.version, "blossom_v2");
    assert_eq!(policy.match_win_weight, None);
    assert_eq!(policy.match_record_weight, Some(1_000_000_000));
    assert_eq!(policy.elo_difference_weight, None);
    assert_eq!(policy.squared_elo_difference_weight, Some(10));
}

#[test]
fn game_and_point_margins_do_not_change_performance_score() {
    assert_same_performance_with_different_details(MatchFormat::BestOfThree, two_zero(), two_one());
    assert_same_performance_with_different_details(
        MatchFormat::BestOfFive,
        three_zero(),
        three_two(),
    );
}

fn assert_same_performance_with_different_details(
    match_format: MatchFormat,
    first_games: Vec<GameScore>,
    second_games: Vec<GameScore>,
) {
    let first = completed_one_round(match_format, first_games);
    let second = completed_one_round(match_format, second_games);

    let first_scores = first
        .standings()
        .iter()
        .map(|standing| (&standing.entrant_id, standing.performance_score))
        .collect::<std::collections::HashMap<_, _>>();
    let second_scores = second
        .standings()
        .iter()
        .map(|standing| (&standing.entrant_id, standing.performance_score))
        .collect::<std::collections::HashMap<_, _>>();

    assert_eq!(first_scores, second_scores);
    assert_ne!(
        first
            .standings()
            .iter()
            .map(|standing| standing.games_won)
            .sum::<u32>(),
        second
            .standings()
            .iter()
            .map(|standing| standing.games_won)
            .sum::<u32>()
    );
}

#[test]
fn elo_expectation_delta_is_conserved_and_uses_only_the_match_winner() {
    let delta = EloExpectationDeltaV1::calculate(
        EloRating::new(1_500),
        EloRating::new(1_500),
        MatchSide::Home,
    );

    assert_eq!(delta.home.scaled_value(), 500_000);
    assert_eq!(delta.away.scaled_value(), -500_000);
    assert_eq!(delta.home.scaled_value() + delta.away.scaled_value(), 0);
}

#[test]
fn incomplete_round_cannot_update_standings() {
    let mut application = application(MatchFormat::BestOfThree, 4);
    application.start_tournament().unwrap();
    application
        .calculate_pairings(BlossomV1Policy::default())
        .unwrap();
    application.publish_pairings().unwrap();

    assert!(matches!(
        application.complete_round(),
        Err(TournamentApplicationError::RoundIncomplete {
            missing_result_count: 2
        })
    ));
    assert!(application.completed_rounds().is_empty());
}

#[test]
fn completing_a_match_releases_its_table_to_the_next_waiting_match() {
    let mut application = application(MatchFormat::BestOfThree, 6);
    application.start_tournament().unwrap();
    application
        .calculate_pairings(BlossomV1Policy::default())
        .unwrap();
    let initially_scheduled = application
        .publish_pairings()
        .unwrap()
        .scheduled_matches
        .clone();
    let assigned = initially_scheduled
        .iter()
        .filter(|scheduled| scheduled.table_number().is_some())
        .collect::<Vec<_>>();
    let waiting = initially_scheduled
        .iter()
        .find(|scheduled| scheduled.table_number().is_none())
        .unwrap();

    assert_eq!(assigned.len(), 2);
    assert!(matches!(
        application.enter_match_result(&waiting.match_id, two_zero()),
        Err(TournamentApplicationError::MatchAwaitingTable { .. })
    ));

    let released_table = assigned[0].table_number();
    application
        .enter_match_result(&assigned[0].match_id, two_zero())
        .unwrap();
    let reassigned = application
        .active_round()
        .unwrap()
        .scheduled_matches
        .iter()
        .find(|scheduled| scheduled.match_id == waiting.match_id)
        .unwrap();

    assert_eq!(reassigned.table_number(), released_table);
}

#[test]
fn started_roster_changes_affect_future_pairings_without_deleting_history() {
    let mut application = application(MatchFormat::BestOfThree, 4);
    application.start_tournament().unwrap();
    application
        .calculate_pairings(BlossomV1Policy::default())
        .unwrap();
    let first_round = application
        .publish_pairings()
        .unwrap()
        .scheduled_matches
        .clone();
    for scheduled_match in first_round {
        application
            .enter_match_result(&scheduled_match.match_id, two_zero())
            .unwrap();
    }
    application.complete_round().unwrap();

    let withdrawn_id = EntrantId::new("entrant-0");
    let mut active_roster = application
        .active_entrants()
        .filter(|entrant| entrant.entrant_id != withdrawn_id)
        .cloned()
        .collect::<Vec<_>>();
    active_roster.push(TournamentEntrant {
        entrant_id: EntrantId::new("late-entry"),
        name: "Late Entry".to_owned(),
        club_id: ClubId::new("late-club"),
        club_name: "Late Club".to_owned(),
        starting_elo: EloRating::new(1_300),
    });
    application.replace_active_roster(active_roster).unwrap();

    assert!(!application.is_entrant_active(&withdrawn_id));
    assert!(
        application
            .standings()
            .iter()
            .any(|standing| standing.entrant_id == withdrawn_id)
    );
    let proposal = application
        .calculate_pairings(BlossomV1Policy::default())
        .unwrap();
    assert!(proposal.matches.iter().all(|pairing| {
        pairing.first_entrant_id != withdrawn_id && pairing.second_entrant_id != withdrawn_id
    }));
    assert!(proposal.matches.iter().any(|pairing| {
        pairing.first_entrant_id == EntrantId::new("late-entry")
            || pairing.second_entrant_id == EntrantId::new("late-entry")
    }));
}

#[test]
fn roster_changes_do_not_rewrite_an_active_round() {
    let mut application = application(MatchFormat::BestOfThree, 4);
    application.start_tournament().unwrap();
    application
        .calculate_pairings(BlossomV1Policy::default())
        .unwrap();
    let published_matches = application
        .publish_pairings()
        .unwrap()
        .scheduled_matches
        .clone();
    let withdrawn_id = published_matches[0].home_entrant_id.clone();
    let active_roster = application
        .active_entrants()
        .filter(|entrant| entrant.entrant_id != withdrawn_id)
        .cloned()
        .collect::<Vec<_>>();

    application.replace_active_roster(active_roster).unwrap();

    assert_eq!(
        application.active_round().unwrap().scheduled_matches,
        published_matches
    );
    for scheduled_match in published_matches {
        application
            .enter_match_result(&scheduled_match.match_id, two_zero())
            .unwrap();
    }
    application.complete_round().unwrap();
    assert!(!application.is_entrant_active(&withdrawn_id));
}

#[test]
fn roster_changes_invalidate_an_unpublished_pairing_preview() {
    let mut application = application(MatchFormat::BestOfThree, 4);
    application.start_tournament().unwrap();
    application
        .calculate_pairings(BlossomV1Policy::default())
        .unwrap();
    let mut roster = application.active_entrants().cloned().collect::<Vec<_>>();
    roster[0].name = "Corrected name".to_owned();

    application.replace_active_roster(roster).unwrap();

    assert!(application.pending_pairing().is_none());
    assert_eq!(
        application.active_entrants().next().unwrap().name,
        "Corrected name"
    );
}

fn completed_one_round(match_format: MatchFormat, games: Vec<GameScore>) -> TournamentApplication {
    let mut application = application(match_format, 4);
    application.start_tournament().unwrap();
    application
        .calculate_pairings(BlossomV1Policy::default())
        .unwrap();
    let matches = application
        .publish_pairings()
        .unwrap()
        .scheduled_matches
        .clone();
    for scheduled_match in matches {
        application
            .enter_match_result(&scheduled_match.match_id, games.clone())
            .unwrap();
    }
    application.complete_round().unwrap();
    application
}
