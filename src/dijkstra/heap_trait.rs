/// Trait for priority queue implementations used in Dijkstra's algorithm.
///
/// This trait abstracts over different heap implementations (binary heap, Fibonacci heap,
/// pairing heap, etc.) to allow a single generic Dijkstra implementation.
pub trait PriorityQueue {
    /// Handle type for nodes in the heap. Used for `decrease_key` operations.
    /// For heaps that don't support `decrease_key`, this can be a unit type `()`.
    type Handle: Clone;

    /// Insert a node with the given key and node ID into the heap.
    /// Returns a handle that can be used for `decrease_key` if supported.
    fn insert(&mut self, key: u32, node_id: usize) -> Self::Handle;

    /// Extract and return the minimum key and node ID from the heap.
    /// Returns `None` if the heap is empty.
    fn extract_min(&mut self) -> Option<(u32, usize)>;

    /// Check if this heap implementation supports `decrease_key` operations.
    /// If `false`, Dijkstra will re-insert nodes instead of decreasing their keys.
    fn supports_decrease_key(&self) -> bool {
        false
    }

    /// Decrease the key of a node identified by its handle.
    /// This method is only called if `supports_decrease_key()` returns `true`.
    /// The implementation should be a no-op if `new_key >= current_key`.
    fn decrease_key(&mut self, handle: &Self::Handle, new_key: u32);
}
