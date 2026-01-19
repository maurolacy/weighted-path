#![allow(non_snake_case)]

mod bfs;

use std::{fs::File, io::{self, BufRead, BufReader}};

use bfs::GraphChallenge;

fn main() {
    // Read all lines from test1.txt
    let file = File::open("test1.txt").unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines().collect::<Result<Vec<String>, io::Error>>().unwrap();
    //let lines = io::stdin().lines().collect::<Result<Vec<String>, io::Error>>().unwrap();

    // Call the GraphChallenge function
    let result = GraphChallenge(lines.iter().map(|line| line.as_str()).collect());

    // Print the result
    println!("{}", result);
}
