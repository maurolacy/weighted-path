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
/// Uses a smaller initial bucket allocation and lets the heap grow dynamically.
pub fn dijkstra_dial(
    start: usize,
    end: usize,
    graph: &[Vec<(usize, u32)>],
) -> (Vec<usize>, Option<u32>) {
    let max_nodes = graph.len();
    // Start with a smaller initial allocation (e.g., 1000 buckets)
    // The heap will grow dynamically via ensure_bucket_capacity as needed
    // This avoids allocating huge arrays upfront (e.g., 1M+ buckets for large graphs)
    let initial_max_distance = 1000.min(max_nodes * 10);
    crate::dijkstra::dijkstra(
        start,
        end,
        graph,
        DialHeap::new(max_nodes, initial_max_distance),
    )
}
