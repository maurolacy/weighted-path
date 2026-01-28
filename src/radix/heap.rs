/// Radix heap implementation for priority queues with non-decreasing integer keys.
///
/// A radix heap is a specialized priority queue that exploits the property that
/// extracted keys are monotonically non-decreasing (i.e., each `extract_min` returns
/// a key >= the previous one). This makes it very efficient for algorithms like
/// Dijkstra's algorithm.
///
/// The heap organizes elements into buckets based on the number of bits needed to
/// represent the delta between their key and the current minimum key. This allows
/// for O(1) amortized `insert` and `decrease_key`, and O(log C) amortized `extract_min`,
/// where C is the maximum difference between keys.
///
/// # Performance
///
/// - **Insert**: O(1) amortized
/// - **Extract-min**: O(log C) amortized, where C is the key range
/// - **Decrease-key**: O(1) amortized
///
/// In practice, radix heaps often outperform Fibonacci heaps for Dijkstra's algorithm,
/// especially when edge weights are bounded integers.

#[derive(Clone, Debug)]
struct Node {
    key: u32,
    node_id: usize,
}

/// Handle to a node in the radix heap, used for decrease_key operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RadixHandle {
    node_id: usize,
}

/// Radix heap implementation.
///
/// The heap maintains a set of buckets, where bucket `i` contains nodes whose key
/// delta from the current minimum requires `i` bits to represent. When we extract
/// the minimum, we redistribute nodes from the first non-empty bucket into lower buckets.
pub struct RadixHeap {
    /// Buckets: bucket[i] contains nodes where the key delta needs i bits.
    /// We use Vec with swap_remove for O(1) removal from any position.
    buckets: Vec<Vec<Node>>,
    /// Current minimum key (the last extracted key, or 0 initially).
    min_key: u32,
    /// Total number of elements in the heap.
    size: usize,
    /// Direct mapping from node_id to (bucket_index, position_in_bucket) for O(1) decrease_key.
    /// Uses Vec instead of HashMap for better performance when node IDs are dense (0..V-1).
    /// None means the node is not in the heap.
    node_positions: Vec<Option<(usize, usize)>>,
}

impl RadixHeap {
    /// Create a new empty radix heap.
    pub fn new() -> Self {
        // u32 has 32 bits, so we need at most 33 buckets (0..=32)
        // to represent any key delta
        let max_buckets = 33;
        RadixHeap {
            buckets: vec![Vec::new(); max_buckets],
            min_key: 0,
            size: 0,
            node_positions: Vec::new(),
        }
    }

    /// Create a new empty radix heap with pre-allocated capacity for node positions.
    ///
    /// `max_node_id` should be the maximum node ID that will be inserted (typically V-1
    /// for a graph with V nodes, where node IDs are 0..V-1). This pre-allocates the
    /// `node_positions` Vec to avoid resizes during execution.
    pub fn with_capacity(max_node_id: usize) -> Self {
        // u32 has 32 bits, so we need at most 33 buckets (0..=32)
        // to represent any key delta
        let max_buckets = 33;
        RadixHeap {
            buckets: vec![Vec::new(); max_buckets],
            min_key: 0,
            size: 0,
            node_positions: vec![None; max_node_id + 1],
        }
    }

    /// Get the bucket index for a given key.
    ///
    /// The bucket index is the number of bits needed to represent (key ^ min_key),
    /// or 0 if key == min_key.
    fn bucket_index(&self, key: u32) -> usize {
        // By radix-heap invariant we must have key >= min_key here.
        // If this is violated, the heap's internal state is already inconsistent.
        debug_assert!(key >= self.min_key);
        let delta = key ^ self.min_key;
        // Find the position of the most significant bit.
        // This is the number of bits needed to represent the delta between key and min_key
        (32 - delta.leading_zeros()) as usize
    }

