#![allow(non_snake_case)]

use std::collections::{BinaryHeap, HashMap};

// code goes here
// note: you are able to modify the parameter types
pub(crate) fn GraphChallenge(lines: Vec<&str>) -> String {
    //  println!("{:?}", lines);

    // 1. Parse the graph and build and adjacency matrix.
    if lines.is_empty() {
        return "-1".to_string();
    }
    // Number of nodes
    let N = lines[0].parse::<u32>().unwrap() as usize;
    //  println!("N: {N}");
    if N == 0 {
        return "-1".to_string();
    }

    // Get the nodes
    let mut nodes = HashMap::new();
    let mut nodes_reverse = HashMap::new();
    for (i, &item) in lines.iter().enumerate().skip(1usize).take(N) {
        nodes.insert(item, i - 1); // node id map
        nodes_reverse.insert(i - 1, item); // node map
    }
    if N == 1 {
        return nodes_reverse.get(&0).unwrap().to_string(); // single node case
    }

    // Build the adjacency matrix
    let A_line = vec![u32::MAX; N];
    let mut A = vec![A_line; N];
    for line in lines.iter().skip(1 + N) {
        // Split the line into node 1, node 2, and weight
        let mut splits = line.split("|");
        let node_1 = splits.next().unwrap();
        let node_2 = splits.next().unwrap();
        let weight = splits.next().unwrap().parse::<u32>().unwrap();
        let node_1_index = nodes.get(node_1).unwrap();
        let node_2_index = nodes.get(node_2).unwrap();
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
        return "-1".to_string();
    }

    // Map path node ids to nodes
    let mut path_nodes = String::new();
    let mut first = true;
    for node_id in path {
        let node = nodes_reverse.get(&{ node_id }).unwrap();
        if first {
            path_nodes = node.to_string();
            first = false;
            continue;
        }
        path_nodes = format!("{path_nodes}-{node}");
    }
    path_nodes
    //  return "-1".to_string();
}

fn dijkstra(start: usize, end: usize, graph: &[Vec<u32>]) -> Vec<usize> {
    let mut distances = vec![u32::MAX; graph.len()];
    distances[start] = 0;
    let mut previous = vec![None; graph.len()];
    let mut priority_queue = BinaryHeap::new();
    priority_queue.push((0, start));

    while let Some((current_distance, current_node)) = priority_queue.pop() {
        // if current_node == end {
        //  break;
        //}
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
