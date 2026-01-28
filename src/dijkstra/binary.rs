use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::dijkstra::heap_trait::PriorityQueue;

/// Wrapper around `BinaryHeap` to implement `PriorityQueue` trait.
///
/// `BinaryHeap` is a max-heap, so we use `Reverse` to make it a min-heap.
/// This implementation doesn't support `decrease_key`, so nodes are re-inserted
/// when their distance decreases (which is why we skip duplicates in Dijkstra).
pub struct BinaryHeapPQ {
    heap: BinaryHeap<Reverse<(u32, usize)>>,
}

impl Default for BinaryHeapPQ {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryHeapPQ {
    pub fn new() -> Self {
        BinaryHeapPQ {
            heap: BinaryHeap::new(),
        }
    }
}

impl PriorityQueue for BinaryHeapPQ {
    type Handle = ();

    fn insert(&mut self, key: u32, node_id: usize) -> Self::Handle {
        self.heap.push(Reverse((key, node_id)));
    }

    fn extract_min(&mut self) -> Option<(u32, usize)> {
        self.heap
            .pop()
            .map(|Reverse((key, node_id))| (key, node_id))
    }

    fn supports_decrease_key(&self) -> bool {
        false
    }

    fn decrease_key(&mut self, _handle: &Self::Handle, _new_key: u32) {
        // BinaryHeap doesn't support decrease_key, so this is never called
        unreachable!("BinaryHeap doesn't support decrease_key")
    }
}

/// Dijkstra implementation using a binary heap.
///
/// This is a thin wrapper around the generic Dijkstra implementation.
pub fn dijkstra_binary(
    start: usize,
    end: usize,
    graph: &[Vec<(usize, u32)>],
) -> (Vec<usize>, Option<u32>) {
    crate::dijkstra::dijkstra(start, end, graph, BinaryHeapPQ::new())
}