    /// Insert a node with the given key and node ID.
    pub fn insert(&mut self, key: u32, node_id: usize) -> RadixHandle {
        let bucket_idx = self.bucket_index(key);
        let pos = self.buckets[bucket_idx].len();
        self.buckets[bucket_idx].push(Node { key, node_id });

        // Resize if needed, then access directly (no bounds check)
        if node_id >= self.node_positions.len() {
            self.node_positions.resize(node_id + 1, None);
        }
        self.node_positions[node_id] = Some((bucket_idx, pos));
        self.size += 1;
        RadixHandle { node_id }
    }

    /// Extract and return the minimum key and node ID.
    ///
    /// This operation redistributes nodes from the first non-empty bucket into
    /// lower buckets based on their new key deltas.
    pub fn extract_min(&mut self) -> Option<(u32, usize)> {
        if self.size == 0 {
            return None;
        }

        // Find the first non-empty bucket
        let first_bucket = self
            .buckets
            .iter()
            .enumerate()
            .find(|(_, bucket)| !bucket.is_empty())
            .map(|(i, _)| i);

        let bucket_idx = first_bucket?;

        // Find the minimum in this bucket
        let (min_pos, min_key) = self.buckets[bucket_idx]
            .iter()
            .enumerate()
            .min_by(|x, y| x.1.key.cmp(&y.1.key))
            .map(|(pos, node)| (pos, node.key))
            .unwrap_or((0, u32::MAX));

        // Remove the minimum node from the bucket using swap_remove for O(1) removal
        let extracted_node = self.buckets[bucket_idx].swap_remove(min_pos);
        self.node_positions[extracted_node.node_id] = None;

        // Update the swapped node's position if a swap occurred
        // After swap_remove, if min_pos != len-1, the element at len-1 moved to min_pos
        let bucket_len = self.buckets[bucket_idx].len();
        if min_pos < bucket_len {
            // A swap occurred (min_pos was not the last element)
            let swapped_node = &self.buckets[bucket_idx][min_pos];
            self.node_positions[swapped_node.node_id] = Some((bucket_idx, min_pos));
        }

        self.size -= 1;

        // Update min_key
        self.min_key = min_key;

        // Redistribute all remaining nodes from this bucket into lower buckets.
        let nodes_to_redistribute = std::mem::take(&mut self.buckets[bucket_idx]);
        for node in nodes_to_redistribute {
            let node_id = node.node_id;
            let new_bucket_idx = self.bucket_index(node.key);
            let pos = self.buckets[new_bucket_idx].len();
            self.buckets[new_bucket_idx].push(node);
            self.node_positions[node_id] = Some((new_bucket_idx, pos));
        }

        Some((min_key, extracted_node.node_id))
    }

    /// Decrease the key of a node identified by its handle.
    ///
    /// This operation moves the node to a new bucket if necessary.
    /// The new key must be less than the current key (and >= min_key).
    pub fn decrease_key(&mut self, handle: &RadixHandle, new_key: u32) {
        let node_id = handle.node_id;

        // Find the node's current position
        if node_id >= self.node_positions.len() {
            self.node_positions.resize(node_id + 1, None);
        }
        let (old_bucket_idx, old_pos) = match self.node_positions[node_id] {
            Some(pos) => pos,
            None => return, // Node not in heap
        };

        // Validate that new_key is actually smaller
        let current_key = self.buckets[old_bucket_idx][old_pos].key;
        if new_key >= current_key {
            return; // Not actually decreasing
        }

        // Remove the node using swap_remove for O(1) removal
        let mut node = self.buckets[old_bucket_idx].swap_remove(old_pos);

        // Update position of the swapped node if a swap occurred
        // After swap_remove, if old_pos != len-1, the element at len-1 moved to old_pos
        let bucket_len = self.buckets[old_bucket_idx].len();
        if old_pos < bucket_len {
            // A swap occurred (old_pos was not the last element)
            let swapped_node = &self.buckets[old_bucket_idx][old_pos];
            self.node_positions[swapped_node.node_id] = Some((old_bucket_idx, old_pos));
        }

        // Update the node's key
        node.key = new_key;

        // Calculate new bucket index
        let new_bucket_idx = self.bucket_index(new_key);

        // Insert into new bucket
        let new_pos = self.buckets[new_bucket_idx].len();
        self.buckets[new_bucket_idx].push(node);

        self.node_positions[node_id] = Some((new_bucket_idx, new_pos));
    }

