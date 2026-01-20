use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Core Dijkstra implementation using a binary heap.
///
/// Exposed as `pub` so it can be re-exported from the `dijkstra` module.
pub fn dijkstra(start: usize, end: usize, graph: &[Vec<(usize, u32)>]) -> Vec<usize> {
    let mut distances = vec![u32::MAX; graph.len()];
    distances[start] = 0;
    let mut previous = vec![None; graph.len()];
    let mut priority_queue = BinaryHeap::new();
    priority_queue.push((Reverse(0), start));

    while let Some((Reverse(current_distance), current_node)) = priority_queue.pop() {
        // Early termination: if we've reached the target, we're done
        if current_node == end {
            break;
        }

        // Skip if we've already found a better path to this node
        if distances[current_node] < current_distance {
            continue;
        }

        // Iterate only over actual neighbors (adjacency list)
        for &(neighbor, weight) in &graph[current_node] {
            let new_distance = current_distance + weight;
            if new_distance < distances[neighbor] {
                distances[neighbor] = new_distance;
                previous[neighbor] = Some(current_node);
                priority_queue.push((Reverse(new_distance), neighbor));
            }
        }
    }

    // Reconstruct path in reverse order to avoid reversing at the end
    let mut path = Vec::new();
    let mut current = end;

    // Check if path exists
    if distances[end] == u32::MAX {
        return path; // No path found
    }

    // Build path backwards, then reverse once
    while let Some(prev) = previous[current] {
        path.push(current);
        current = prev;
    }
    path.push(start);
    path.reverse();
    path
}
