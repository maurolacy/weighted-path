pub mod binary;
pub mod fib_safe;
pub mod fib_unsafe;

use std::collections::HashMap;

pub use binary::dijkstra;
pub use fib_safe::dijkstra_fibonacci_safe;
pub use fib_unsafe::dijkstra_fibonacci;

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
    find_shortest_path_directed(lines, true) // Default to undirected/bidirectional
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
    if lines.is_empty() {
        return Ok("-1".to_string());
    }

    // Number of nodes
    let num_nodes = lines[0].parse::<u32>().map_err(|_| {
        format!(
            "Invalid number of nodes: '{}' (expected a positive integer)",
            lines[0]
        )
    })? as usize;

    if num_nodes == 0 {
        return Ok("-1".to_string());
    }

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

    if num_nodes == 1 {
        return Ok(nodes_reverse
            .get(&0)
            .ok_or_else(|| "Internal error: single node not found in reverse map".to_string())?
            .to_string());
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

    // Use binary-heap Dijkstra by default
    let path = dijkstra(0, num_nodes - 1, &graph);

    if path.len() <= 1 {
        return Ok("-1".to_string());
    }

    // Map path node ids to nodes - optimized string building
    let mut path_parts = Vec::with_capacity(path.len());
    for node_id in path {
        let node = nodes_reverse.get(&node_id).ok_or_else(|| {
            format!(
                "Internal error: node ID {} not found in reverse map",
                node_id
            )
        })?;
        path_parts.push(*node);
    }
    Ok(path_parts.join("-"))
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
