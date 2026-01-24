pub mod binary;
pub mod fib;
pub mod fib_unsafe;
mod heap_trait;
pub mod pairing;
pub mod radix;

use std::collections::HashMap;

use heap_trait::PriorityQueue;

pub use binary::dijkstra_binary;
pub use fib::dijkstra_fibonacci;
pub use fib_unsafe::dijkstra_fibonacci_unsafe;
pub use pairing::dijkstra_pairing;
pub use radix::dijkstra_radix;

/// Function type for Dijkstra implementations over an adjacency list.
///
/// Parameters:
/// * `start` - Index of the source node in the adjacency list.
/// * `end` - Index of the target node in the adjacency list.
/// * `graph` - Adjacency list where `graph[u]` contains `(v, weight)` edges.
///
/// Returns:
/// * A vector of node indices representing the shortest path from `start` to `end`
///   (inclusive). Implementations should return an empty vector when no path exists.
pub type DijkstraFn = fn(usize, usize, &[Vec<(usize, u32)>]) -> Vec<usize>;

/// Parsed graph representation used by both the main API and benchmarks.
pub struct ParsedGraph<'a> {
    /// Adjacency list: `graph[u]` contains `(v, weight)` edges.
    pub graph: Vec<Vec<(usize, u32)>>,
    /// Reverse mapping from node index to original node name.
    pub nodes_reverse: HashMap<usize, &'a str>,
}

/// Parse a weighted graph from lines into an adjacency list and reverse node map.
pub fn parse_graph<'a>(
    lines: &'a [&'a str],
    bidirectional: bool,
) -> Result<ParsedGraph<'a>, String> {
    if lines.is_empty() {
        return Err("No input lines provided".to_string());
    }

    // Number of nodes
    let num_nodes = lines[0].parse::<u32>().map_err(|_| {
        format!(
            "Invalid number of nodes: '{}' (expected a positive integer)",
            lines[0]
        )
    })? as usize;

    // Validate we have enough lines for node names
    if lines.len() < 1 + num_nodes {
        return Err(format!(
            "Not enough lines: expected {} node names, but only {} lines provided",
            num_nodes,
            lines.len().saturating_sub(1)
        ));
    }

    // Get the nodes
    let mut nodes = HashMap::new();
    let mut nodes_reverse = HashMap::new();
    let mut seen_nodes = std::collections::HashSet::new();

    for (i, &item) in lines.iter().enumerate().skip(1usize).take(num_nodes) {
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

    // Build the adjacency list: Vec<Vec<(neighbor_index, weight)>>
    let mut graph = vec![Vec::new(); num_nodes];

    for (line_num, line) in lines.iter().skip(1 + num_nodes).enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue; // Skip empty lines
        }

        // Split the line into node 1, node 2, and weight
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() != 3 {
            return Err(format!(
                "Invalid edge format at line {}: '{}' (expected format: node1|node2|weight)",
                line_num + 1 + num_nodes + 1,
                line
            ));
        }

        let node_1 = parts[0].trim();
        let node_2 = parts[1].trim();
        let weight_str = parts[2].trim();

        // Validate node names exist
        let node_1_index = nodes.get(node_1).ok_or_else(|| {
            format!(
                "Node '{}' in edge definition not found in node list",
                node_1
            )
        })?;
        let node_2_index = nodes.get(node_2).ok_or_else(|| {
            format!(
                "Node '{}' in edge definition not found in node list",
                node_2
            )
        })?;

        // Validate weight
        let weight = weight_str.parse::<u32>().map_err(|_| {
            format!(
                "Invalid weight '{}' in edge '{}|{}|{}' (expected a positive integer)",
                weight_str, node_1, node_2, weight_str
            )
        })?;

        // Check for self-loops
        if node_1_index == node_2_index {
            return Err(format!(
                "Self-loop detected: node '{}' connected to itself",
                node_1
            ));
        }

        // Add edge to adjacency list
        graph[*node_1_index].push((*node_2_index, weight));

        // If bidirectional is true, also add the reverse edge
        if bidirectional {
            graph[*node_2_index].push((*node_1_index, weight));
        }
    }

    Ok(ParsedGraph {
        graph,
        nodes_reverse,
    })
}

