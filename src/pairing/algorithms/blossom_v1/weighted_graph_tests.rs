use crate::identity::EntrantId;

use super::weighted_graph::WeightedSolverGraph;
use super::*;

fn edge(first: &str, second: &str, cost: u64) -> PairingCandidateEdge {
    PairingCandidateEdge {
        first_entrant_id: EntrantId::new(first),
        target: PairingEdgeTarget::Entrant(EntrantId::new(second)),
        same_club: false,
        rematch: false,
        cost: PairingCost::new(cost),
        breakdown: PairingCostBreakdown {
            performance_score_gap: 0,
            match_win_gap: 0,
            opponent_strength_gap: 0,
            elo_gap: cost,
            same_club_penalty: 0,
            rematch_penalty: 0,
            bye_penalty: 0,
            deterministic_tie_break: 0,
            total: cost,
        },
    }
}

fn graph(edges: Vec<PairingCandidateEdge>) -> PairingCandidateGraph {
    PairingCandidateGraph {
        relaxation_tier: RelaxationTier::Strict,
        entrant_ids: ["a", "b", "c", "d"]
            .into_iter()
            .map(EntrantId::new)
            .collect(),
        edges,
        diagnostics: PairingDiagnostics::default(),
    }
}

#[test]
fn lower_pairing_cost_projects_to_higher_solver_weight() {
    let graph = graph(vec![edge("a", "b", 3), edge("c", "d", 10)]);
    let weighted = WeightedSolverGraph::from_candidate_graph(&graph).unwrap();

    assert_eq!(weighted.weight_offset(), 11);
    assert_eq!(weighted.cardinality_bonus(), 23);
    assert_eq!(weighted.edges()[0].maximum_weight, 31);
    assert_eq!(weighted.edges()[1].maximum_weight, 24);
}

#[test]
fn projection_preserves_order_between_complete_matchings() {
    let graph = graph(vec![
        edge("a", "b", 1),
        edge("c", "d", 9),
        edge("a", "c", 4),
        edge("b", "d", 5),
    ]);
    let weighted = WeightedSolverGraph::from_candidate_graph(&graph).unwrap();

    let expensive_weight = weighted.edges()[0].maximum_weight + weighted.edges()[1].maximum_weight;
    let cheap_weight = weighted.edges()[2].maximum_weight + weighted.edges()[3].maximum_weight;

    assert!(cheap_weight > expensive_weight);
}

#[test]
fn initial_duals_are_feasible_and_make_maximum_weight_edges_tight() {
    let graph = graph(vec![
        edge("a", "b", 1),
        edge("a", "c", 4),
        edge("b", "d", 3),
        edge("c", "d", 2),
    ]);
    let weighted = WeightedSolverGraph::from_candidate_graph(&graph).unwrap();
    let slacks = weighted.initial_edge_slacks();

    assert_eq!(weighted.initial_vertex_duals().len(), 4);
    assert_eq!(slacks, vec![0, 6, 4, 2]);
}

#[test]
fn cardinality_bonus_makes_two_edges_outweigh_any_single_edge() {
    let graph = graph(vec![edge("a", "b", 0), edge("c", "d", 100)]);
    let weighted = WeightedSolverGraph::from_candidate_graph(&graph).unwrap();

    let both_edges = weighted
        .edges()
        .iter()
        .map(|edge| edge.maximum_weight)
        .sum::<u128>();
    let best_single_edge = weighted
        .edges()
        .iter()
        .map(|edge| edge.maximum_weight)
        .max()
        .unwrap();

    assert!(both_edges > best_single_edge);
}

#[test]
fn projection_overflow_is_typed() {
    let graph = graph(vec![edge("a", "b", u64::MAX), edge("c", "d", 1)]);

    assert!(matches!(
        WeightedSolverGraph::from_candidate_graph(&graph),
        Err(BlossomPairingError::PairingCostOverflow {
            component: PairingCostComponent::SolverWeightProjection
        })
    ));
}
