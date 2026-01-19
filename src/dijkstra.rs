#![allow(non_snake_case)]

use std::collections::{BinaryHeap, HashMap};
use crate::fibonacci_heap::{FibonacciHeap, Node};

// code goes here
// note: you are able to modify the parameter types
pub fn GraphChallenge(lines: Vec<&str>) -> Result<String, String> {
    GraphChallengeWithDirection(lines, true) // Default to undirected/bidirectional
}

/// Process graph with explicit control over edge directionality.
///
/// # Arguments
/// * `lines` - Graph definition lines (same format as GraphChallenge)
/// * `bidirectional` - If true, edges are made bidirectional (undirected graph).
///                     If false, edges are one-way only (directed graph).
///                     When true, if both A->B and B->A are specified, the last weight wins.
pub fn GraphChallengeWithDirection(lines: Vec<&str>, bidirectional: bool) -> Result<String, String> {
    //  println!("{:?}", lines);

    // 1. Parse the graph and build and adjacency matrix.
    if lines.is_empty() {
        return Ok("-1".to_string());
    }

    // Number of nodes
    let N = lines[0]
        .parse::<u32>()
        .map_err(|_| format!("Invalid number of nodes: '{}' (expected a positive integer)", lines[0]))? as usize;

    if N == 0 {
        return Ok("-1".to_string());
    }

    // Validate we have enough lines for node names
    if lines.len() < 1 + N {
        return Err(format!(
            "Not enough lines: expected {} node names, but only {} lines provided",
            N,
            lines.len().saturating_sub(1)
        ));
    }

    // Get the nodes
    let mut nodes = HashMap::new();
    let mut nodes_reverse = HashMap::new();
    let mut seen_nodes = std::collections::HashSet::new();

    for (i, &item) in lines.iter().enumerate().skip(1usize).take(N) {
        let node_name = item.trim();
        if node_name.is_empty() {
            return Err(format!("Empty node name at line {}", i + 1));
        }

        // Check for duplicate node names
        if !seen_nodes.insert(node_name) {
            return Err(format!("Duplicate node name: '{}'", node_name));
        }

        nodes.insert(node_name, i - 1); // node id map
        nodes_reverse.insert(i - 1, node_name); // node map
    }

    if N == 1 {
        return Ok(nodes_reverse.get(&0)
            .ok_or_else(|| "Internal error: single node not found in reverse map".to_string())?
            .to_string());
    }

    // Build the adjacency list: Vec<Vec<(neighbor_index, weight)>>
    // More space-efficient for sparse graphs: O(V + E) instead of O(V²)
    let mut graph = vec![Vec::new(); N];

    for (line_num, line) in lines.iter().skip(1 + N).enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue; // Skip empty lines
        }

        // Split the line into node 1, node 2, and weight
        let parts: Vec<&str> = line.split("|").collect();

        if parts.len() != 3 {
            return Err(format!(
                "Invalid edge format at line {}: '{}' (expected format: node1|node2|weight)",
                line_num + 1 + N + 1,
                line
            ));
        }

        let node_1 = parts[0].trim();
        let node_2 = parts[1].trim();
        let weight_str = parts[2].trim();

        // Validate node names exist
        let node_1_index = nodes.get(node_1)
            .ok_or_else(|| format!("Node '{}' in edge definition not found in node list", node_1))?;
        let node_2_index = nodes.get(node_2)
            .ok_or_else(|| format!("Node '{}' in edge definition not found in node list", node_2))?;

        // Validate weight
        let weight = weight_str
            .parse::<u32>()
            .map_err(|_| format!("Invalid weight '{}' in edge '{}|{}|{}' (expected a positive integer)", weight_str, node_1, node_2, weight_str))?;

        // Check for self-loops (optional validation)
        if node_1_index == node_2_index {
            return Err(format!("Self-loop detected: node '{}' connected to itself", node_1));
        }

        // Add edge to adjacency list
        graph[*node_1_index].push((*node_2_index, weight));

        // If bidirectional is true, also add the reverse edge
        if bidirectional {
            graph[*node_2_index].push((*node_1_index, weight));
        }
    }

    // 2. Use Dijstra's to traverse the graph from the first node, and find
    // the shortest path to the last one.
    // Using adjacency list is more efficient for sparse graphs.
    // Use binary heap by default (faster for most cases), Fibonacci heap available for benchmarking
    let path = dijkstra(0, N - 1, &graph);
    //  println!("Path: {path:#?}");

    // 3. Return the shortest path; if no shortest path found, return -1.
    if path.len() <= 1 {
        return Ok("-1".to_string());
    }

    // Map path node ids to nodes - optimized string building
    let mut path_parts = Vec::with_capacity(path.len());
    for node_id in path {
        let node = nodes_reverse.get(&node_id)
            .ok_or_else(|| format!("Internal error: node ID {} not found in reverse map", node_id))?;
        path_parts.push(*node);
    }
    Ok(path_parts.join("-"))
}

