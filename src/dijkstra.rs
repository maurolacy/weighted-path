#![allow(non_snake_case)]

use std::collections::{BinaryHeap, HashMap};

// code goes here
// note: you are able to modify the parameter types
pub(crate) fn GraphChallenge(lines: Vec<&str>) -> Result<String, String> {
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

    // Build the adjacency matrix
    let A_line = vec![u32::MAX; N];
    let mut A = vec![A_line; N];
    
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
        
        A[*node_1_index][*node_2_index] = weight;
        A[*node_2_index][*node_1_index] = weight; // Bi-directional edges
    }
    //  println!("{A:#?}");

    // 2. Use Dijstra's to traverse the graph form the first node, and find
    // the shortest path to the last one.
    let path = dijkstra(0, N - 1, &A);
    //  println!("Path: {path:#?}");

    // 3. Return the shortest path; if no shortest path found, return -1.
    if path.len() <= 1 {
        return Ok("-1".to_string());
    }

    // Map path node ids to nodes
    let mut path_nodes = String::new();
    let mut first = true;
    for node_id in path {
        let node = nodes_reverse.get(&node_id)
            .ok_or_else(|| format!("Internal error: node ID {} not found in reverse map", node_id))?;
        if first {
            path_nodes = node.to_string();
            first = false;
            continue;
        }
        path_nodes = format!("{path_nodes}-{node}");
    }
    Ok(path_nodes)
}

fn dijkstra(start: usize, end: usize, graph: &[Vec<u32>]) -> Vec<usize> {
    let mut distances = vec![u32::MAX; graph.len()];
    distances[start] = 0;
    let mut previous = vec![None; graph.len()];
    let mut priority_queue = BinaryHeap::new();
    priority_queue.push((0, start));

    while let Some((current_distance, current_node)) = priority_queue.pop() {
        if distances[current_node] < current_distance {
            continue;
        }
        for (neighbor, weight) in graph[current_node].iter().enumerate() {
            if *weight == u32::MAX {
                continue;
            }
            let new_distance = current_distance + weight;
            if new_distance < distances[neighbor] {
                distances[neighbor] = new_distance;
                previous[neighbor] = Some(current_node);
                priority_queue.push((new_distance, neighbor));
            }
        }
    }
    let mut path = Vec::new();
    let mut current = end;
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
