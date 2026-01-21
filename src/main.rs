#![allow(non_snake_case)]

use std::env::args;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process;

use weighted_path::dijkstra::{
    find_shortest_path, find_shortest_path_fibonacci, find_shortest_path_fibonacci_unsafe,
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
    //   weighted_path [--heap <binary|fib|fib-unsafe>] <input_file>
    //
    // If no heap is specified, the default is `binary`.
    let mut args = args();
    // Skip program name
    let _ = args.next();

    let mut heap_impl = String::from("binary");
    let mut file_path: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--heap" => {
                heap_impl = args
                    .next()
                    .ok_or("Expected heap type after --heap (binary|fib|fib-unsafe)")?;
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

    let file_path =
        file_path.ok_or("Usage: weighted_path [--heap <binary|fib|fib-unsafe>] <input_file>")?;

    let file = File::open(&file_path)
        .map_err(|e| format!("Failed to open file '{}': {}", file_path, e))?;

    let reader = BufReader::new(file);
    let lines = reader
        .lines()
        .collect::<Result<Vec<String>, io::Error>>()
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Find the shortest path using the selected heap / Dijkstra implementation
    let line_refs: Vec<&str> = lines.iter().map(|line| line.as_str()).collect();
    let result = match heap_impl.as_str() {
        "binary" | "bin" => find_shortest_path(line_refs),
        "fib" => find_shortest_path_fibonacci(line_refs),
        "fib-unsafe" => find_shortest_path_fibonacci_unsafe(line_refs),
        other => {
            return Err(format!(
                "Unknown heap implementation '{}'. Expected one of: binary|bin, fib, fib-unsafe",
                other
            ));
        }
    }
    .map_err(|e| format!("Graph processing error: {}", e))?;

    // Print the result
    println!("{}", result);
    Ok(())
}
