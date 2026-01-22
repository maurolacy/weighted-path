use crate::fibonacci::FibonacciHeap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::dijkstra::heap_trait::PriorityQueue;

/// Wrapper around `FibonacciHeap` to implement `PriorityQueue` trait.
pub struct FibonacciHeapPQ {
    heap: FibonacciHeap,
}

impl Default for FibonacciHeapPQ {
    fn default() -> Self {
        Self::new()
    }
}

impl FibonacciHeapPQ {
    pub fn new() -> Self {
        FibonacciHeapPQ {
            heap: FibonacciHeap::new(),
        }
    }
}

impl PriorityQueue for FibonacciHeapPQ {
    type Handle = Rc<RefCell<crate::fibonacci::Node>>;

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

/// Dijkstra implementation using the safe `Rc<RefCell>` Fibonacci heap.
///
/// Memory-safe but slower than the raw-pointer variant due to Rc/RefCell overhead.
/// This is a thin wrapper around the generic Dijkstra implementation.
pub fn dijkstra_fibonacci(start: usize, end: usize, graph: &[Vec<(usize, u32)>]) -> Vec<usize> {
    crate::dijkstra::generic::dijkstra_generic(start, end, graph, FibonacciHeapPQ::new())
}
