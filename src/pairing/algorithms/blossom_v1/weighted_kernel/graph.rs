use super::{
    BlossomPairingError, DenseBlossomKernel, DenseEdge, InvalidSolverOutputReason, OUTER,
    invalid_kernel,
};

impl DenseBlossomKernel {
    pub(super) fn reduced_cost(&self, edge: DenseEdge) -> Result<i128, BlossomPairingError> {
        let dual_sum = self.duals[edge.source]
            .checked_add(self.duals[edge.target])
            .ok_or(invalid_kernel())?;
        let doubled_weight = edge.weight.checked_mul(2).ok_or(invalid_kernel())?;
        i128::try_from(dual_sum)
            .ok()
            .and_then(|dual| {
                i128::try_from(doubled_weight)
                    .ok()
                    .and_then(|weight| dual.checked_sub(weight))
            })
            .ok_or(invalid_kernel())
    }

    pub(super) fn nonnegative_reduced_cost(
        &self,
        edge: DenseEdge,
    ) -> Result<u128, BlossomPairingError> {
        let reduced = self.reduced_cost(edge)?;
        let dual_sum = self.duals[edge.source]
            .checked_add(self.duals[edge.target])
            .ok_or(invalid_kernel())?;
        let doubled_weight = edge.weight.checked_mul(2).ok_or(invalid_kernel())?;
        u128::try_from(reduced).map_err(|_| BlossomPairingError::InvalidSolverOutput {
            reason: InvalidSolverOutputReason::NegativeReducedCost {
                first_node: edge.source,
                second_node: edge.target,
                dual_sum,
                doubled_weight,
            },
        })
    }

    pub(super) fn update_slack(
        &mut self,
        source: usize,
        target: usize,
    ) -> Result<(), BlossomPairingError> {
        let current = self.slack_sources[target];
        if current == 0 || self.edge_key(source, target)? < self.edge_key(current, target)? {
            self.slack_sources[target] = source;
        }
        Ok(())
    }

    pub(super) fn set_slack(&mut self, target: usize) -> Result<(), BlossomPairingError> {
        self.slack_sources[target] = 0;
        for source in 1..=self.original_count {
            if self.edges[source][target].weight > 0
                && self.top_level[source] != target
                && self.forest_state[self.top_level[source]] == OUTER
            {
                self.update_slack(source, target)?;
            }
        }
        Ok(())
    }

    pub(super) fn queue_push(&mut self, node: usize) {
        if node <= self.original_count {
            self.queue.push_back(node);
        } else {
            let members = self.blossom_members[node].clone();
            for member in members {
                self.queue_push(member);
            }
        }
    }

    pub(super) fn set_top_level(&mut self, node: usize, blossom: usize) {
        self.top_level[node] = blossom;
        if node > self.original_count {
            let members = self.blossom_members[node].clone();
            for member in members {
                self.set_top_level(member, blossom);
            }
        }
    }

    fn edge_key(&self, source: usize, target: usize) -> Result<(i128, usize), BlossomPairingError> {
        let edge = self.edges[source][target];
        Ok((
            self.reduced_cost(edge)?,
            edge.candidate_edge_index.unwrap_or(usize::MAX),
        ))
    }
}
