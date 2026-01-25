/// Dial's algorithm (bucket-based Dijkstra) implementation.
///
/// Dial's algorithm uses buckets indexed by distance values instead of a
/// priority queue. It's very efficient when edge weights are small integers,
/// providing O(V + E + C) time complexity where C is the maximum distance.
///
/// # Performance
///
/// - **Time complexity**: O(V + E + C), where C is the maximum distance
/// - **Space complexity**: O(V + C) for buckets
///
/// This is optimal when C (maximum distance) is small compared to V log V.
/// For graphs with bounded integer edge weights (like 1..=100), this can be
/// faster than priority queue-based algorithms.
use std::collections::VecDeque;

/// Handle for Dial's algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DialHandle {
    node_id: usize,
}

/// Dial's algorithm bucket-based structure.
///
/// Buckets are indexed by exact distance values. Bucket[i] contains all nodes
/// with distance i. We process buckets in order (0, 1, 2, ...).
pub struct DialHeap {
    /// Buckets indexed by distance: buckets[i] contains nodes with distance i.
    buckets: Vec<VecDeque<usize>>,
    /// Current bucket being processed.
    current_bucket: usize,
    /// Maximum distance seen so far (for dynamic bucket allocation).
    max_distance: usize,
    /// Distances from source to each node.
    distances: Vec<u32>,
    /// Map from node_id to bucket index (for efficient decrease_key).
    node_buckets: Vec<Option<usize>>,
    /// Number of nodes currently in buckets.
    size: usize,
}

impl DialHeap {
    /// Create a new Dial heap with initial capacity.
    ///
    /// `max_nodes` should be the number of nodes in the graph.
    /// `initial_max_distance` is an estimate of maximum distance (for pre-allocation).
    pub fn new(max_nodes: usize, initial_max_distance: usize) -> Self {
        DialHeap {
            buckets: vec![VecDeque::new(); initial_max_distance + 1],
            current_bucket: 0,
            max_distance: 0,
            distances: vec![u32::MAX; max_nodes],
            node_buckets: vec![None; max_nodes],
            size: 0,
        }
    }

    /// Insert a node with the given distance.
    ///
    /// Nodes with u32::MAX distance are not inserted into buckets (they represent
    /// unreachable nodes). They will be inserted when their distance decreases via decrease_key.
    pub fn insert(&mut self, distance: u32, node_id: usize) -> DialHandle {
        // Ensure vectors are large enough
        if node_id >= self.distances.len() {
            self.distances.resize(node_id + 1, u32::MAX);
            self.node_buckets.resize(node_id + 1, None);
        }

        // Don't insert nodes with u32::MAX - they'll be inserted via decrease_key
        if distance == u32::MAX {
            self.distances[node_id] = distance;
            return DialHandle { node_id };
        }

        // Only insert if this is a better distance
        if distance < self.distances[node_id] {
            // Remove from old bucket if it exists
            if let Some(old_bucket) = self.node_buckets[node_id] {
                self.remove_from_bucket(old_bucket, node_id);
            }

            // Add to new bucket
            let dist = distance as usize;
            self.ensure_bucket_capacity(dist);
            self.buckets[dist].push_back(node_id);
            self.node_buckets[node_id] = Some(dist);
            self.distances[node_id] = distance;
            self.size += 1;
            self.max_distance = self.max_distance.max(dist);
        }

        DialHandle { node_id }
    }

    /// Extract the minimum distance node.
    ///
    /// Processes buckets in order (0, 1, 2, ...) and extracts from the current bucket.
    pub fn extract_min(&mut self) -> Option<(u32, usize)> {
        loop {
            if self.current_bucket > self.max_distance {
                return None;
            }

            // Process current bucket
            while let Some(node_id) = self.buckets[self.current_bucket].pop_front() {
                // Verify this node still has the correct distance for this bucket
                // (it might have been moved to a different bucket via decrease_key)
                if self.distances[node_id] == self.current_bucket as u32
                    && self.node_buckets[node_id] == Some(self.current_bucket)
                {
                    // This node is valid - extract it
                    self.node_buckets[node_id] = None;
                    self.size -= 1;
                    return Some((self.distances[node_id], node_id));
                }
                // Otherwise, this node was moved to a different bucket via decrease_key
                // Skip it and continue with next node in this bucket
            }

            // Bucket is empty, move to next
            self.current_bucket += 1;
        }
    }

    /// Decrease the key (distance) of a node.
    ///
    /// Moves the node to a lower bucket if the new distance is smaller.
    pub fn decrease_key(&mut self, handle: &DialHandle, new_distance: u32) {
        let node_id = handle.node_id;

        // Ensure vectors are large enough
        if node_id >= self.distances.len() {
            self.distances.resize(node_id + 1, u32::MAX);
            self.node_buckets.resize(node_id + 1, None);
        }

        // Check if this is actually a decrease
        if new_distance >= self.distances[node_id] {
            return;
        }

        // Don't process u32::MAX
        if new_distance == u32::MAX {
            return;
        }

        // Remove from old bucket if it exists
        if let Some(old_bucket) = self.node_buckets[node_id] {
            self.remove_from_bucket(old_bucket, node_id);
        }

        // Add to new bucket
        let new_bucket = new_distance as usize;
        self.ensure_bucket_capacity(new_bucket);
        self.buckets[new_bucket].push_back(node_id);
        self.node_buckets[node_id] = Some(new_bucket);
        self.distances[node_id] = new_distance;
        self.size += 1;
        self.max_distance = self.max_distance.max(new_bucket);
    }

    /// Remove a node from a specific bucket.
    fn remove_from_bucket(&mut self, bucket_idx: usize, node_id: usize) {
        let bucket = &mut self.buckets[bucket_idx];
        if let Some(pos) = bucket.iter().position(|&id| id == node_id) {
            bucket.remove(pos);
            self.size -= 1;
        }
    }

    /// Ensure we have enough buckets for the given distance.
    fn ensure_bucket_capacity(&mut self, dist: usize) {
        if dist >= self.buckets.len() {
            // Limit bucket size to prevent excessive memory usage
            const MAX_BUCKETS: usize = 10_000_000;
            let new_size = (dist + 1).min(MAX_BUCKETS);
            self.buckets.resize(new_size, VecDeque::new());
        }
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

impl Default for DialHeap {
    fn default() -> Self {
        Self::new(0, 10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dial_heap_basic() {
        let mut heap = DialHeap::new(10, 100);
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
    fn test_dial_heap_decrease_key() {
        let mut heap = DialHeap::new(10, 100);
        let handle1 = heap.insert(20, 1);
        let handle2 = heap.insert(30, 2);

        heap.decrease_key(&handle1, 10);
        heap.decrease_key(&handle2, 15);

        assert_eq!(heap.extract_min(), Some((10, 1)));
        assert_eq!(heap.extract_min(), Some((15, 2)));
    }

    #[test]
    fn test_dial_heap_dijkstra_like() {
        let mut heap = DialHeap::new(10, 1000);
        let handles = [
            heap.insert(0, 0),
            heap.insert(u32::MAX, 1),
            heap.insert(u32::MAX, 2),
            heap.insert(u32::MAX, 3),
        ];

        assert_eq!(heap.extract_min(), Some((0, 0)));

        heap.decrease_key(&handles[1], 10);
        heap.decrease_key(&handles[2], 20);
        heap.decrease_key(&handles[3], 30);

        assert_eq!(heap.extract_min(), Some((10, 1)));
        assert_eq!(heap.extract_min(), Some((20, 2)));
        assert_eq!(heap.extract_min(), Some((30, 3)));
    }
}
