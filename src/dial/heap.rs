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
    /// Uses Vec with swap_remove for O(1) removal.
    buckets: Vec<Vec<usize>>,
    /// Current bucket being processed.
    current_bucket: usize,
    /// Maximum distance seen so far (for dynamic bucket allocation).
    max_distance: usize,
    /// Distances from source to each node.
    distances: Vec<u32>,
    /// Map from node_id to (bucket_index, position_in_bucket) for O(1) decrease_key.
    /// None means the node is not in the heap.
    node_positions: Vec<Option<(usize, usize)>>,
    /// Number of nodes currently in buckets.
    size: usize,
}

/// Maximum number of buckets supported by Dial's algorithm.
/// Distances exceeding this value will cause a panic.
const MAX_BUCKETS: usize = 10_000_000;

impl DialHeap {
    /// Create a new Dial heap with initial capacity.
    ///
    /// `max_nodes` should be the number of nodes in the graph.
    /// `initial_max_distance` is an estimate of maximum distance (for pre-allocation).
    pub fn new(max_nodes: usize, initial_max_distance: usize) -> Self {
        DialHeap {
            buckets: vec![Vec::new(); initial_max_distance + 1],
            current_bucket: 0,
            max_distance: 0,
            distances: vec![u32::MAX; max_nodes],
            node_positions: vec![None; max_nodes],
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
            self.node_positions.resize(node_id + 1, None);
        }

        // Don't insert nodes with u32::MAX - they'll be inserted via decrease_key
        if distance == u32::MAX {
            self.distances[node_id] = distance;
            return DialHandle { node_id };
        }

        // Only insert if this is a better distance
        if distance < self.distances[node_id] {
            // Remove from old bucket if it exists
            if let Some((old_bucket, old_pos)) = self.node_positions[node_id] {
                self.remove_from_bucket(old_bucket, old_pos);
            }

            // Add to new bucket
            let dist = distance as usize;
            self.ensure_bucket_capacity(dist);
            let pos = self.buckets[dist].len();
            self.buckets[dist].push(node_id);
            self.node_positions[node_id] = Some((dist, pos));
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
        // Process buckets sequentially starting from current_bucket
        while self.current_bucket <= self.max_distance {
            if let Some(node_id) = self.buckets[self.current_bucket].pop() {
                self.node_positions[node_id] = None;
                self.size -= 1;
                return Some((self.distances[node_id], node_id));
            }
            self.current_bucket += 1;
        }
        None
    }

    /// Decrease the key (distance) of a node.
    ///
    /// Moves the node to a lower bucket if the new distance is smaller.
    pub fn decrease_key(&mut self, handle: &DialHandle, new_distance: u32) {
        // Don't process u32::MAX
        if new_distance == u32::MAX {
            return;
        }

        let node_id = handle.node_id;
        // Ensure vectors are large enough
        if node_id >= self.distances.len() {
            self.distances.resize(node_id + 1, u32::MAX);
            self.node_positions.resize(node_id + 1, None);
        }

        // Check if this is actually a decrease
        if new_distance >= self.distances[node_id] {
            return;
        }

        // Remove from old bucket if it exists
        if let Some((old_bucket, old_pos)) = self.node_positions[node_id] {
            self.remove_from_bucket(old_bucket, old_pos);
        }

        // Add to new bucket
        let new_bucket = new_distance as usize;
        self.ensure_bucket_capacity(new_bucket);
        let new_pos = self.buckets[new_bucket].len();
        self.buckets[new_bucket].push(node_id);
        self.node_positions[node_id] = Some((new_bucket, new_pos));
        self.distances[node_id] = new_distance;
        self.size += 1;
        self.max_distance = self.max_distance.max(new_bucket);
    }

    /// Remove a node from a specific bucket at the given position.
    /// Uses swap_remove for O(1) removal and updates the swapped node's position.
    fn remove_from_bucket(&mut self, bucket_idx: usize, pos: usize) {
        let bucket = &mut self.buckets[bucket_idx];
        bucket.swap_remove(pos);
        // Update position of swapped node if a swap occurred
        if pos < bucket.len() {
            let swapped_node_id = bucket[pos];
            self.node_positions[swapped_node_id] = Some((bucket_idx, pos));
        }
        self.size -= 1;
    }

    /// Ensure we have enough buckets for the given distance.
    /// Panics if the distance exceeds MAX_BUCKETS.
    fn ensure_bucket_capacity(&mut self, dist: usize) {
        if dist >= MAX_BUCKETS {
            panic!(
                "Distance {} exceeds maximum supported distance {} for Dial's algorithm. \
                 Consider using a different heap implementation for graphs with large edge weights.",
                dist, MAX_BUCKETS
            );
        }
        if dist >= self.buckets.len() {
            self.buckets.resize(dist + 1, Vec::new());
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

    #[test]
    #[should_panic(expected = "exceeds maximum supported distance")]
    fn test_dial_heap_max_buckets_insert() {
        let mut heap = DialHeap::new(10, 100);
        // Insert with distance exactly at MAX_BUCKETS should panic
        heap.insert(MAX_BUCKETS as u32, 1);
    }

    #[test]
    #[should_panic(expected = "exceeds maximum supported distance")]
    fn test_dial_heap_max_buckets_decrease_key() {
        let mut heap = DialHeap::new(10, 100);
        // Insert with u32::MAX (doesn't go into buckets, but sets distance)
        let handle = heap.insert(u32::MAX, 1);
        // Decrease key with distance exceeding MAX_BUCKETS should panic
        // This is a valid decrease (u32::MAX > MAX_BUCKETS), so it will call ensure_bucket_capacity
        heap.decrease_key(&handle, MAX_BUCKETS as u32);
    }
}