/// Core Dijkstra's algorithm implementation that works with any `PriorityQueue`.
///
/// This is the generic implementation that all specific heap variants use.
/// It handles both heaps that support `decrease_key` and those that don't.
///
/// For convenience, specific heap variants are available:
/// - `dijkstra_binary` - Binary heap
/// - `dijkstra_fibonacci` - Safe Fibonacci heap
/// - `dijkstra_fibonacci_unsafe` - Unsafe Fibonacci heap
/// - `dijkstra_pairing` - Pairing heap
/// - `dijkstra_radix` - Radix heap
pub fn dijkstra<Q: PriorityQueue>(
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

fn find_shortest_path_with(
    lines: Vec<&str>,
    bidirectional: bool,
    dijkstra_impl: DijkstraFn,
) -> Result<String, String> {
    if lines.is_empty() {
        return Ok("-1".to_string());
    }

    let parsed = parse_graph(&lines, bidirectional)?;
    let num_nodes = parsed.graph.len();

    if num_nodes == 0 {
        return Ok("-1".to_string());
    }

    if num_nodes == 1 {
        return Ok(parsed
            .nodes_reverse
            .get(&0)
            .ok_or_else(|| "Internal error: single node not found in reverse map".to_string())?
            .to_string());
    }

    // Use the provided Dijkstra implementation
    let path = dijkstra_impl(0, num_nodes - 1, &parsed.graph);

    if path.len() <= 1 {
        return Ok("-1".to_string());
    }

    // Map path node ids to nodes - optimized string building
    let mut path_parts = Vec::with_capacity(path.len());
    for node_id in path {
        let node = parsed.nodes_reverse.get(&node_id).ok_or_else(|| {
            format!(
                "Internal error: node ID {} not found in reverse map",
                node_id
            )
        })?;
        path_parts.push(*node);
    }
    Ok(path_parts.join("-"))
}

/// Find the shortest path in a weighted graph using Dijkstra's algorithm.
///
/// This is a convenience function that treats the graph as undirected (bidirectional edges).
///
/// # Arguments
/// * `lines` - Graph definition lines (see README for format)
///
/// # Returns
/// * `Ok(path)` - Shortest path as a hyphen-separated string (e.g., "A-B-C")
/// * `Err(message)` - Error message if input is invalid or no path exists
pub fn find_shortest_path(lines: Vec<&str>) -> Result<String, String> {
    find_shortest_path_with(lines, true, dijkstra_binary) // Default to undirected/bidirectional
}

/// Find the shortest path using the safe Fibonacci-heap Dijkstra implementation.
pub fn find_shortest_path_fibonacci(lines: Vec<&str>) -> Result<String, String> {
    find_shortest_path_with(lines, true, dijkstra_fibonacci)
}

/// Find the shortest path using the unsafe (raw pointer) Fibonacci-heap Dijkstra implementation.
pub fn find_shortest_path_fibonacci_unsafe(lines: Vec<&str>) -> Result<String, String> {
    find_shortest_path_with(lines, true, dijkstra_fibonacci_unsafe)
}

/// Find the shortest path using the Pairing-heap Dijkstra implementation.
pub fn find_shortest_path_pairing(lines: Vec<&str>) -> Result<String, String> {
    find_shortest_path_with(lines, true, dijkstra_pairing)
}

/// Find the shortest path using the Radix-heap Dijkstra implementation.
pub fn find_shortest_path_radix(lines: Vec<&str>) -> Result<String, String> {
    find_shortest_path_with(lines, true, dijkstra_radix)
}

#[allow(clippy::doc_overindented_list_items)]
/// Find the shortest path in a weighted graph with explicit control over edge directionality.
///
/// # Arguments
/// * `lines` - Graph definition lines (same format as `find_shortest_path`)
/// * `bidirectional` - If `true`, edges are made bidirectional (undirected graph).
///                     If `false`, edges are one-way only (directed graph).
///                     When `true`, if both A->B and B->A are specified, the last weight wins.
///
/// # Returns
/// * `Ok(path)` - Shortest path as a hyphen-separated string (e.g., "A-B-C")
/// * `Err(message)` - Error message if input is invalid or no path exists
pub fn find_shortest_path_directed(
    lines: Vec<&str>,
    bidirectional: bool,
) -> Result<String, String> {
    find_shortest_path_with(lines, bidirectional, dijkstra_binary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let result = find_shortest_path(vec![]);
        assert_eq!(result, Ok("-1".to_string()));
    }

    #[test]
    fn test_single_node() {
        let input = vec!["1", "A"];
        let result = find_shortest_path(input);
        assert_eq!(result, Ok("A".to_string()));
    }

    #[test]
    fn test_two_nodes_connected() {
        let input = vec!["2", "A", "B", "A|B|5"];
        let result = find_shortest_path(input);
        assert_eq!(result, Ok("A-B".to_string()));
    }

    #[test]
    fn test_two_nodes_disconnected() {
        let input = vec!["2", "A", "B"];
        let result = find_shortest_path(input);
        assert_eq!(result, Ok("-1".to_string()));
    }

    #[test]
    fn test_simple_path() {
        let input = vec!["3", "A", "B", "C", "A|B|2", "B|C|3"];
        let result = find_shortest_path(input);
        assert_eq!(result, Ok("A-B-C".to_string()));
    }

    #[test]
    fn test_shortest_path_through_intermediate() {
        // Direct path A->D costs 100, but A->B->C->D costs 1+1+1=3
        let input = vec![
            "4", "A", "B", "C", "D", "A|B|1", "B|C|1", "C|D|1", "A|D|100",
        ];
        let result = find_shortest_path(input);
        assert_eq!(result, Ok("A-B-C-D".to_string()));
    }

    #[test]
    fn test_invalid_node_count() {
        let input = vec!["abc"];
        let result = find_shortest_path(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid number of nodes"));
    }

    #[test]
    fn test_not_enough_nodes() {
        let input = vec!["3", "A", "B"];
        let result = find_shortest_path(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not enough lines"));
    }

    #[test]
    fn test_duplicate_node_names() {
        let input = vec!["2", "A", "A", "A|A|5"];
        let result = find_shortest_path(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate node name"));
    }

    #[test]
    fn test_invalid_edge_format() {
        let input = vec!["2", "A", "B", "A|B"];
        let result = find_shortest_path(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid edge format"));
    }

    #[test]
    fn test_node_not_in_list() {
        let input = vec!["2", "A", "B", "A|C|5"];
        let result = find_shortest_path(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in node list"));
    }

    #[test]
    fn test_invalid_weight() {
        let input = vec!["2", "A", "B", "A|B|abc"];
        let result = find_shortest_path(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid weight"));
    }

    #[test]
    fn test_self_loop() {
        let input = vec!["2", "A", "B", "A|A|5"];
        let result = find_shortest_path(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Self-loop detected"));
    }

    #[test]
    fn test_empty_node_name() {
        let input = vec!["2", "", "B", "A|B|5"];
        let result = find_shortest_path(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty node name"));
    }

    #[test]
    fn test_zero_nodes() {
        let input = vec!["0"];
        let result = find_shortest_path(input);
        assert_eq!(result, Ok("-1".to_string()));
    }

    #[test]
    fn test_fibonacci_heap_simple() {
        // Test that Fibonacci heap version produces same results as binary heap
        let graph = vec![
            vec![(1, 2), (2, 4)], // node 0
            vec![(0, 2), (3, 1)], // node 1
            vec![(0, 4), (3, 5)], // node 2
            vec![(1, 1), (2, 5)], // node 3
        ];
        let path_binary = dijkstra_binary(0, 3, &graph);
        let path_fib = dijkstra_fibonacci(0, 3, &graph);
        assert_eq!(path_binary, path_fib);
        assert_eq!(path_binary, vec![0, 1, 3]);
    }

    #[test]
    fn test_fibonacci_heap_comprehensive() {
        // Comprehensive test comparing both implementations on various graphs
        let test_cases = vec![
            // Simple path
            (
                vec![vec![(1, 1)], vec![(2, 1)], vec![]],
                0,
                2,
                vec![0, 1, 2],
            ),
            // Disconnected nodes
            (vec![vec![], vec![]], 0, 1, vec![]),
            // Single node
            (vec![vec![]], 0, 0, vec![0]),
            // Complex graph
            (
                vec![
                    vec![(1, 4), (2, 2)],          // node 0
                    vec![(2, 1), (3, 5)],          // node 1
                    vec![(1, 1), (3, 8), (4, 10)], // node 2
                    vec![(4, 2)],                  // node 3
                    vec![],                        // node 4
                ],
                0,
                4,
                vec![0, 2, 1, 3, 4],
            ),
            // Graph with multiple paths
            (
                vec![
                    vec![(1, 1), (2, 5)], // node 0
                    vec![(2, 1), (3, 2)], // node 1
                    vec![(3, 3)],         // node 2
                    vec![],               // node 3
                ],
                0,
                3,
                vec![0, 1, 3],
            ),
        ];

        for (graph, start, end, expected_path) in test_cases {
            let path_binary = dijkstra_binary(start, end, &graph);
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
        let result = find_shortest_path(input);
        // Should work, but the last edge will overwrite the first
        assert_eq!(result, Ok("A-B".to_string()));
    }

    #[test]
    fn test_valid_inputs_from_files() {
        use std::fs;
        use std::path::Path;

        let testdata_dir = Path::new("testdata");
        if !testdata_dir.exists() {
            return;
        }

        for i in 0..=18 {
            let input_file = testdata_dir.join(format!("input{}.txt", i));
            let output_file = testdata_dir.join(format!("output{}.txt", i));

            if !input_file.exists() || !output_file.exists() {
                continue;
            }

            let input_content = fs::read_to_string(&input_file)
                .unwrap_or_else(|_| panic!("Failed to read {:?}", input_file));
            let input_lines: Vec<&str> = input_content.lines().collect();

            let expected_output = fs::read_to_string(&output_file)
                .unwrap_or_else(|_| panic!("Failed to read {:?}", output_file))
                .trim()
                .to_string();

            let result = find_shortest_path(input_lines);

            match result {
                Ok(actual) => {
                    assert_eq!(
                        actual, expected_output,
                        "Mismatch for input{}.txt: expected '{}', got '{}'",
                        i, expected_output, actual
                    );
                }
                Err(e) => {
                    panic!("input{}.txt should succeed but got error: {}", i, e);
                }
            }
        }
    }

    #[test]
    fn test_invalid_inputs_from_files() {
        use std::fs;
        use std::path::Path;

        let testdata_dir = Path::new("testdata");
        if !testdata_dir.exists() {
            return;
        }

        for i in 1..=6 {
            let input_file = testdata_dir.join(format!("invalid{}.txt", i));
            let error_file = testdata_dir.join(format!("error_invalid{}.txt", i));

            if !input_file.exists() || !error_file.exists() {
                continue;
            }

            let input_content = fs::read_to_string(&input_file)
                .unwrap_or_else(|_| panic!("Failed to read {:?}", input_file));
            let input_lines: Vec<&str> = input_content.lines().collect();

            let expected_error_full = fs::read_to_string(&error_file)
                .unwrap_or_else(|_| panic!("Failed to read {:?}", error_file))
                .trim()
                .to_string();

            let expected_error = expected_error_full
                .strip_prefix("Graph processing error: ")
                .unwrap_or(&expected_error_full)
                .to_string();

            let result = find_shortest_path(input_lines);

            match result {
                Ok(_) => {
                    panic!(
                        "invalid{}.txt should fail but succeeded. Expected error: {}",
                        i, expected_error
                    );
                }
                Err(actual_error) => {
                    assert_eq!(
                        actual_error, expected_error,
                        "Error message mismatch for invalid{}.txt:\n  Expected: {}\n  Got: {}",
                        i, expected_error, actual_error
                    );
                }
            }
        }
    }
}
