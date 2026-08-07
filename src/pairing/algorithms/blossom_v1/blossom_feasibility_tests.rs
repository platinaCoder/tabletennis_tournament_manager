use crate::identity::EntrantId;
use crate::pairing::EloRating;

use super::blossom_feasibility::maximum_cardinality_matching;
use super::reference_solver::solve_exactly;
use super::solver_graph::SolverGraph;
use super::*;

#[test]
fn blossom_feasibility_matches_oracle_for_every_graph_through_six_nodes() {
    for node_count in [2, 4, 6] {
        let possible_edges = possible_edges(node_count);
        let graph_count = 1_u64 << possible_edges.len();

        for edge_mask in 0..graph_count {
            let graph = graph_from_mask(node_count, &possible_edges, edge_mask);
            let solver_graph = SolverGraph::from_candidate_graph(&graph).unwrap();
            let blossom_complete = maximum_cardinality_matching(&solver_graph).is_complete();
            let oracle_complete = solve_exactly(&graph).unwrap().is_some();

            assert_eq!(
                blossom_complete, oracle_complete,
                "node_count={node_count}, edge_mask={edge_mask:#b}"
            );
        }
    }
}

#[test]
fn production_relaxation_selects_same_club_tier_after_strict_failure() {
    let request = request_with_same_club_entrants();

    let graph = build_first_feasible_graph(&request).unwrap();

    assert_eq!(graph.relaxation_tier, RelaxationTier::SameClubAllowed);
    assert!(graph.diagnostics.eligible_edge_count > 0);
}

#[test]
fn production_relaxation_stays_strict_when_strict_graph_is_feasible() {
    let mut request = request_with_same_club_entrants();
    for (index, entrant) in request.entrants.iter_mut().enumerate() {
        entrant.club_id = ClubId::new(format!("club-{index}"));
    }

    let graph = build_first_feasible_graph(&request).unwrap();

    assert_eq!(graph.relaxation_tier, RelaxationTier::Strict);
}

fn possible_edges(node_count: usize) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for first in 0..node_count {
        for second in first + 1..node_count {
            edges.push((first, second));
        }
    }
    edges
}

fn graph_from_mask(
    node_count: usize,
    possible_edges: &[(usize, usize)],
    edge_mask: u64,
) -> PairingCandidateGraph {
    let entrant_ids = (0..node_count)
        .map(|node| EntrantId::new(format!("entrant-{node}")))
        .collect::<Vec<_>>();
    let edges = possible_edges
        .iter()
        .enumerate()
        .filter(|(edge_index, _)| edge_mask & (1_u64 << edge_index) != 0)
        .map(|(_, (first, second))| PairingCandidateEdge {
            first_entrant_id: entrant_ids[*first].clone(),
            target: PairingEdgeTarget::Entrant(entrant_ids[*second].clone()),
            same_club: false,
            rematch: false,
            cost: PairingCost::new(1),
            breakdown: PairingCostBreakdown {
                performance_score_gap: 0,
                match_win_gap: 0,
                opponent_strength_gap: 0,
                elo_gap: 1,
                same_club_penalty: 0,
                rematch_penalty: 0,
                bye_penalty: 0,
                deterministic_tie_break: 0,
                total: 1,
            },
        })
        .collect();

    PairingCandidateGraph {
        relaxation_tier: RelaxationTier::Strict,
        entrant_ids,
        edges,
        diagnostics: PairingDiagnostics::default(),
    }
}

fn request_with_same_club_entrants() -> PairingRequest {
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
