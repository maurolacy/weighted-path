use crate::fibonacci::FibonacciHeap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::dijkstra::heap_trait::PriorityQueue;

// Implement PriorityQueue trait directly on FibonacciHeap
impl PriorityQueue for FibonacciHeap {
    type Handle = Rc<RefCell<crate::fibonacci::Node>>;

    fn insert(&mut self, key: u32, node_id: usize) -> Self::Handle {
        FibonacciHeap::insert(self, key, node_id)
    }

    fn extract_min(&mut self) -> Option<(u32, usize)> {
        FibonacciHeap::extract_min(self)
    }

    fn supports_decrease_key(&self) -> bool {
        true
    }

    fn decrease_key(&mut self, handle: &Self::Handle, new_key: u32) {
        FibonacciHeap::decrease_key(self, handle, new_key);
    }
}

/// Dijkstra implementation using the safe `Rc<RefCell>` Fibonacci heap.
///
/// Memory-safe but slower than the raw-pointer variant due to Rc/RefCell overhead.
/// This is a thin wrapper around the generic Dijkstra implementation.
pub fn dijkstra_fibonacci(
    start: usize,
    end: usize,
    graph: &[Vec<(usize, u32)>],
) -> (Vec<usize>, Option<u32>) {
    crate::dijkstra::dijkstra(start, end, graph, FibonacciHeap::new())
}
