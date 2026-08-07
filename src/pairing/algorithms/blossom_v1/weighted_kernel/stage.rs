use super::{
    BlossomPairingError, DenseBlossomKernel, DenseEdge, INNER, OUTER, UNREACHED, invalid_kernel,
    non_integral_dual_adjustment,
};

impl DenseBlossomKernel {
    pub(super) fn augment_stage(&mut self) -> Result<bool, BlossomPairingError> {
        self.begin_forest();
        if self.queue.is_empty() {
            return Ok(false);
        }

        loop {
            while let Some(source) = self.queue.pop_front() {
                if self.forest_state[self.top_level[source]] == INNER {
                    continue;
                }
                for target in 1..=self.original_count {
                    let edge = self.edges[source][target];
                    if edge.weight == 0 || self.top_level[source] == self.top_level[target] {
                        continue;
                    }
                    if self.nonnegative_reduced_cost(edge)? == 0 {
                        if self.process_tight_edge(edge)? {
                            return Ok(true);
                        }
                    } else {
                        self.update_slack(source, self.top_level[target])?;
                    }
                }
            }

            let Some(delta) = self.next_delta()? else {
                return Ok(false);
            };
            if !self.apply_delta(delta)? {
                return Ok(false);
            }
            self.queue.clear();

            for node in 1..=self.active_count {
                let slack_source = self.slack_sources[node];
                if self.top_level[node] == node
                    && slack_source != 0
                    && self.top_level[slack_source] != node
                {
                    let edge = self.edges[slack_source][node];
                    if self.nonnegative_reduced_cost(edge)? == 0 && self.process_tight_edge(edge)? {
                        return Ok(true);
                    }
                }
            }
            for blossom in self.original_count + 1..=self.active_count {
                if self.top_level[blossom] == blossom
                    && self.forest_state[blossom] == INNER
                    && self.duals[blossom] == 0
                {
                    self.expand_blossom(blossom)?;
                }
            }
        }
    }

    fn begin_forest(&mut self) {
        for node in 1..=self.active_count {
            self.forest_state[node] = UNREACHED;
            self.slack_sources[node] = 0;
        }
        self.queue.clear();
        for node in 1..=self.active_count {
            if self.top_level[node] == node && self.mates[node] == 0 {
                self.parents[node] = 0;
                self.forest_state[node] = OUTER;
                self.queue_push(node);
            }
        }
    }

    fn process_tight_edge(&mut self, edge: DenseEdge) -> Result<bool, BlossomPairingError> {
        let first = self.top_level[edge.source];
        let second = self.top_level[edge.target];
        match self.forest_state[second] {
            UNREACHED => {
                self.parents[second] = edge.source;
                self.forest_state[second] = INNER;
                let matched = self.top_level[self.mates[second]];
                if matched == 0 {
                    return Err(invalid_kernel());
                }
                self.slack_sources[second] = 0;
                self.slack_sources[matched] = 0;
                self.forest_state[matched] = OUTER;
                self.queue_push(matched);
            }
            OUTER => {
                let base = self.lowest_common_ancestor(first, second)?;
                if base == 0 {
                    self.augment(first, second)?;
                    self.augment(second, first)?;
                    return Ok(true);
                }
                self.add_blossom(first, base, second)?;
            }
            INNER => {}
            _ => return Err(invalid_kernel()),
        }
        Ok(false)
    }

    fn next_delta(&self) -> Result<Option<u128>, BlossomPairingError> {
        let mut delta = None;
        for node in 1..=self.original_count {
            if self.forest_state[self.top_level[node]] == OUTER {
                update_minimum(&mut delta, self.duals[node]);
            }
        }
        for blossom in self.original_count + 1..=self.active_count {
            if self.top_level[blossom] == blossom && self.forest_state[blossom] == INNER {
                update_minimum(&mut delta, half_even(self.duals[blossom])?);
            }
        }
        for node in 1..=self.active_count {
            let source = self.slack_sources[node];
            if self.top_level[node] != node || source == 0 {
                continue;
            }
            let reduced_cost = self.nonnegative_reduced_cost(self.edges[source][node])?;
            match self.forest_state[node] {
                UNREACHED => update_minimum(&mut delta, reduced_cost),
                OUTER => update_minimum(&mut delta, half_even(reduced_cost)?),
                INNER => {}
                _ => return Err(invalid_kernel()),
            }
        }
        Ok(delta)
    }

    fn apply_delta(&mut self, delta: u128) -> Result<bool, BlossomPairingError> {
        if (1..=self.original_count).any(|node| {
            self.forest_state[self.top_level[node]] == OUTER && self.duals[node] == delta
        }) {
            return Ok(false);
        }
        for node in 1..=self.original_count {
            match self.forest_state[self.top_level[node]] {
                OUTER => {
                    self.duals[node] = self.duals[node]
                        .checked_sub(delta)
                        .ok_or(invalid_kernel())?;
                }
                INNER => {
                    self.duals[node] = self.duals[node]
                        .checked_add(delta)
                        .ok_or(invalid_kernel())?;
                }
                _ => {}
            }
        }
        let doubled = delta.checked_mul(2).ok_or(invalid_kernel())?;
        for blossom in self.original_count + 1..=self.active_count {
            if self.top_level[blossom] != blossom {
                continue;
            }
            match self.forest_state[blossom] {
                OUTER => {
                    self.duals[blossom] = self.duals[blossom]
                        .checked_add(doubled)
                        .ok_or(invalid_kernel())?;
                }
                INNER => {
                    self.duals[blossom] = self.duals[blossom]
                        .checked_sub(doubled)
                        .ok_or(invalid_kernel())?;
                }
                _ => {}
            }
        }
        Ok(true)
    }
}

fn update_minimum(current: &mut Option<u128>, candidate: u128) {
    if current.is_none_or(|value| candidate < value) {
        *current = Some(candidate);
    }
}

fn half_even(value: u128) -> Result<u128, BlossomPairingError> {
    if value.is_multiple_of(2) {
        Ok(value / 2)
    } else {
        Err(non_integral_dual_adjustment())
    }
}
