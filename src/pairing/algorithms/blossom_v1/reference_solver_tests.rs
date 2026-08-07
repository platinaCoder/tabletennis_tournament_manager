use crate::identity::EntrantId;
use crate::pairing::EloRating;

use super::reference_solver::{ReferenceMatching, solve_exactly};
use super::*;

fn edge(first: &str, second: &str, cost: u64) -> PairingCandidateEdge {
    PairingCandidateEdge {
        first_entrant_id: EntrantId::new(first),
        target: PairingEdgeTarget::Entrant(EntrantId::new(second)),
        same_club: false,
        rematch: false,
        cost: PairingCost::new(cost),
        breakdown: breakdown(cost),
    }
}

fn bye_edge(entrant: &str, cost: u64) -> PairingCandidateEdge {
    PairingCandidateEdge {
        first_entrant_id: EntrantId::new(entrant),
        target: PairingEdgeTarget::Bye,
        same_club: false,
        rematch: false,
        cost: PairingCost::new(cost),
        breakdown: breakdown(cost),
    }
}

fn breakdown(total: u64) -> PairingCostBreakdown {
    PairingCostBreakdown {
        performance_score_gap: 0,
        match_win_gap: 0,
        opponent_strength_gap: 0,
        elo_gap: 0,
        same_club_penalty: 0,
        rematch_penalty: 0,
        bye_penalty: 0,
        deterministic_tie_break: 0,
        total,
    }
}

fn graph(ids: &[&str], edges: Vec<PairingCandidateEdge>) -> PairingCandidateGraph {
    PairingCandidateGraph {
        relaxation_tier: RelaxationTier::Strict,
        entrant_ids: ids.iter().map(|id| EntrantId::new(*id)).collect(),
        edges,
        diagnostics: PairingDiagnostics::default(),
    }
}

#[test]
fn exhaustive_oracle_finds_global_minimum_instead_of_greedy_choice() {
    let graph = graph(
        &["a", "b", "c", "d"],
        vec![
            edge("a", "b", 1),
            edge("a", "c", 40),
            edge("a", "d", 50),
            edge("b", "c", 50),
            edge("b", "d", 40),
            edge("c", "d", 100),
        ],
    );

    let matching = solve_exactly(&graph).unwrap().unwrap();

    assert_eq!(matching.total_cost, 80);
    assert_eq!(matching.edge_indices, vec![1, 4]);
}

#[test]
fn exhaustive_oracle_returns_none_when_no_complete_matching_exists() {
    let graph = graph(
        &["a", "b", "c", "d"],
        vec![edge("a", "b", 1), edge("a", "c", 1), edge("b", "c", 1)],
    );

    assert_eq!(solve_exactly(&graph).unwrap(), None);
}

#[test]
fn exhaustive_oracle_matches_exactly_one_entrant_to_the_bye_node() {
    let graph = graph(
        &["a", "b", "c"],
        vec![
            edge("a", "b", 5),
            edge("a", "c", 5),
            edge("b", "c", 1),
            bye_edge("a", 0),
            bye_edge("b", 100),
            bye_edge("c", 100),
        ],
    );

    let matching = solve_exactly(&graph).unwrap().unwrap();

    assert_eq!(matching.total_cost, 1);
    assert_eq!(matching.edge_indices, vec![2, 3]);
}

#[test]
fn equal_cost_matchings_use_deterministic_edge_order() {
    let graph = graph(
        &["a", "b", "c", "d"],
        vec![
            edge("a", "b", 1),
            edge("a", "c", 1),
            edge("b", "d", 1),
            edge("c", "d", 1),
        ],
    );

    assert_eq!(
        solve_exactly(&graph).unwrap(),
        Some(ReferenceMatching {
            edge_indices: vec![0, 3],
            total_cost: 2
        })
    );
}

#[test]
fn matching_total_overflow_is_typed() {
    let graph = graph(
        &["a", "b", "c", "d"],
        vec![edge("a", "b", u64::MAX), edge("c", "d", 1)],
    );

    assert!(matches!(
        solve_exactly(&graph),
        Err(BlossomPairingError::PairingCostOverflow {
            component: PairingCostComponent::Total
        })
    ));
}

#[test]
fn tier_runner_treats_strict_failure_as_expected_relaxation() {
    let request = same_club_request();

    let (tier, matching) = first_complete_tier(&request).unwrap();

    assert_eq!(tier, RelaxationTier::SameClubAllowed);
    assert_eq!(matching.edge_indices.len(), 2);
}

#[test]
fn tier_runner_reaches_rematches_only_after_other_tiers_fail() {
    let mut request = same_club_request();
    request.round_number = RoundNumber::try_from(2_i64).unwrap();
    for (index, entrant) in request.entrants.iter_mut().enumerate() {
        entrant.club_id = ClubId::new(format!("club-{index}"));
    }
    for first in 0..request.entrants.len() {
        for second in first + 1..request.entrants.len() {
            request.previous_matches.push(PreviousMatch {
                first_entrant_id: request.entrants[first].entrant_id.clone(),
                second_entrant_id: request.entrants[second].entrant_id.clone(),
                round_number: RoundNumber::try_from(1_i64).unwrap(),
            });
        }
    }

    let (tier, matching) = first_complete_tier(&request).unwrap();

    assert_eq!(tier, RelaxationTier::RematchesAllowed);
    assert_eq!(matching.edge_indices.len(), 2);
}

fn first_complete_tier(
    request: &PairingRequest,
) -> Result<(RelaxationTier, ReferenceMatching), BlossomPairingError> {
    for tier in RelaxationTier::ORDERED {
        let graph = build_candidate_graph(request, tier)?;
        if let Some(matching) = solve_exactly(&graph)? {
            return Ok((tier, matching));
        }
    }

    Err(BlossomPairingError::NoCompleteMatching {
        final_tier: RelaxationTier::RematchesAllowed,
        unmatched_entrants: request
            .entrants
            .iter()
            .map(|entrant| entrant.entrant_id.clone())
            .collect(),
    })
}

fn same_club_request() -> PairingRequest {
    PairingRequest {
        round_number: RoundNumber::try_from(1_i64).unwrap(),
        entrants: ["a", "b", "c", "d"]
            .into_iter()
            .map(|id| PairingEntrant {
                entrant_id: EntrantId::new(id),
                club_id: ClubId::new("same-club"),
                starting_elo: EloRating::new(1500),
                performance_score: PerformanceScore::ZERO,
                matches_won: 0,
                opponent_score_sum: PerformanceScore::ZERO,
                bye_count: 0,
            })
            .collect(),
        previous_matches: Vec::new(),
        policy: BlossomV1Policy {
            avoid_same_club: true,
            avoid_rematches: true,
            recent_rematch_window: 3,
            performance_score_weight: 1,
            match_win_weight: 1,
            opponent_strength_weight: 1,
            elo_difference_weight: 1,
            bye_repeat_penalty: 100,
            same_club_penalty: 100,
            rematch_penalty: 1_000,
            maximum_entrant_count: 16,
        },
    }
}
