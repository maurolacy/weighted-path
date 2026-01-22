use crate::dijkstra::heap_trait::PriorityQueue;

/// Generic Dijkstra's algorithm implementation that works with any `PriorityQueue`.
///
/// This is the core implementation that all specific heap variants use.
/// It handles both heaps that support `decrease_key` and those that don't.
pub fn dijkstra_generic<Q: PriorityQueue>(
    start: usize,
    end: usize,
    graph: &[Vec<(usize, u32)>],
    mut heap: Q,
) -> Vec<usize> {
    let mut distances = vec![u32::MAX; graph.len()];
    distances[start] = 0;
    let mut previous = vec![None; graph.len()];

    // Check once if heap supports decrease_key (compile-time constant, but checked at runtime)
    let supports_decrease_key = heap.supports_decrease_key();

    // Track handles for decrease_key operations (only used if heap supports it)
    let mut handles: Vec<Option<Q::Handle>> = if supports_decrease_key {
        vec![None; graph.len()]
    } else {
        Vec::new() // Don't allocate if not needed
    };

    // Insert start node
    let handle = heap.insert(0, start);
    if supports_decrease_key {
        handles[start] = Some(handle);
    }

    while let Some((current_distance, current_node)) = heap.extract_min() {
        // Skip if we've already found a better path (duplicate entry)
        // This happens for heaps that don't support decrease_key (like BinaryHeap)
        if distances[current_node] < current_distance {
            continue;
        }

        // Early termination: if we've reached the target, we're done
        if current_node == end {
            break;
        }

        // Process neighbors
        for &(neighbor, weight) in &graph[current_node] {
            let new_distance = current_distance + weight;
            if new_distance < distances[neighbor] {
                distances[neighbor] = new_distance;
                previous[neighbor] = Some(current_node);

                // Use decrease_key if heap supports it and node is already in heap
                if supports_decrease_key {
                    if let Some(ref handle) = handles[neighbor] {
                        heap.decrease_key(handle, new_distance);
                    } else {
                        let handle = heap.insert(new_distance, neighbor);
                        handles[neighbor] = Some(handle);
                    }
                } else {
                    // For heaps without decrease_key (like BinaryHeap), always re-insert
                    // Duplicates will be filtered out by the check above
                    heap.insert(new_distance, neighbor);
                }
            }
        }
    }

    // Reconstruct path
    let mut path = Vec::new();
    let mut current = end;

    if distances[end] == u32::MAX {
        return path; // No path found
    }

    while let Some(prev) = previous[current] {
        path.push(current);
        current = prev;
    }
    path.push(start);
    path.reverse();
    path
}
