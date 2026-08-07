use super::{
    BlossomPairingError, DenseBlossomKernel, DenseEdge, INNER, OUTER, UNREACHED, invalid_kernel,
};

impl DenseBlossomKernel {
    pub(super) fn set_match(
        &mut self,
        node: usize,
        partner: usize,
    ) -> Result<(), BlossomPairingError> {
        self.mates[node] = self.edges[node][partner].target;
        if node <= self.original_count {
            return Ok(());
        }

        let edge = self.edges[node][partner];
        let entry = self.blossom_origin[node][edge.source];
        let position = self.rotate_entry_to_even_position(node, entry)?;
        for index in 0..position {
            let member = self.blossom_members[node][index];
            let paired = self.blossom_members[node][index ^ 1];
            self.set_match(member, paired)?;
        }
        self.set_match(entry, partner)?;
        self.blossom_members[node].rotate_left(position);
        Ok(())
    }

    pub(super) fn augment(
        &mut self,
        mut node: usize,
        mut partner: usize,
    ) -> Result<(), BlossomPairingError> {
        loop {
            let displaced = self.top_level[self.mates[node]];
            self.set_match(node, partner)?;
            if displaced == 0 {
                return Ok(());
            }
            let parent_top = self.top_level[self.parents[displaced]];
            self.set_match(displaced, parent_top)?;
            node = parent_top;
            partner = displaced;
        }
    }

    pub(super) fn lowest_common_ancestor(
        &mut self,
        mut first: usize,
        mut second: usize,
    ) -> Result<usize, BlossomPairingError> {
        self.visit_clock = self.visit_clock.checked_add(1).ok_or(invalid_kernel())?;
        loop {
            if first != 0 {
                if self.visited_at[first] == self.visit_clock {
                    return Ok(first);
                }
                self.visited_at[first] = self.visit_clock;
                first = self.top_level[self.mates[first]];
                if first != 0 {
                    first = self.top_level[self.parents[first]];
                }
            }
            std::mem::swap(&mut first, &mut second);
            if first == 0 && second == 0 {
                return Ok(0);
            }
        }
    }

    pub(super) fn add_blossom(
        &mut self,
        first: usize,
        base: usize,
        second: usize,
    ) -> Result<(), BlossomPairingError> {
        let blossom = self.allocate_blossom()?;
        self.duals[blossom] = 0;
        self.forest_state[blossom] = OUTER;
        self.mates[blossom] = self.mates[base];
        self.blossom_members[blossom].clear();
        self.blossom_members[blossom].push(base);
        self.append_blossom_path(blossom, first, base)?;
        self.blossom_members[blossom][1..].reverse();
        self.append_blossom_path(blossom, second, base)?;
        self.set_top_level(blossom, blossom);
        self.rebuild_blossom_edges(blossom)?;
        self.set_slack(blossom)
    }

    pub(super) fn expand_blossom(&mut self, blossom: usize) -> Result<(), BlossomPairingError> {
        let members = self.blossom_members[blossom].clone();
        for member in members {
            self.set_top_level(member, member);
        }
        let parent_edge = self.edges[blossom][self.parents[blossom]];
        let entry = self.blossom_origin[blossom][parent_edge.source];
        let position = self.rotate_entry_to_even_position(blossom, entry)?;
        let members = self.blossom_members[blossom].clone();

        for index in (0..position).step_by(2) {
            let inner = members[index];
            let outer = members[index + 1];
            self.parents[inner] = self.edges[outer][inner].source;
            self.forest_state[inner] = INNER;
            self.forest_state[outer] = OUTER;
            self.slack_sources[inner] = 0;
            self.set_slack(outer)?;
            self.queue_push(outer);
        }
        self.forest_state[entry] = INNER;
        self.parents[entry] = self.parents[blossom];
        for member in members.iter().skip(position + 1).copied() {
            self.forest_state[member] = UNREACHED;
            self.set_slack(member)?;
        }
        self.top_level[blossom] = 0;
        Ok(())
    }

    fn allocate_blossom(&mut self) -> Result<usize, BlossomPairingError> {
        let mut blossom = self.original_count + 1;
        while blossom <= self.active_count && self.top_level[blossom] != 0 {
            blossom += 1;
        }
        if blossom > self.active_count {
            self.active_count = self.active_count.checked_add(1).ok_or(invalid_kernel())?;
            blossom = self.active_count;
        }
        if blossom >= self.edges.len() {
            return Err(invalid_kernel());
        }
        Ok(blossom)
    }

    fn append_blossom_path(
        &mut self,
        blossom: usize,
        mut node: usize,
        base: usize,
    ) -> Result<(), BlossomPairingError> {
        while node != base {
            self.blossom_members[blossom].push(node);
            let matched = self.top_level[self.mates[node]];
            if matched == 0 {
                return Err(invalid_kernel());
            }
            self.blossom_members[blossom].push(matched);
            self.queue_push(matched);
            node = self.top_level[self.parents[matched]];
        }
        Ok(())
    }

    fn rebuild_blossom_edges(&mut self, blossom: usize) -> Result<(), BlossomPairingError> {
        for node in 1..=self.active_count {
            self.edges[blossom][node] = DenseEdge::empty(blossom, node);
            self.edges[node][blossom] = DenseEdge::empty(node, blossom);
        }
        self.blossom_origin[blossom].fill(0);
        let members = self.blossom_members[blossom].clone();
        for member in members {
            for node in 1..=self.active_count {
                let candidate = self.edges[member][node];
                let current = self.edges[blossom][node];
                if candidate.weight > 0
                    && (current.weight == 0 || self.edge_precedes(candidate, current)?)
                {
                    self.edges[blossom][node] = candidate;
                    self.edges[node][blossom] = self.edges[node][member];
                }
            }
            for original in 1..=self.original_count {
                if self.blossom_origin[member][original] != 0 {
                    self.blossom_origin[blossom][original] = member;
                }
            }
        }
        Ok(())
    }

    fn edge_precedes(
        &self,
        candidate: DenseEdge,
        current: DenseEdge,
    ) -> Result<bool, BlossomPairingError> {
        Ok((
            self.reduced_cost(candidate)?,
            candidate.candidate_edge_index.unwrap_or(usize::MAX),
        ) < (
            self.reduced_cost(current)?,
            current.candidate_edge_index.unwrap_or(usize::MAX),
        ))
    }

    fn rotate_entry_to_even_position(
        &mut self,
        blossom: usize,
        entry: usize,
    ) -> Result<usize, BlossomPairingError> {
        let position = self.blossom_members[blossom]
            .iter()
            .position(|member| *member == entry)
            .ok_or(invalid_kernel())?;
        if position % 2 == 1 {
            self.blossom_members[blossom][1..].reverse();
            Ok(self.blossom_members[blossom].len() - position)
        } else {
            Ok(position)
        }
    }
}
