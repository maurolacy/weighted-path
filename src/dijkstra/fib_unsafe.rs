use crate::fibonacci::{UnsafeFibonacciHeap, UnsafeNode};

use crate::dijkstra::heap_trait::PriorityQueue;

/// Wrapper around `UnsafeFibonacciHeap` to implement `PriorityQueue` trait.
pub struct UnsafeFibonacciHeapPQ {
    heap: UnsafeFibonacciHeap,
}

impl Default for UnsafeFibonacciHeapPQ {
    fn default() -> Self {
        Self::new()
    }
}

impl UnsafeFibonacciHeapPQ {
    pub fn new() -> Self {
        UnsafeFibonacciHeapPQ {
            heap: UnsafeFibonacciHeap::new(),
        }
    }
}

impl PriorityQueue for UnsafeFibonacciHeapPQ {
    type Handle = *mut UnsafeNode;

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
        // Note: handle is already a pointer, so we pass it directly
        self.heap.decrease_key(*handle, new_key);
    }
}

/// Dijkstra implementation using the unsafe (raw pointer) Fibonacci-heap Dijkstra implementation.
///
/// Potentially much faster on dense graphs with many `decrease_key` operations.
/// This is a thin wrapper around the generic Dijkstra implementation.
pub fn dijkstra_fibonacci_unsafe(
    start: usize,
    end: usize,
    graph: &[Vec<(usize, u32)>],
) -> Vec<usize> {
    crate::dijkstra::generic::dijkstra_generic(start, end, graph, UnsafeFibonacciHeapPQ::new())
}
