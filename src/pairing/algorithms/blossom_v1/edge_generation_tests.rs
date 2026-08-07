use crate::identity::EntrantId;
use crate::pairing::EloRating;

use super::*;

fn round(number: i64) -> RoundNumber {
    RoundNumber::try_from(number).unwrap()
}

fn policy() -> BlossomV1Policy {
    BlossomV1Policy {
        avoid_same_club: true,
        avoid_rematches: true,
        recent_rematch_window: 3,
        performance_score_weight: 10,
        match_win_weight: 20,
        opponent_strength_weight: 5,
        elo_difference_weight: 2,
        bye_repeat_penalty: 100,
        same_club_penalty: 1_000,
        rematch_penalty: 10_000,
        maximum_entrant_count: 64,
    }
}

fn entrant(id: &str, club: &str, elo: u32) -> PairingEntrant {
    PairingEntrant {
        entrant_id: EntrantId::new(id),
        club_id: ClubId::new(club),
        starting_elo: EloRating::new(elo),
        performance_score: PerformanceScore::ZERO,
        matches_won: 0,
        opponent_score_sum: PerformanceScore::ZERO,
        bye_count: 0,
    }
}

fn request(round_number: i64, entrants: Vec<PairingEntrant>) -> PairingRequest {
    PairingRequest {
        round_number: round(round_number),
        entrants,
        previous_matches: Vec::new(),
        policy: policy(),
    }
}

fn match_edge<'a>(
    graph: &'a PairingCandidateGraph,
    first: &str,
    second: &str,
) -> &'a PairingCandidateEdge {
    graph
        .edges
        .iter()
        .find(|edge| {
            edge.first_entrant_id.as_str() == first
                && matches!(
                    &edge.target,
                    PairingEdgeTarget::Entrant(entrant_id) if entrant_id.as_str() == second
                )
        })
        .unwrap()
}

#[test]
fn relaxation_tiers_admit_edges_in_the_required_sequence() {
    let mut request = request(
        2,
        vec![
            entrant("a", "same", 1500),
            entrant("b", "same", 1500),
            entrant("c", "other", 1500),
        ],
    );
    request.previous_matches.push(PreviousMatch {
        first_entrant_id: EntrantId::new("a"),
        second_entrant_id: EntrantId::new("c"),
        round_number: round(1),
    });

    let strict = build_candidate_graph(&request, RelaxationTier::Strict).unwrap();
    assert_eq!(strict.diagnostics.candidate_pair_count, 3);
    assert_eq!(strict.diagnostics.rejected_same_club_edges, 1);
    assert_eq!(strict.diagnostics.rejected_rematch_edges, 1);
    assert_eq!(strict.diagnostics.eligible_edge_count, 4);

    let clubs = build_candidate_graph(&request, RelaxationTier::SameClubAllowed).unwrap();
    assert_eq!(clubs.diagnostics.rejected_same_club_edges, 0);
    assert_eq!(clubs.diagnostics.rejected_rematch_edges, 1);
    assert_eq!(clubs.diagnostics.eligible_edge_count, 5);
    assert_eq!(
        match_edge(&clubs, "a", "b").breakdown.same_club_penalty,
        1_000
    );

    let rematches = build_candidate_graph(&request, RelaxationTier::RematchesAllowed).unwrap();
    assert_eq!(rematches.diagnostics.rejected_same_club_edges, 0);
    assert_eq!(rematches.diagnostics.rejected_rematch_edges, 0);
    assert_eq!(rematches.diagnostics.eligible_edge_count, 6);
    assert_eq!(
        match_edge(&rematches, "a", "c").breakdown.rematch_penalty,
        10_000
    );

    assert_eq!(
        RelaxationTier::ORDERED,
        [
            RelaxationTier::Strict,
            RelaxationTier::SameClubAllowed,
            RelaxationTier::RematchesAllowed
        ]
    );
}