// Expose for testing and benchmarking
pub(crate) fn dijkstra(start: usize, end: usize, graph: &[Vec<(usize, u32)>]) -> Vec<usize> {
    let mut distances = vec![u32::MAX; graph.len()];
    distances[start] = 0;
    let mut previous = vec![None; graph.len()];
    let mut priority_queue = BinaryHeap::new();
    priority_queue.push((std::cmp::Reverse(0), start));

    while let Some((std::cmp::Reverse(current_distance), current_node)) = priority_queue.pop() {
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
                priority_queue.push((std::cmp::Reverse(new_distance), neighbor));
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

// Fibonacci heap version - potentially faster for dense graphs with many decrease_key operations
// Time complexity: O(E + V log V) amortized vs O((V + E) log V) for binary heap
// Uses our custom Fibonacci heap implementation that properly supports decrease_key
/// Fibonacci heap version - exposed for benchmarking
pub fn dijkstra_fibonacci(start: usize, end: usize, graph: &[Vec<(usize, u32)>]) -> Vec<usize> {
    let mut distances = vec![u32::MAX; graph.len()];
    distances[start] = 0;
    let mut previous = vec![None; graph.len()];

    // Track handles (raw pointers) for decrease_key operations
    // Our custom implementation uses raw pointers to avoid RefCell borrow conflicts
    let mut handles: Vec<Option<*mut Node>> = vec![None; graph.len()];
    let mut heap = FibonacciHeap::new();

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
                    // Node is already in heap - use decrease_key (O(1) amortized)
                    heap.decrease_key(handle, new_distance);
                } else {
                    // Insert new node
                    let handle = heap.insert(new_distance, neighbor);
                    handles[neighbor] = Some(handle);
                }
            }
        }
    }

    // Reconstruct path
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

