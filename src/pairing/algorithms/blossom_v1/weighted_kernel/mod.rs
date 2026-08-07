//! Dense primal-dual weighted Blossom kernel.
//!
//! The kernel owns contraction, expansion, alternating-forest search, and
//! augmentation. Its numeric node and edge indexes never cross this module's
//! boundary; successful output maps back to stable candidate-edge indexes.

mod blossom;
mod graph;
mod stage;

use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};

use super::weighted_graph::WeightedSolverGraph;
use super::{BlossomPairingError, InvalidSolverOutputReason};

const UNREACHED: i8 = -1;
const OUTER: i8 = 0;
const INNER: i8 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct DenseEdge {
    source: usize,
    target: usize,
    weight: u128,
    candidate_edge_index: Option<usize>,
}

impl DenseEdge {
    const fn empty(source: usize, target: usize) -> Self {
        Self {
            source,
            target,
            weight: 0,
            candidate_edge_index: None,
        }
    }
}

struct DenseBlossomKernel {
    original_count: usize,
    active_count: usize,
    edges: Vec<Vec<DenseEdge>>,
    duals: Vec<u128>,
    mates: Vec<usize>,
    slack_sources: Vec<usize>,
    top_level: Vec<usize>,
    parents: Vec<usize>,
    blossom_origin: Vec<Vec<usize>>,
    blossom_members: Vec<Vec<usize>>,
    forest_state: Vec<i8>,
    visited_at: Vec<u64>,
    visit_clock: u64,
    queue: VecDeque<usize>,
}

impl DenseBlossomKernel {
    fn new(graph: &WeightedSolverGraph) -> Result<Self, BlossomPairingError> {
        let original_count = graph.unweighted().node_count();
        let capacity = original_count
            .checked_mul(2)
            .and_then(|value| value.checked_add(3))
            .ok_or(invalid_kernel())?;
        let mut edges = (0..capacity)
            .map(|source| {
                (0..capacity)
                    .map(|target| DenseEdge::empty(source, target))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        for edge in graph.edges() {
            let first = edge.first_node + 1;
            let second = edge.second_node + 1;
            let replace = edge.maximum_weight > edges[first][second].weight
                || (edge.maximum_weight == edges[first][second].weight
                    && edges[first][second]
                        .candidate_edge_index
                        .is_none_or(|current| edge.candidate_edge_index < current));
            if replace {
                edges[first][second] = DenseEdge {
                    source: first,
                    target: second,
                    weight: edge.maximum_weight,
                    candidate_edge_index: Some(edge.candidate_edge_index),
                };
                edges[second][first] = DenseEdge {
                    source: second,
                    target: first,
                    weight: edge.maximum_weight,
                    candidate_edge_index: Some(edge.candidate_edge_index),
                };
            }
        }

        let maximum_weight = graph
            .edges()
            .iter()
            .map(|edge| edge.maximum_weight)
            .max()
            .unwrap_or(0);
        let mut top_level = vec![0; capacity];
        let mut blossom_origin = vec![vec![0; original_count + 1]; capacity];
        for node in 1..=original_count {
            top_level[node] = node;
            blossom_origin[node][node] = node;
        }
        let mut duals = vec![0; capacity];
        duals[1..=original_count].fill(maximum_weight);

        Ok(Self {
            original_count,
            active_count: original_count,
            edges,
            duals,
            mates: vec![0; capacity],
            slack_sources: vec![0; capacity],
            top_level,
            parents: vec![0; capacity],
            blossom_origin,
            blossom_members: vec![Vec::new(); capacity],
            forest_state: vec![UNREACHED; capacity],
            visited_at: vec![0; capacity],
            visit_clock: 0,
            queue: VecDeque::new(),
        })
    }

    fn selected_edges(&self) -> Result<Vec<usize>, BlossomPairingError> {
        let mut selected = Vec::with_capacity(self.original_count / 2);
        for node in 1..=self.original_count {
            let mate = self.mates[node];
            if mate > node {
                selected.push(
                    self.edges[node][mate]
                        .candidate_edge_index
                        .ok_or(invalid_kernel())?,
                );
            }
        }
        selected.sort_unstable();
        selected.dedup();
        Ok(selected)
    }
}

pub(super) fn solve_minimum_cost(
    graph: &WeightedSolverGraph,
) -> Result<Option<Vec<usize>>, BlossomPairingError> {
    catch_unwind(AssertUnwindSafe(|| solve_minimum_cost_inner(graph))).map_err(|_| {
        BlossomPairingError::SolverFailure {
            source: super::SolverError::new("the weighted Blossom kernel panicked"),
        }
    })?
}

fn solve_minimum_cost_inner(
    graph: &WeightedSolverGraph,
) -> Result<Option<Vec<usize>>, BlossomPairingError> {
    let mut kernel = DenseBlossomKernel::new(graph)?;
    while kernel.augment_stage()? {}
    let selected = kernel.selected_edges()?;
    if selected.len() == kernel.original_count / 2 {
        Ok(Some(selected))
    } else {
        Ok(None)
    }
}

const fn invalid_kernel() -> BlossomPairingError {
    BlossomPairingError::InvalidSolverOutput {
        reason: InvalidSolverOutputReason::InvalidBlossomStructure,
    }
}

const fn non_integral_dual_adjustment() -> BlossomPairingError {
    BlossomPairingError::InvalidSolverOutput {
        reason: InvalidSolverOutputReason::NonIntegralDualAdjustment,
    }
}
