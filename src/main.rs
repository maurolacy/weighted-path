#![allow(non_snake_case)]

mod bfs;

use std::{env::args, fs::File, io::{self, BufRead, BufReader}};

use bfs::GraphChallenge;

fn main() {
    // First argument is the file to read
    let file_path = args().nth(1).unwrap();
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines().collect::<Result<Vec<String>, io::Error>>().unwrap();

    // Call the GraphChallenge function
    let result = GraphChallenge(lines.iter().map(|line| line.as_str()).collect());

    // Print the result
    println!("{}", result);
}
