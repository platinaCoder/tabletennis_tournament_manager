use crate::identity::EntrantId;

use super::reference_solver::solve_exactly;
use super::weighted_graph::WeightedSolverGraph;
use super::weighted_kernel::solve_minimum_cost;
use super::*;

fn edge(first: usize, second: usize, cost: u64) -> PairingCandidateEdge {
    PairingCandidateEdge {
        first_entrant_id: EntrantId::new(format!("entrant-{first}")),
        target: PairingEdgeTarget::Entrant(EntrantId::new(format!("entrant-{second}"))),
        same_club: false,
        rematch: false,
        cost: PairingCost::new(cost),
        breakdown: breakdown(cost),
    }
}

fn bye_edge(entrant: usize, cost: u64) -> PairingCandidateEdge {
    PairingCandidateEdge {
        first_entrant_id: EntrantId::new(format!("entrant-{entrant}")),
        target: PairingEdgeTarget::Bye,
        same_club: false,
        rematch: false,
        cost: PairingCost::new(cost),
        breakdown: breakdown(cost),
    }
}

fn breakdown(cost: u64) -> PairingCostBreakdown {
    PairingCostBreakdown {
        performance_score_gap: 0,
        match_win_gap: 0,
        opponent_strength_gap: 0,
        elo_gap: cost,
        same_club_penalty: 0,
        rematch_penalty: 0,
        bye_penalty: 0,
        deterministic_tie_break: 0,
        total: cost,
    }
}

fn graph(node_count: usize, edges: Vec<PairingCandidateEdge>) -> PairingCandidateGraph {
    PairingCandidateGraph {
        relaxation_tier: RelaxationTier::Strict,
        entrant_ids: (0..node_count)
            .map(|node| EntrantId::new(format!("entrant-{node}")))
            .collect(),
        edges,
        diagnostics: PairingDiagnostics::default(),
    }
}

fn assert_matches_oracle(graph: &PairingCandidateGraph) {
    let weighted = WeightedSolverGraph::from_candidate_graph(graph).unwrap();
    let actual = solve_minimum_cost(&weighted)
        .unwrap_or_else(|error| panic!("kernel failed for {graph:?}: {error}"));
    let oracle = solve_exactly(graph).unwrap();

    assert_eq!(actual.is_some(), oracle.is_some());
    if let (Some(selected), Some(oracle)) = (actual, oracle) {
        let actual_cost = selected
            .iter()
            .map(|index| graph.edges[*index].cost.value())
            .sum::<u64>();
        assert_eq!(actual_cost, oracle.total_cost);
    }
}

#[test]
fn weighted_kernel_contracts_an_odd_cycle() {
    let graph = graph(
        4,
        vec![
            edge(1, 2, 0),
            edge(0, 1, 0),
            edge(0, 2, 0),
            edge(0, 3, 1),
            edge(1, 3, 1),
            edge(2, 3, 1),
        ],
    );

    assert_matches_oracle(&graph);
}

#[test]
fn weighted_kernel_handles_a_synthetic_bye_node() {
    let graph = graph(
        3,
        vec![
            edge(0, 1, 4),
            edge(0, 2, 2),
            edge(1, 2, 1),
            bye_edge(0, 0),
            bye_edge(1, 20),
            bye_edge(2, 20),
        ],
    );

    assert_matches_oracle(&graph);
}

#[test]
fn weighted_kernel_matches_oracle_for_every_four_node_ternary_cost_graph() {
    let endpoints = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    let graph_count = 3_u64.pow(endpoints.len() as u32);
    for encoded_costs in 0..graph_count {
        let mut remaining = encoded_costs;
        let edges = endpoints
            .iter()
            .map(|(first, second)| {
                let cost = remaining % 3;
                remaining /= 3;
                edge(*first, *second, cost)
            })
            .collect();
        assert_matches_oracle(&graph(4, edges));
    }
}

#[test]
fn weighted_kernel_matches_oracle_on_generated_six_node_graphs() {
    let endpoints = (0..6)
        .flat_map(|first| (first + 1..6).map(move |second| (first, second)))
        .collect::<Vec<_>>();
    let mut state = 0x5eed_u64;
    for _ in 0..300 {
        let edges = endpoints
            .iter()
            .filter_map(|(first, second)| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                (!state.is_multiple_of(5)).then(|| edge(*first, *second, (state >> 16) % 31))
            })
            .collect();
        assert_matches_oracle(&graph(6, edges));
    }
}

#[test]
fn weighted_kernel_matches_oracle_for_every_six_node_binary_cost_graph() {
    let endpoints = (0..6)
        .flat_map(|first| (first + 1..6).map(move |second| (first, second)))
        .collect::<Vec<_>>();
    let graph_count = 1_u64 << endpoints.len();
    for encoded_costs in 0..graph_count {
        let edges = endpoints
            .iter()
            .enumerate()
            .map(|(bit, (first, second))| edge(*first, *second, (encoded_costs >> bit) & 1))
            .collect();
        assert_matches_oracle(&graph(6, edges));
    }
}

#[test]
fn weighted_kernel_matches_oracle_on_generated_odd_fields() {
    let endpoints = (0..5)
        .flat_map(|first| (first + 1..5).map(move |second| (first, second)))
        .collect::<Vec<_>>();
    let mut state = 0xb10_550_u64;
    for _ in 0..500 {
        let mut edges = endpoints
            .iter()
            .filter_map(|(first, second)| {
                state = state
                    .wrapping_mul(2_862_933_555_777_941_757)
                    .wrapping_add(3_037_000_493);
                (!state.is_multiple_of(4)).then(|| edge(*first, *second, (state >> 19) % 41))
            })
            .collect::<Vec<_>>();
        edges.extend((0..5).map(|entrant| {
            state = state
                .wrapping_mul(2_862_933_555_777_941_757)
                .wrapping_add(3_037_000_493);
            bye_edge(entrant, (state >> 23) % 41)
        }));
        assert_matches_oracle(&graph(5, edges));
    }
}

#[test]
fn weighted_kernel_matches_oracle_on_larger_generated_graphs() {
    let mut state = 0xdeca_fbad_u64;
    for (node_count, samples) in [(8, 1_000), (10, 200)] {
        let endpoints = (0..node_count)
            .flat_map(|first| (first + 1..node_count).map(move |second| (first, second)))
            .collect::<Vec<_>>();
        for _ in 0..samples {
            let edges = endpoints
                .iter()
                .filter_map(|(first, second)| {
                    state = state
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1_442_695_040_888_963_407);
                    (!state.is_multiple_of(6)).then(|| edge(*first, *second, (state >> 17) % 101))
                })
                .collect();
            assert_matches_oracle(&graph(node_count, edges));
        }
    }
}
