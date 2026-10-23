//! Reachability for the control-flow graph while it is being built.
//!
//! Edges are only added during lowering. Reachability is therefore monotonic:
//! once a block is reached, its outgoing edges never need to be visited again.
//! Edges from unreached blocks wait until their source becomes reachable. This
//! also handles backedges and edges emitted before a block's incoming edge.

use std::collections::{HashMap, HashSet};

use sonatina_ir::BlockId;

pub(super) struct Reachability {
    reached: HashSet<BlockId>,
    pending: HashMap<BlockId, Vec<BlockId>>,
    #[cfg(test)]
    edge_visits: usize,
}

impl Reachability {
    pub(super) fn new(entry: BlockId) -> Self {
        Self {
            reached: HashSet::from([entry]),
            pending: HashMap::new(),
            #[cfg(test)]
            edge_visits: 0,
        }
    }

    pub(super) fn contains(&self, block: BlockId) -> bool {
        self.reached.contains(&block)
    }

    pub(super) fn add_edge(&mut self, source: BlockId, target: BlockId) {
        #[cfg(test)]
        {
            self.edge_visits += 1;
        }
        if !self.contains(source) {
            self.pending.entry(source).or_default().push(target);
            return;
        }

        let mut work = vec![target];
        while let Some(block) = work.pop() {
            if self.reached.insert(block)
                && let Some(successors) = self.pending.remove(&block)
            {
                #[cfg(test)]
                {
                    self.edge_visits += successors.len();
                }
                work.extend(successors);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predecessors_must_be_reachable_and_late_edges_propagate_through_cycles() {
        let mut reachability = Reachability::new(BlockId(0));
        for (source, target) in [(1, 2), (2, 3), (3, 1), (4, 5)] {
            reachability.add_edge(BlockId(source), BlockId(target));
        }
        for block in 1..=5 {
            assert!(!reachability.contains(BlockId(block)));
        }

        reachability.add_edge(BlockId(0), BlockId(1));
        for block in 0..=3 {
            assert!(reachability.contains(BlockId(block)));
        }
        assert!(!reachability.contains(BlockId(5)));
        reachability.add_edge(BlockId(3), BlockId(4));
        assert!(reachability.contains(BlockId(5)));
        assert!(reachability.pending.is_empty());
    }

    #[test]
    fn continuation_queries_visit_each_edge_at_most_twice() {
        for count in [1_000, 10_000] {
            // A sequence of branches, merges and backedges models many switches
            // and loops in one function. Query after every edge, as lowering does.
            let mut reachability = Reachability::new(BlockId(0));
            for index in 0..count {
                let source = index * 3;
                for (from, to) in [
                    (source, source + 1),
                    (source, source + 2),
                    (source + 1, source + 3),
                    (source + 2, source + 3),
                    (source + 3, 0),
                ] {
                    reachability.add_edge(BlockId(from), BlockId(to));
                    assert!(reachability.contains(BlockId(to)));
                }
            }
            assert_eq!(reachability.edge_visits, count as usize * 5);

            // Even adversarial insertion order propagates a newly reached chain
            // once, without rescanning its already reached prefix for each edge.
            let mut reachability = Reachability::new(BlockId(0));
            for index in 1..count {
                reachability.add_edge(BlockId(index), BlockId(index + 1));
                assert!(!reachability.contains(BlockId(index + 1)));
            }
            reachability.add_edge(BlockId(0), BlockId(1));
            assert!(reachability.contains(BlockId(count)));
            assert_eq!(reachability.edge_visits, count as usize * 2 - 1);
            assert!(reachability.pending.is_empty());
        }
    }
}
