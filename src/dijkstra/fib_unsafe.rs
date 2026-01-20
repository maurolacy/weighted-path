use crate::fibonacci_heap_unsafe::{UnsafeFibonacciHeap, UnsafeNode};

/// Dijkstra implementation using the unsafe/raw-pointer Fibonacci heap.
///
/// Potentially much faster on dense graphs with many `decrease_key` operations.
pub fn dijkstra_fibonacci(start: usize, end: usize, graph: &[Vec<(usize, u32)>]) -> Vec<usize> {
    let mut distances = vec![u32::MAX; graph.len()];
    distances[start] = 0;
    let mut previous = vec![None; graph.len()];

    // Track handles (raw pointers) for decrease_key operations
    let mut handles: Vec<Option<*mut UnsafeNode>> = vec![None; graph.len()];
    let mut heap = UnsafeFibonacciHeap::new();

    // Insert start node
    let handle = heap.insert(0, start);
    handles[start] = Some(handle);

    while let Some((current_distance, current_node)) = heap.extract_min() {
        // Skip if we've already found a better path (duplicate entry)
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

                // Use decrease_key if node is already in heap, otherwise insert
                if let Some(handle) = handles[neighbor] {
                    heap.decrease_key(handle, new_distance);
                } else {
                    let handle = heap.insert(new_distance, neighbor);
                    handles[neighbor] = Some(handle);
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
