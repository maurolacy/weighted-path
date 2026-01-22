use crate::pairing::PairingHeap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::dijkstra::heap_trait::PriorityQueue;

/// Wrapper around `PairingHeap` to implement `PriorityQueue` trait.
pub struct PairingHeapPQ {
    heap: PairingHeap,
}

impl Default for PairingHeapPQ {
    fn default() -> Self {
        Self::new()
    }
}

impl PairingHeapPQ {
    pub fn new() -> Self {
        PairingHeapPQ {
            heap: PairingHeap::new(),
        }
    }
}

impl PriorityQueue for PairingHeapPQ {
    type Handle = Rc<RefCell<crate::pairing::Node>>;

    fn insert(&mut self, key: u32, node_id: usize) -> Self::Handle {
        self.heap.insert(key, node_id)
    }

    fn extract_min(&mut self) -> Option<(u32, usize)> {
        self.heap.extract_min()
    }

    fn supports_decrease_key(&self) -> bool {
        true
    }

    fn decrease_key(&mut self, handle: &Self::Handle, new_key: u32) {
        self.heap.decrease_key(handle, new_key);
    }
}

/// Dijkstra implementation using a Pairing heap.
///
/// Pairing heaps are simpler than Fibonacci heaps but offer similar amortized
/// complexity. In practice, they often outperform Fibonacci heaps due to lower
/// constant factors and simpler operations.
/// This is a thin wrapper around the generic Dijkstra implementation.
pub fn dijkstra_pairing(start: usize, end: usize, graph: &[Vec<(usize, u32)>]) -> Vec<usize> {
    crate::dijkstra::generic::dijkstra_generic(start, end, graph, PairingHeapPQ::new())
}
