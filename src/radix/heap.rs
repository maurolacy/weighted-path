/// Radix heap implementation for priority queues with non-decreasing integer keys.
///
/// A radix heap is a specialized priority queue that exploits the property that
/// extracted keys are monotonically non-decreasing (i.e., each `extract_min` returns
/// a key >= the previous one). This makes it very efficient for algorithms like
/// Dijkstra's algorithm.
///
/// The heap organizes elements into buckets based on the number of bits needed to
/// represent the difference between their key and the current minimum key. This allows
/// for O(1) amortized `insert` and `decrease_key`, and O(log C) amortized `extract_min`,
/// where C is the maximum key difference.
///
/// # Performance
///
/// - **Insert**: O(1) amortized
/// - **Extract-min**: O(log C) amortized, where C is the key range
/// - **Decrease-key**: O(1) amortized
///
/// In practice, radix heaps often outperform Fibonacci heaps for Dijkstra's algorithm,
/// especially when edge weights are bounded integers.
use std::collections::{HashMap, VecDeque};

/// Node in the radix heap.
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
/// difference from the current minimum requires `i` bits to represent. When we extract
/// the minimum, we redistribute nodes from the first non-empty bucket into lower buckets.
pub struct RadixHeap {
    /// Buckets: bucket[i] contains nodes where the key difference needs i bits.
    /// We use VecDeque for efficient insertion and removal.
    buckets: Vec<VecDeque<Node>>,
    /// Current minimum key (the last extracted key, or 0 initially).
    min_key: u32,
    /// Total number of elements in the heap.
    size: usize,
    /// Map from node_id to (bucket_index, position_in_bucket) for O(1) decrease_key.
    node_positions: HashMap<usize, (usize, usize)>,
}

impl RadixHeap {
    /// Create a new empty radix heap.
    pub fn new() -> Self {
        // u32 has 32 bits, so we need at most 33 buckets (0..=32)
        // to represent any key difference
        let max_buckets = 33;
        RadixHeap {
            buckets: vec![VecDeque::new(); max_buckets],
            min_key: 0,
            size: 0,
            node_positions: HashMap::new(),
        }
    }

    /// Get the bucket index for a given key.
    ///
    /// The bucket index is the number of bits needed to represent (key - min_key),
    /// or 0 if key == min_key.
    fn bucket_index(&self, key: u32) -> usize {
        if key == self.min_key {
            return 0;
        }
        // Calculate the difference, handling potential underflow
        let diff = key.wrapping_sub(self.min_key);
        // Find the position of the most significant bit
        // This is the number of bits needed to represent the difference
        if diff == 0 {
            0
        } else {
            (32 - diff.leading_zeros()) as usize
        }
    }

    /// Insert a node with the given key and node ID.
    pub fn insert(&mut self, key: u32, node_id: usize) -> RadixHandle {
        let bucket_idx = self.bucket_index(key);
        let pos = self.buckets[bucket_idx].len();
        self.buckets[bucket_idx].push_back(Node { key, node_id });
        self.node_positions.insert(node_id, (bucket_idx, pos));
        self.size += 1;
        RadixHandle { node_id }
    }

    /// Extract and return the minimum key and node ID.
    ///
    /// This operation redistributes nodes from the first non-empty bucket into
    /// lower buckets based on their new key differences.
    pub fn extract_min(&mut self) -> Option<(u32, usize)> {
        if self.size == 0 {
            return None;
        }

        // Find the first non-empty bucket
        let mut first_bucket = None;
        for (i, bucket) in self.buckets.iter().enumerate() {
            if !bucket.is_empty() {
                first_bucket = Some(i);
                break;
            }
        }

        let bucket_idx = first_bucket?;

        // If bucket 0 is non-empty, we can extract directly
        if bucket_idx == 0 {
            let node = self.buckets[0].pop_front().unwrap();
            self.min_key = node.key;
            self.node_positions.remove(&node.node_id);
            self.size -= 1;
            // Update positions of remaining nodes in bucket 0
            for (pos, remaining_node) in self.buckets[0].iter().enumerate() {
                self.node_positions.insert(remaining_node.node_id, (0, pos));
            }
            return Some((node.key, node.node_id));
        }

        // Otherwise, we need to redistribute nodes from this bucket
        // Find the minimum in this bucket
        let mut min_key = u32::MAX;
        let mut min_pos = 0;

        for (pos, node) in self.buckets[bucket_idx].iter().enumerate() {
            if node.key < min_key {
                min_key = node.key;
                min_pos = pos;
            }
        }

        // Remove the minimum node from the bucket
        let extracted_node = self.buckets[bucket_idx].remove(min_pos).unwrap();
        self.node_positions.remove(&extracted_node.node_id);
        self.size -= 1;

        // Update min_key
        self.min_key = min_key;

        // Redistribute all remaining nodes from this bucket into lower buckets
        let nodes_to_redistribute: Vec<Node> = self.buckets[bucket_idx].drain(..).collect();
        for node in nodes_to_redistribute {
            let new_bucket_idx = self.bucket_index(node.key);
            let pos = self.buckets[new_bucket_idx].len();
            self.buckets[new_bucket_idx].push_back(node.clone());
            self.node_positions
                .insert(node.node_id, (new_bucket_idx, pos));
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
        let (old_bucket_idx, old_pos) = match self.node_positions.get(&node_id) {
            Some(pos) => *pos,
            None => return, // Node not in heap
        };

        // Get the node
        let mut node = self.buckets[old_bucket_idx]
            .remove(old_pos)
            .expect("Node should exist at tracked position");

        // Validate that new_key is actually smaller
        if new_key >= node.key {
            // Not actually decreasing, put it back
            self.buckets[old_bucket_idx].insert(old_pos, node);
            return;
        }

        // Update the node's key
        node.key = new_key;

        // Calculate new bucket index
        let new_bucket_idx = self.bucket_index(new_key);

        // Update positions of remaining nodes in the old bucket
        for (pos, remaining_node) in self.buckets[old_bucket_idx].iter().enumerate() {
            self.node_positions
                .insert(remaining_node.node_id, (old_bucket_idx, pos));
        }

        // Insert into new bucket
        let new_pos = self.buckets[new_bucket_idx].len();
        self.buckets[new_bucket_idx].push_back(node);
        self.node_positions
            .insert(node_id, (new_bucket_idx, new_pos));
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
}
