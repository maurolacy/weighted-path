use crate::fibonacci::{UnsafeFibonacciHeap, UnsafeNode};

use crate::dijkstra::heap_trait::PriorityQueue;

// Implement PriorityQueue trait directly on UnsafeFibonacciHeap
impl PriorityQueue for UnsafeFibonacciHeap {
    type Handle = *mut UnsafeNode;

    fn insert(&mut self, key: u32, node_id: usize) -> Self::Handle {
        UnsafeFibonacciHeap::insert(self, key, node_id)
    }

    fn extract_min(&mut self) -> Option<(u32, usize)> {
        UnsafeFibonacciHeap::extract_min(self)
    }

    fn supports_decrease_key(&self) -> bool {
        true
    }

    fn decrease_key(&mut self, handle: &Self::Handle, new_key: u32) {
        // Note: handle is already a pointer, so we pass it directly
        UnsafeFibonacciHeap::decrease_key(self, *handle, new_key);
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
    crate::dijkstra::generic::dijkstra_generic(start, end, graph, UnsafeFibonacciHeap::new())
}
