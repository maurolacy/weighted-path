use crate::pairing::PairingHeap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::dijkstra::heap_trait::PriorityQueue;

// Implement PriorityQueue trait directly on PairingHeap
impl PriorityQueue for PairingHeap {
    type Handle = Rc<RefCell<crate::pairing::Node>>;

    fn insert(&mut self, key: u32, node_id: usize) -> Self::Handle {
        PairingHeap::insert(self, key, node_id)
    }

    fn extract_min(&mut self) -> Option<(u32, usize)> {
        PairingHeap::extract_min(self)
    }

    fn supports_decrease_key(&self) -> bool {
        true
    }

    fn decrease_key(&mut self, handle: &Self::Handle, new_key: u32) {
        PairingHeap::decrease_key(self, handle, new_key);
    }
}

/// Dijkstra implementation using a Pairing heap.
///
/// Pairing heaps are simpler than Fibonacci heaps but offer similar amortized
/// complexity. In practice, they often outperform Fibonacci heaps due to lower
/// constant factors and simpler operations.
/// This is a thin wrapper around the generic Dijkstra implementation.
pub fn dijkstra_pairing(
    start: usize,
    end: usize,
    graph: &[Vec<(usize, u32)>],
) -> (Vec<usize>, Option<u32>) {
    crate::dijkstra::dijkstra(start, end, graph, PairingHeap::new())
}
