#![allow(non_snake_case)]

use std::env::args;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process;

use weighted_path::dijkstra::{
    dijkstra_binary, dijkstra_fibonacci, dijkstra_fibonacci_unsafe, dijkstra_pairing,
    dijkstra_radix, find_shortest_path, find_shortest_path_fibonacci,
    find_shortest_path_fibonacci_unsafe, find_shortest_path_pairing, find_shortest_path_radix,
    parse_graph,
};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    // Simple, manual argument parsing:
    //
    // Usage:
    //   weighted_path [--heap <binary|fib|fib-unsafe|pairing|radix>] [--profile] <input_file>
    //
    // If no heap is specified, the default is `binary`.
    // If --profile is specified, runs Dijkstra multiple times for profiling.
    let mut args = args();
    // Skip program name
    let _ = args.next();

    let mut heap_impl = String::from("binary");
    let mut file_path: Option<String> = None;
    let mut profile_mode = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--heap" => {
                heap_impl = args.next().ok_or(
                    "Expected heap type after --heap (binary|fib|fib-unsafe|pairing|radix)",
                )?;
            }
            "-p" | "--profile" => {
                profile_mode = true;
            }
            // Treat first non-flag as file path
            _ if file_path.is_none() => {
                file_path = Some(arg);
            }
            _ => {
                return Err(format!("Unexpected argument: '{}'", arg));
            }
        }
    }

    let file_path = file_path.ok_or(
        "Usage: weighted_path [--heap <binary|fib|fib-unsafe|pairing|radix>] [--profile] <input_file>",
    )?;

    let file = File::open(&file_path)
        .map_err(|e| format!("Failed to open file '{}': {}", file_path, e))?;

    let reader = BufReader::new(file);
    let lines = reader
        .lines()
        .collect::<Result<Vec<String>, io::Error>>()
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let line_refs: Vec<&str> = lines.iter().map(|line| line.as_str()).collect();

    if profile_mode {
        // Profile mode: parse once, run Dijkstra many times
        let parsed =
            parse_graph(&line_refs, true).map_err(|e| format!("Graph processing error: {}", e))?;
        let num_nodes = parsed.graph.len();

        if num_nodes == 0 {
            return Err("Empty graph".to_string());
        }

        println!(
            "Running Dijkstra {} times on graph with {} nodes using {} heap...",
            100, num_nodes, heap_impl
        );

        match heap_impl.as_str() {
            "binary" | "bin" => {
                for _ in 0..100 {
                    let _path = dijkstra_binary(0, num_nodes - 1, &parsed.graph);
                }
            }
            "fib" => {
                for _ in 0..100 {
                    let _path = dijkstra_fibonacci(0, num_nodes - 1, &parsed.graph);
                }
            }
            "fib-unsafe" => {
                for _ in 0..100 {
                    let _path = dijkstra_fibonacci_unsafe(0, num_nodes - 1, &parsed.graph);
                }
            }
            "pairing" | "pair" => {
                for _ in 0..100 {
                    let _path = dijkstra_pairing(0, num_nodes - 1, &parsed.graph);
                }
            }
            "radix" => {
                for _ in 0..100 {
                    let _path = dijkstra_radix(0, num_nodes - 1, &parsed.graph);
                }
            }
            other => {
                return Err(format!(
                    "Unknown heap implementation '{}'. Expected one of: binary|bin, fib, fib-unsafe, pairing|pair, radix",
                    other
                ));
            }
        }

        println!("Done");
    } else {
        // Normal mode: find shortest path and print result
        let result = match heap_impl.as_str() {
            "binary" | "bin" => find_shortest_path(line_refs),
            "fib" => find_shortest_path_fibonacci(line_refs),
            "fib-unsafe" => find_shortest_path_fibonacci_unsafe(line_refs),
            "pairing" | "pair" => find_shortest_path_pairing(line_refs),
            "radix" => find_shortest_path_radix(line_refs),
            other => {
                return Err(format!(
                    "Unknown heap implementation '{}'. Expected one of: binary|bin, fib, fib-unsafe, pairing|pair, radix",
                    other
                ));
            }
        }
        .map_err(|e| format!("Graph processing error: {}", e))?;

        // Print the result
        println!("{}", result);
    }

    Ok(())
}