    /// Check if the heap is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Get the number of elements in the heap.
    pub fn len(&self) -> usize {
        self.size
    }
}

impl Default for RadixHeap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radix_heap_basic() {
        let mut heap = RadixHeap::new();
        assert_eq!(heap.extract_min(), None);

        heap.insert(10, 1);
        heap.insert(5, 2);
        heap.insert(15, 3);

        assert_eq!(heap.extract_min(), Some((5, 2)));
        assert_eq!(heap.extract_min(), Some((10, 1)));
        assert_eq!(heap.extract_min(), Some((15, 3)));
        assert_eq!(heap.extract_min(), None);
    }

    #[test]
    fn test_radix_heap_non_decreasing() {
        let mut heap = RadixHeap::new();

        // Insert keys in non-decreasing order (simulating Dijkstra's)
        heap.insert(0, 0);
        assert_eq!(heap.extract_min(), Some((0, 0)));

        heap.insert(5, 1);
        heap.insert(10, 2);
        assert_eq!(heap.extract_min(), Some((5, 1)));

        heap.insert(15, 3);
        assert_eq!(heap.extract_min(), Some((10, 2)));
        assert_eq!(heap.extract_min(), Some((15, 3)));
    }

    #[test]
    fn test_radix_heap_large_range() {
        let mut heap = RadixHeap::new();

        heap.insert(1000, 1);
        heap.insert(1, 2);
        heap.insert(500, 3);

        assert_eq!(heap.extract_min(), Some((1, 2)));
        assert_eq!(heap.extract_min(), Some((500, 3)));
        assert_eq!(heap.extract_min(), Some((1000, 1)));
    }

    #[test]
    fn test_radix_heap_dijkstra_like_sequence() {
        // Test that mimics Dijkstra's algorithm usage pattern
        // Multiple extract_min and decrease_key operations
        let mut heap = RadixHeap::new();
        // Insert nodes like in Dijkstra's
        let handles = [
            heap.insert(0, 0),
            heap.insert(u32::MAX, 1),
            heap.insert(u32::MAX, 2),
            heap.insert(u32::MAX, 3),
            heap.insert(u32::MAX, 4),
        ];

        // Extract start node
        assert_eq!(heap.extract_min(), Some((0, 0)));

        // Simulate discovering neighbors and decreasing keys
        // This pattern can trigger the bug if positions become stale
        heap.decrease_key(&handles[1], 10);
        heap.decrease_key(&handles[2], 20);
        heap.decrease_key(&handles[3], 30);
        heap.decrease_key(&handles[4], 40);

        // Extract minimum (should be node 1 with key 10)
        assert_eq!(heap.extract_min(), Some((10, 1)));

        // Now decrease keys of remaining nodes
        heap.decrease_key(&handles[2], 15); // Decrease node 2 from 20 to 15
        heap.decrease_key(&handles[3], 25); // Decrease node 3 from 30 to 25

        // Extract and verify
        assert_eq!(heap.extract_min(), Some((15, 2)));
        assert_eq!(heap.extract_min(), Some((25, 3)));
        assert_eq!(heap.extract_min(), Some((40, 4)));
    }

    #[test]
    fn test_radix_heap_invariant() {
        // If bucket_index is wrongly computed, the invariant may be violated:
        // - 8 can remain in a higher bucket (computed vs old min_key = 0),
        // - inserting 9 (vs new min_key = 7) goes to a lower bucket,
        // - and extract_min can incorrectly return 9 before 8.
        let mut heap = RadixHeap::new();

        heap.insert(7, 1);
        heap.insert(8, 2);
        assert_eq!(heap.extract_min(), Some((7, 1)));

        heap.insert(9, 3);
        assert_eq!(heap.extract_min(), Some((8, 2)));
        assert_eq!(heap.extract_min(), Some((9, 3)));
    }
}
