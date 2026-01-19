/*

  README: Looking good, but couldn't finish debugging it because of repeated
  'Error executing code, timeout occurred.' errors.

  Not an issue with the code, as returning at the beginnig also produces the same error.
*/

#![allow(non_snake_case)]

use std::collections::HashMap;
use std::collections::VecDeque;

// code goes here
// note: you are able to modify the parameter types
pub(crate) fn GraphChallenge(lines: Vec<&str>) -> String {
//  println!("{:?}", lines);

  // 1. Parse the graph and build and adjacency matrix.
  if lines.len() == 0 {
    return "-1".to_string();
  }
  // Number of nodes
  let N = lines[0].parse::<u32>().unwrap() as usize;
//  println!("N: {N}");

  // Get the nodes
  let mut nodes = HashMap::new();
  let mut nodes_reverse = HashMap::new();
  for i in 1usize..N+1 {
    nodes.insert(lines[i], i-1); // node id map
    nodes_reverse.insert(i-1, lines[i]); // node map
  }

  // Build the adjacency matrix
  let A_line = vec![0u32; N];
  let mut A = vec![A_line; N];
  for i in 0..N {
    let line = lines[1+N+i];
    let (node_1, node_2_weight) = line.split_at(line.find("|").unwrap());
    let (node_2, weight) = node_2_weight[1..].split_at(node_2_weight[1..].find("|").unwrap());
    let weight = weight[1..].parse::<u32>().unwrap();
    let node_1_index = nodes.get(node_1).unwrap();
    let node_2_index = nodes.get(node_2).unwrap();
    A[*node_1_index][*node_2_index] = weight;
    A[*node_2_index][*node_1_index] = weight; // Bi-directional edges
  }
//  println!("{A:#?}");

  // 2. Use DFS of BFS (preferred) to traverse the graph form the first node, and find
  // the shortest path to the last one.
  let path = bfs(0, (N-1) as u32, &A);
//  println!("Path: {path:#?}");
 
  // 3. Return the shortest path; if no shortest path found, return -1.
  if path.len() == 0 {
    return "-1".to_string();
  }
  // Map path node ids to nodes
  let mut path_nodes = String::new();
  let mut first = true;
  for node_id in path {
    let node = nodes_reverse.get(&(node_id as usize)).unwrap();
    if first {
      path_nodes = node.to_string();
      first = false;
      continue;
    }
    path_nodes = format!("{path_nodes}-{node}");
  }
  return path_nodes;
//  return "-1".to_string();
}

 /* 
  Adapted / coiped from vTurbine/0-bfs.rs (GitHub).
*/
fn bfs(from: u32, to: u32, v: &Vec<Vec<u32>>) -> Vec<u32> {
    let mut frontier:   VecDeque<u32>   = VecDeque::new();
    let mut path:       Vec<u32>        = Vec::new();
    let mut visited:    Vec<(u32, u32)> = Vec::new();

    visited.resize(v.len(), (0xffff, 0));

    frontier.push_front(from);
    visited[from as usize] = (from, 0);

    /* Construct field for tracer */
    while !frontier.is_empty() {
        let p = frontier.pop_front().unwrap();

        // stop expanding if reached target point
        if p == to {
            break;
        }

        let nbrs_weights = &v[p as usize];

        for (i, w) in nbrs_weights.into_iter().enumerate() {
            if i == p as usize || *w == 0 {
              continue;
            }
            if visited[i].0 == 0xffff || visited[i].1 > *w {
                visited[i] = (p, *w);
                frontier.push_back(i as u32);
            }
        }
    }

    /* Follow the White rabbit */
    let mut p = to;
    path.push(p);

    while p != from {
//        println!("visited[{p}]: {:#?}", visited[p as usize]);
        p = visited[p as usize].0;
        path.push(p);
    }

    path.reverse();

    return path;
}

// keep this function call here   
/*
fn main() {
  GraphChallenge(coderbyteInternalStdinFunction(io::stdin()));
}
*/