// keep this function call here
/*
fn main() {
  GraphChallenge(coderbyteInternalStdinFunction(io::stdin()));
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let result = GraphChallenge(vec![]);
        assert_eq!(result, Ok("-1".to_string()));
    }

    #[test]
    fn test_single_node() {
        let input = vec!["1", "A"];
        let result = GraphChallenge(input);
        assert_eq!(result, Ok("A".to_string()));
    }

    #[test]
    fn test_two_nodes_connected() {
        let input = vec!["2", "A", "B", "A|B|5"];
        let result = GraphChallenge(input);
        assert_eq!(result, Ok("A-B".to_string()));
    }

    #[test]
    fn test_two_nodes_disconnected() {
        let input = vec!["2", "A", "B"];
        let result = GraphChallenge(input);
        assert_eq!(result, Ok("-1".to_string()));
    }

    #[test]
    fn test_simple_path() {
        let input = vec!["3", "A", "B", "C", "A|B|2", "B|C|3"];
        let result = GraphChallenge(input);
        assert_eq!(result, Ok("A-B-C".to_string()));
    }

    #[test]
    fn test_shortest_path_through_intermediate() {
        // Direct path A->D costs 100, but A->B->C->D costs 1+1+1=3
        let input = vec!["4", "A", "B", "C", "D", "A|B|1", "B|C|1", "C|D|1", "A|D|100"];
        let result = GraphChallenge(input);
        assert_eq!(result, Ok("A-B-C-D".to_string()));
    }

    #[test]
    fn test_invalid_node_count() {
        let input = vec!["abc"];
        let result = GraphChallenge(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid number of nodes"));
    }

    #[test]
    fn test_not_enough_nodes() {
        let input = vec!["3", "A", "B"];
        let result = GraphChallenge(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not enough lines"));
    }

    #[test]
    fn test_duplicate_node_names() {
        let input = vec!["2", "A", "A", "A|A|5"];
        let result = GraphChallenge(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate node name"));
    }

    #[test]
    fn test_invalid_edge_format() {
        let input = vec!["2", "A", "B", "A|B"];
        let result = GraphChallenge(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid edge format"));
    }

    #[test]
    fn test_node_not_in_list() {
        let input = vec!["2", "A", "B", "A|C|5"];
        let result = GraphChallenge(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in node list"));
    }

    #[test]
    fn test_invalid_weight() {
        let input = vec!["2", "A", "B", "A|B|abc"];
        let result = GraphChallenge(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid weight"));
    }

    #[test]
    fn test_self_loop() {
        let input = vec!["2", "A", "B", "A|A|5"];
        let result = GraphChallenge(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Self-loop detected"));
    }

    #[test]
    fn test_empty_node_name() {
        let input = vec!["2", "", "B", "A|B|5"];
        let result = GraphChallenge(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty node name"));
    }

    #[test]
    fn test_zero_nodes() {
        let input = vec!["0"];
        let result = GraphChallenge(input);
        assert_eq!(result, Ok("-1".to_string()));
    }

    #[test]
    fn test_fibonacci_heap_simple() {
        // Test that Fibonacci heap version produces same results as binary heap
        let graph = vec![
            vec![(1, 2), (2, 4)],  // node 0
            vec![(0, 2), (3, 1)],  // node 1
            vec![(0, 4), (3, 5)],  // node 2
            vec![(1, 1), (2, 5)],  // node 3
        ];
        let path_binary = dijkstra(0, 3, &graph);
        let path_fib = dijkstra_fibonacci(0, 3, &graph);
        assert_eq!(path_binary, path_fib);
        assert_eq!(path_binary, vec![0, 1, 3]);
    }

    #[test]
    fn test_fibonacci_heap_comprehensive() {
        // Comprehensive test comparing both implementations on various graphs
        let test_cases = vec![
            // Simple path
            (vec![vec![(1, 1)], vec![(2, 1)], vec![]], 0, 2, vec![0, 1, 2]),
            // Disconnected nodes
            (vec![vec![], vec![]], 0, 1, vec![]),
            // Single node
            (vec![vec![]], 0, 0, vec![0]),
            // Complex graph
            (
                vec![
                    vec![(1, 4), (2, 2)],      // node 0
                    vec![(2, 1), (3, 5)],     // node 1
                    vec![(1, 1), (3, 8), (4, 10)], // node 2
                    vec![(4, 2)],             // node 3
                    vec![],                   // node 4
                ],
                0,
                4,
                vec![0, 2, 1, 3, 4], // Path: 0->2->1->3->4 (cost: 2+1+5+2=10)
            ),
            // Graph with multiple paths
            (
                vec![
                    vec![(1, 1), (2, 5)],     // node 0
                    vec![(2, 1), (3, 2)],     // node 1
                    vec![(3, 3)],             // node 2
                    vec![],                   // node 3
                ],
                0,
                3,
                vec![0, 1, 3], // Shortest: 0->1->3 (cost: 1+2=3), not 0->2->3 (cost: 5+3=8)
            ),
        ];

        for (graph, start, end, expected_path) in test_cases {
            let path_binary = dijkstra(start, end, &graph);
            let path_fib = dijkstra_fibonacci(start, end, &graph);

            assert_eq!(
                path_binary, path_fib,
                "Mismatch for graph: start={}, end={}, binary={:?}, fib={:?}",
                start, end, path_binary, path_fib
            );

            if !expected_path.is_empty() {
                assert_eq!(
                    path_binary, expected_path,
                    "Path doesn't match expected: got {:?}, expected {:?}",
                    path_binary, expected_path
                );
            }
        }
    }

    #[test]
    fn test_empty_lines_in_edges() {
        // Should skip empty lines in edge definitions
        let input = vec!["2", "A", "B", "A|B|5", "", "A|B|3"];
        let result = GraphChallenge(input);
        // Should work, but the last edge will overwrite the first
        assert_eq!(result, Ok("A-B".to_string()));
    }
}