#[test]
fn round_one_cost_uses_elo_but_not_later_round_components() {
    let mut first = entrant("a", "one", 1000);
    first.performance_score = PerformanceScore::from_scaled(50);
    first.matches_won = 4;
    first.opponent_score_sum = PerformanceScore::from_scaled(90);
    let second = entrant("b", "two", 1200);
    let request = request(1, vec![first, second]);

    let graph = build_candidate_graph(&request, RelaxationTier::Strict).unwrap();
    let edge = match_edge(&graph, "a", "b");

    assert_eq!(edge.breakdown.performance_score_gap, 0);
    assert_eq!(edge.breakdown.match_win_gap, 0);
    assert_eq!(edge.breakdown.opponent_strength_gap, 0);
    assert_eq!(edge.breakdown.elo_gap, 400);
}

#[test]
fn later_round_cost_includes_every_competitive_component() {
    let mut first = entrant("a", "one", 1000);
    first.performance_score = PerformanceScore::from_scaled(7);
    first.matches_won = 3;
    first.opponent_score_sum = PerformanceScore::from_scaled(11);
    let mut second = entrant("b", "two", 1200);
    second.performance_score = PerformanceScore::from_scaled(2);
    second.matches_won = 1;
    second.opponent_score_sum = PerformanceScore::from_scaled(3);
    let request = request(2, vec![first, second]);

    let graph = build_candidate_graph(&request, RelaxationTier::Strict).unwrap();
    let edge = match_edge(&graph, "a", "b");

    assert_eq!(edge.breakdown.performance_score_gap, 50);
    assert_eq!(edge.breakdown.match_win_gap, 40);
    assert_eq!(edge.breakdown.opponent_strength_gap, 40);
    assert_eq!(edge.breakdown.elo_gap, 400);
}

#[test]
fn odd_entrant_graph_contains_one_penalized_bye_edge_per_entrant() {
    let mut repeated_bye = entrant("a", "one", 1500);
    repeated_bye.bye_count = 2;
    let request = request(
        2,
        vec![
            repeated_bye,
            entrant("b", "two", 1500),
            entrant("c", "three", 1500),
        ],
    );

    let graph = build_candidate_graph(&request, RelaxationTier::Strict).unwrap();
    let bye_edges = graph
        .edges
        .iter()
        .filter(|edge| edge.target == PairingEdgeTarget::Bye)
        .collect::<Vec<_>>();

    assert_eq!(bye_edges.len(), 3);
    let repeated = bye_edges
        .iter()
        .find(|edge| edge.first_entrant_id.as_str() == "a")
        .unwrap();
    assert_eq!(repeated.breakdown.bye_penalty, 200);
}

#[test]
fn graph_edges_are_deterministic_when_request_order_changes() {
    let first = request(
        1,
        vec![
            entrant("c", "three", 1300),
            entrant("a", "one", 1100),
            entrant("b", "two", 1200),
        ],
    );
    let second = request(
        1,
        vec![
            entrant("b", "two", 1200),
            entrant("c", "three", 1300),
            entrant("a", "one", 1100),
        ],
    );

    let first_graph = build_candidate_graph(&first, RelaxationTier::Strict).unwrap();
    let second_graph = build_candidate_graph(&second, RelaxationTier::Strict).unwrap();

    assert_eq!(first_graph.edges, second_graph.edges);
}

#[test]
fn component_overflow_returns_a_typed_error() {
    let mut first = entrant("a", "one", 1500);
    first.performance_score = PerformanceScore::from_scaled(i64::MIN);
    let mut second = entrant("b", "two", 1500);
    second.performance_score = PerformanceScore::from_scaled(i64::MAX);
    let mut request = request(2, vec![first, second]);
    request.policy.performance_score_weight = 2;

    assert!(matches!(
        build_candidate_graph(&request, RelaxationTier::Strict),
        Err(BlossomPairingError::PairingCostOverflow {
            component: PairingCostComponent::PerformanceScoreGap
        })
    ));
}
