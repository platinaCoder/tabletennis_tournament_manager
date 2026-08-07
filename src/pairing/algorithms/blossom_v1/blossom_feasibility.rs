use crate::identity::EntrantId;

use super::solver_graph::SolverGraph;

const NONE: usize = usize::MAX;

pub(super) struct CardinalityMatching {
    mates: Vec<usize>,
}

impl CardinalityMatching {
    pub fn is_complete(&self) -> bool {
        self.mates.iter().all(|mate| *mate != NONE)
    }

    pub fn unmatched_entrants(&self, entrant_ids: &[EntrantId]) -> Vec<EntrantId> {
        entrant_ids
            .iter()
            .enumerate()
            .filter(|(node, _)| self.mates[*node] == NONE)
            .map(|(_, entrant_id)| entrant_id.clone())
            .collect()
    }
}

pub(super) fn maximum_cardinality_matching(graph: &SolverGraph) -> CardinalityMatching {
    let node_count = graph.node_count();
    let mut search = BlossomSearch {
        graph,
        mates: vec![NONE; node_count],
        parent: vec![NONE; node_count],
        base: (0..node_count).collect(),
        used: vec![false; node_count],
        blossom: vec![false; node_count],
        queue: Vec::with_capacity(node_count),
    };

    for root in 0..node_count {
        if search.mates[root] == NONE {
            search.find_augmenting_path(root);
        }
    }

    CardinalityMatching {
        mates: search.mates,
    }
}

struct BlossomSearch<'a> {
    graph: &'a SolverGraph,
    mates: Vec<usize>,
    parent: Vec<usize>,
    base: Vec<usize>,
    used: Vec<bool>,
    blossom: Vec<bool>,
    queue: Vec<usize>,
}

impl BlossomSearch<'_> {
    fn find_augmenting_path(&mut self, root: usize) -> bool {
        self.used.fill(false);
        self.parent.fill(NONE);
        for (node, base) in self.base.iter_mut().enumerate() {
            *base = node;
        }
        self.queue.clear();
        self.queue.push(root);
        self.used[root] = true;

        let mut queue_index = 0;
        while queue_index < self.queue.len() {
            let vertex = self.queue[queue_index];
            queue_index += 1;

            for neighbour in self.graph.adjacency[vertex].iter().copied() {
                if self.base[vertex] == self.base[neighbour] || self.mates[vertex] == neighbour {
                    continue;
                }

                if neighbour == root
                    || (self.mates[neighbour] != NONE && self.parent[self.mates[neighbour]] != NONE)
                {
                    self.contract_blossom(vertex, neighbour);
                } else if self.parent[neighbour] == NONE {
                    self.parent[neighbour] = vertex;
                    if self.mates[neighbour] == NONE {
                        self.augment(neighbour);
                        return true;
                    }

                    let matched_neighbour = self.mates[neighbour];
                    self.used[matched_neighbour] = true;
                    self.queue.push(matched_neighbour);
                }
            }
        }

        false
    }

    fn contract_blossom(&mut self, first: usize, second: usize) {
        let common_base = self.lowest_common_ancestor(first, second);
        self.blossom.fill(false);
        self.mark_blossom_path(first, common_base, second);
        self.mark_blossom_path(second, common_base, first);

        for node in 0..self.graph.node_count() {
            if self.blossom[self.base[node]] {
                self.base[node] = common_base;
                if !self.used[node] {
                    self.used[node] = true;
                    self.queue.push(node);
                }
            }
        }
    }

    fn lowest_common_ancestor(&self, mut first: usize, mut second: usize) -> usize {
        let mut first_path = vec![false; self.graph.node_count()];
        loop {
            first = self.base[first];
            first_path[first] = true;
            if self.mates[first] == NONE {
                break;
            }
            first = self.parent[self.mates[first]];
        }

        loop {
            second = self.base[second];
            if first_path[second] {
                return second;
            }
            second = self.parent[self.mates[second]];
        }
    }

    fn mark_blossom_path(&mut self, mut vertex: usize, base: usize, mut child: usize) {
        while self.base[vertex] != base {
            self.blossom[self.base[vertex]] = true;
            self.blossom[self.base[self.mates[vertex]]] = true;
            self.parent[vertex] = child;
            child = self.mates[vertex];
            vertex = self.parent[self.mates[vertex]];
        }
    }

    fn augment(&mut self, mut vertex: usize) {
        while vertex != NONE {
            let previous = self.parent[vertex];
            let next = if previous == NONE {
                NONE
            } else {
                self.mates[previous]
            };
            if previous != NONE {
                self.mates[vertex] = previous;
                self.mates[previous] = vertex;
            }
            vertex = next;
        }
    }
}
