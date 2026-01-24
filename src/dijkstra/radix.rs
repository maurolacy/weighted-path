use crate::dijkstra::heap_trait::PriorityQueue;
use crate::radix::{RadixHandle, RadixHeap};

/// Dijkstra implementation using a radix heap.
///
/// Radix heaps are optimized for algorithms with non-decreasing integer keys,
/// which makes them ideal for Dijkstra's algorithm. They provide O(1) amortized
/// `insert` and `decrease_key`, and O(log C) amortized `extract_min`, where C
/// is the maximum key difference.
///
/// This is a thin wrapper around the generic Dijkstra implementation.
impl PriorityQueue for RadixHeap {
    type Handle = RadixHandle;

    fn insert(&mut self, key: u32, node_id: usize) -> Self::Handle {
        RadixHeap::insert(self, key, node_id)
    }

    fn extract_min(&mut self) -> Option<(u32, usize)> {
        RadixHeap::extract_min(self)
    }

    fn supports_decrease_key(&self) -> bool {
        true
    }

    fn decrease_key(&mut self, handle: &Self::Handle, new_key: u32) {
        RadixHeap::decrease_key(self, handle, new_key);
    }
}

/// Dijkstra implementation using a radix heap.
pub fn dijkstra_radix(start: usize, end: usize, graph: &[Vec<(usize, u32)>]) -> Vec<usize> {
    // Pre-allocate with the number of nodes to avoid resizes
    let max_node_id = graph.len().saturating_sub(1);
    crate::dijkstra::dijkstra(start, end, graph, RadixHeap::with_capacity(max_node_id))
}
