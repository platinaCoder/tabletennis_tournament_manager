use super::solver_graph::SolverGraph;
use super::{BlossomPairingError, PairingCandidateGraph, PairingCostComponent};

/// Solver-only projection. Stable entrant IDs and candidate-edge indexes are
/// converted here; node indexes and transformed weights never leave this module.
pub(super) struct WeightedSolverGraph {
    unweighted: SolverGraph,
    edges: Vec<WeightedSolverEdge>,
    weight_offset: u64,
    cardinality_bonus: u128,
    initial_vertex_duals: Vec<u128>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WeightedSolverEdge {
    pub first_node: usize,
    pub second_node: usize,
    pub candidate_edge_index: usize,
    pub maximum_weight: u128,
}

impl WeightedSolverGraph {
    pub fn from_candidate_graph(
        candidate_graph: &PairingCandidateGraph,
    ) -> Result<Self, BlossomPairingError> {
        let unweighted = SolverGraph::from_candidate_graph(candidate_graph)?;
        let maximum_cost = candidate_graph
            .edges
            .iter()
            .map(|edge| edge.cost.value())
            .max()
            .unwrap_or(0);
        let weight_offset =
            maximum_cost
                .checked_add(1)
                .ok_or(BlossomPairingError::PairingCostOverflow {
                    component: PairingCostComponent::SolverWeightProjection,
                })?;
        let required_edge_count = unweighted.node_count() / 2;
        let cardinality_bonus = u128::from(weight_offset)
            .checked_mul(u128_index(required_edge_count)?)
            .and_then(|value| value.checked_add(1))
            .ok_or(BlossomPairingError::PairingCostOverflow {
                component: PairingCostComponent::SolverWeightProjection,
            })?;
        let edges = unweighted
            .edges
            .iter()
            .map(|edge| {
                let cost = candidate_graph.edges[edge.candidate_edge_index]
                    .cost
                    .value();
                let preference_weight = weight_offset.checked_sub(cost).ok_or(
                    BlossomPairingError::PairingCostOverflow {
                        component: PairingCostComponent::SolverWeightProjection,
                    },
                )?;
                let maximum_weight = cardinality_bonus
                    .checked_add(u128::from(preference_weight))
                    .ok_or(BlossomPairingError::PairingCostOverflow {
                        component: PairingCostComponent::SolverWeightProjection,
                    })?;

                Ok(WeightedSolverEdge {
                    first_node: edge.first_node,
                    second_node: edge.second_node,
                    candidate_edge_index: edge.candidate_edge_index,
                    maximum_weight,
                })
            })
            .collect::<Result<Vec<_>, BlossomPairingError>>()?;
        let initial_vertex_duals = vec![maximum_solver_weight(&edges); unweighted.node_count()];

        let graph = Self {
            unweighted,
            edges,
            weight_offset,
            cardinality_bonus,
            initial_vertex_duals,
        };
        debug_assert!(graph.projection_is_consistent(candidate_graph));
        Ok(graph)
    }

    pub fn unweighted(&self) -> &SolverGraph {
        &self.unweighted
    }

    fn projection_is_consistent(&self, candidate_graph: &PairingCandidateGraph) -> bool {
        self.edges.iter().all(|edge| {
            candidate_graph
                .edges
                .get(edge.candidate_edge_index)
                .and_then(|candidate| {
                    u128::from(candidate.cost.value()).checked_add(edge.maximum_weight)
                })
                == u128::from(self.weight_offset).checked_add(self.cardinality_bonus)
                && edge.first_node != edge.second_node
                && self.edge_slack(edge).is_some()
        })
    }

    pub fn edge_slack(&self, edge: &WeightedSolverEdge) -> Option<u128> {
        let first_dual = self.initial_vertex_duals[edge.first_node];
        let second_dual = self.initial_vertex_duals[edge.second_node];
        let doubled_weight = edge.maximum_weight.checked_mul(2)?;
        first_dual
            .checked_add(second_dual)?
            .checked_sub(doubled_weight)
    }

    pub fn edges(&self) -> &[WeightedSolverEdge] {
        &self.edges
    }

    #[cfg(test)]
    pub const fn weight_offset(&self) -> u64 {
        self.weight_offset
    }

    #[cfg(test)]
    pub const fn cardinality_bonus(&self) -> u128 {
        self.cardinality_bonus
    }

    #[cfg(test)]
    pub fn initial_vertex_duals(&self) -> &[u128] {
        &self.initial_vertex_duals
    }

    #[cfg(test)]
    pub fn initial_edge_slacks(&self) -> Vec<u128> {
        self.edges
            .iter()
            .map(|edge| self.edge_slack(edge).unwrap())
            .collect()
    }
}

fn maximum_solver_weight(edges: &[WeightedSolverEdge]) -> u128 {
    edges
        .iter()
        .map(|edge| edge.maximum_weight)
        .max()
        .unwrap_or(0)
}

fn u128_index(value: usize) -> Result<u128, BlossomPairingError> {
    u128::try_from(value).map_err(|_| BlossomPairingError::PairingCostOverflow {
        component: PairingCostComponent::SolverWeightProjection,
    })
}
