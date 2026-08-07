use std::collections::HashMap;

use crate::identity::EntrantId;

use super::{
    BlossomPairingError, InvalidSolverOutputReason, PairingCandidateGraph, PairingEdgeTarget,
};

pub(super) struct SolverGraph {
    pub entrant_ids: Vec<EntrantId>,
    pub adjacency: Vec<Vec<usize>>,
    pub edges: Vec<SolverEdge>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SolverEdge {
    pub first_node: usize,
    pub second_node: usize,
    pub candidate_edge_index: usize,
}

impl SolverGraph {
    pub fn from_candidate_graph(
        candidate_graph: &PairingCandidateGraph,
    ) -> Result<Self, BlossomPairingError> {
        let entrant_nodes = entrant_node_index(&candidate_graph.entrant_ids)?;
        let has_bye_node = candidate_graph.entrant_ids.len() % 2 == 1;
        let node_count = candidate_graph.entrant_ids.len() + usize::from(has_bye_node);
        let mut adjacency = vec![Vec::new(); node_count];
        let mut edges = Vec::with_capacity(candidate_graph.edges.len());

        for (candidate_edge_index, edge) in candidate_graph.edges.iter().enumerate() {
            let first = node_for_entrant(&edge.first_entrant_id, &entrant_nodes)?;
            let second = match &edge.target {
                PairingEdgeTarget::Entrant(entrant_id) => {
                    node_for_entrant(entrant_id, &entrant_nodes)?
                }
                PairingEdgeTarget::Bye if has_bye_node => candidate_graph.entrant_ids.len(),
                PairingEdgeTarget::Bye => {
                    return Err(BlossomPairingError::InvalidSolverOutput {
                        reason: InvalidSolverOutputReason::UnexpectedBye,
                    });
                }
            };

            if first == second {
                return Err(BlossomPairingError::InvalidSolverOutput {
                    reason: InvalidSolverOutputReason::SelfPair,
                });
            }

            if !adjacency[first].contains(&second) {
                adjacency[first].push(second);
                adjacency[second].push(first);
                edges.push(SolverEdge {
                    first_node: first,
                    second_node: second,
                    candidate_edge_index,
                });
            }
        }

        for neighbours in &mut adjacency {
            neighbours.sort_unstable();
        }

        Ok(Self {
            entrant_ids: candidate_graph.entrant_ids.clone(),
            adjacency,
            edges,
        })
    }

    pub fn node_count(&self) -> usize {
        self.adjacency.len()
    }
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
