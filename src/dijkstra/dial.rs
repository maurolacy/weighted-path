use crate::dial::{DialHandle, DialHeap};
use crate::dijkstra::heap_trait::PriorityQueue;

/// Dijkstra implementation using Dial's algorithm (bucket-based).
///
/// Dial's algorithm uses buckets indexed by distance values instead of a
/// priority queue. It's very efficient when edge weights are small integers,
/// providing O(V + E + C) time complexity where C is the maximum distance.
///
/// This is a thin wrapper around the generic Dijkstra implementation.
impl PriorityQueue for DialHeap {
    type Handle = DialHandle;

    fn insert(&mut self, key: u32, node_id: usize) -> Self::Handle {
        DialHeap::insert(self, key, node_id)
    }

    fn extract_min(&mut self) -> Option<(u32, usize)> {
        DialHeap::extract_min(self)
    }

    fn supports_decrease_key(&self) -> bool {
        true
    }

    fn decrease_key(&mut self, handle: &Self::Handle, new_key: u32) {
        DialHeap::decrease_key(self, handle, new_key);
    }
}

/// Dijkstra implementation using Dial's algorithm.
///
/// Estimates maximum distance as V * max_edge_weight for bucket pre-allocation.
pub fn dijkstra_dial(start: usize, end: usize, graph: &[Vec<(usize, u32)>]) -> Vec<usize> {
    let max_nodes = graph.len();
    // Estimate max distance: assume worst case path through all nodes with max weight 100
    // This is a conservative estimate; buckets will grow dynamically if needed
    let estimated_max_distance = max_nodes * 100;
    crate::dijkstra::dijkstra(
        start,
        end,
        graph,
        DialHeap::new(max_nodes, estimated_max_distance),
    )
}
