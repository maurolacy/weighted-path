#![allow(non_snake_case)]

use std::{
    env::args,
    fs::File,
    io::{self, BufRead, BufReader},
    process,
};

use weigthed_path::dijkstra::find_shortest_path;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    // First argument is the file to read
    let file_path = args().nth(1).ok_or("Usage: weighted_path <input_file>")?;

    let file = File::open(&file_path)
        .map_err(|e| format!("Failed to open file '{}': {}", file_path, e))?;

    let reader = BufReader::new(file);
    let lines = reader
        .lines()
        .collect::<Result<Vec<String>, io::Error>>()
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Find the shortest path
    let result = find_shortest_path(lines.iter().map(|line| line.as_str()).collect())
        .map_err(|e| format!("Graph processing error: {}", e))?;

    // Print the result
    println!("{}", result);
    Ok(())
}
