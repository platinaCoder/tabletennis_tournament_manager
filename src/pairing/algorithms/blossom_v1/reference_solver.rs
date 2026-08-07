use std::collections::HashMap;

use crate::identity::EntrantId;

use super::{
    BlossomPairingError, InvalidSolverOutputReason, PairingCandidateGraph, PairingCostComponent,
    PairingEdgeTarget, SolverError,
};

const MAXIMUM_REFERENCE_NODE_COUNT: usize = 20;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ReferenceMatching {
    pub edge_indices: Vec<usize>,
    pub total_cost: u64,
}

pub(super) fn solve_exactly(
    graph: &PairingCandidateGraph,
) -> Result<Option<ReferenceMatching>, BlossomPairingError> {
    let has_bye_node = graph.entrant_ids.len() % 2 == 1;
    let node_count = graph.entrant_ids.len() + usize::from(has_bye_node);
    if node_count > MAXIMUM_REFERENCE_NODE_COUNT {
        return Err(BlossomPairingError::SolverFailure {
            source: SolverError::new(format!(
                "reference solver supports at most {MAXIMUM_REFERENCE_NODE_COUNT} nodes"
            )),
        });
    }

    let entrant_nodes = entrant_node_index(&graph.entrant_ids)?;
    let solver_edges = solver_edges(graph, &entrant_nodes, has_bye_node)?;
    let mut search = Search {
        edges: &solver_edges,
        matched: vec![false; node_count],
        selected_edges: Vec::with_capacity(node_count / 2),
        best: None,
    };
    search.visit(0)?;
    Ok(search.best)
}

fn entrant_node_index(
    entrant_ids: &[EntrantId],
) -> Result<HashMap<&EntrantId, usize>, BlossomPairingError> {
    let mut index = HashMap::with_capacity(entrant_ids.len());
    for (node_index, entrant_id) in entrant_ids.iter().enumerate() {
        if index.insert(entrant_id, node_index).is_some() {
            return Err(BlossomPairingError::InvalidSolverOutput {
                reason: InvalidSolverOutputReason::DuplicateEntrant,
            });
        }
    }
    Ok(index)
}

fn solver_edges(
    graph: &PairingCandidateGraph,
    entrant_nodes: &HashMap<&EntrantId, usize>,
    has_bye_node: bool,
) -> Result<Vec<SolverEdge>, BlossomPairingError> {
    graph
        .edges
        .iter()
        .enumerate()
        .map(|(graph_index, edge)| {
            let first_node = node_for_entrant(&edge.first_entrant_id, entrant_nodes)?;
            let second_node = match &edge.target {
                PairingEdgeTarget::Entrant(entrant_id) => {
                    node_for_entrant(entrant_id, entrant_nodes)?
                }
                PairingEdgeTarget::Bye if has_bye_node => graph.entrant_ids.len(),
                PairingEdgeTarget::Bye => {
                    return Err(BlossomPairingError::InvalidSolverOutput {
                        reason: InvalidSolverOutputReason::UnexpectedBye,
                    });
                }
            };

            if first_node == second_node {
                return Err(BlossomPairingError::InvalidSolverOutput {
                    reason: InvalidSolverOutputReason::SelfPair,
                });
            }

            Ok(SolverEdge {
                first_node,
                second_node,
                cost: edge.cost.value(),
                graph_index,
            })
        })
        .collect()
}

fn node_for_entrant(
    entrant_id: &EntrantId,
    entrant_nodes: &HashMap<&EntrantId, usize>,
) -> Result<usize, BlossomPairingError> {
    entrant_nodes
        .get(entrant_id)
        .copied()
        .ok_or(BlossomPairingError::InvalidSolverOutput {
            reason: InvalidSolverOutputReason::UnknownEdge,
        })
}

struct SolverEdge {
    first_node: usize,
    second_node: usize,
    cost: u64,
    graph_index: usize,
}

struct Search<'a> {
    edges: &'a [SolverEdge],
    matched: Vec<bool>,
    selected_edges: Vec<usize>,
    best: Option<ReferenceMatching>,
}

impl Search<'_> {
    fn visit(&mut self, current_cost: u64) -> Result<(), BlossomPairingError> {
        if self
            .best
            .as_ref()
            .is_some_and(|best| current_cost > best.total_cost)
        {
            return Ok(());
        }

        let Some(first_unmatched) = self.matched.iter().position(|matched| !matched) else {
            self.record_complete_matching(current_cost);
            return Ok(());
        };

        for edge_index in 0..self.edges.len() {
            let edge = &self.edges[edge_index];
            let Some(other_node) = other_endpoint(edge, first_unmatched) else {
                continue;
            };
            if self.matched[other_node] {
                continue;
            }

            let next_cost = current_cost.checked_add(edge.cost).ok_or(
                BlossomPairingError::PairingCostOverflow {
                    component: PairingCostComponent::Total,
                },
            )?;
            self.matched[first_unmatched] = true;
            self.matched[other_node] = true;
            self.selected_edges.push(edge.graph_index);
            self.visit(next_cost)?;
            self.selected_edges.pop();
            self.matched[first_unmatched] = false;
            self.matched[other_node] = false;
        }

        Ok(())
    }

    fn record_complete_matching(&mut self, total_cost: u64) {
        let mut edge_indices = self.selected_edges.clone();
        edge_indices.sort_unstable();
        let candidate = ReferenceMatching {
            edge_indices,
            total_cost,
        };

        let replace = self.best.as_ref().is_none_or(|best| {
            (candidate.total_cost, &candidate.edge_indices) < (best.total_cost, &best.edge_indices)
        });
        if replace {
            self.best = Some(candidate);
        }
    }
}

fn other_endpoint(edge: &SolverEdge, node: usize) -> Option<usize> {
    if edge.first_node == node {
        Some(edge.second_node)
    } else if edge.second_node == node {
        Some(edge.first_node)
    } else {
        None
    }
}
