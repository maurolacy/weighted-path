use std::env;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: generate_graph <num_nodes> [edge_density] [output_file]");
        eprintln!("  num_nodes: Number of nodes in the graph (required)");
        eprintln!("  edge_density: Probability of edge between nodes (0.0-1.0, default: 0.1)");
        eprintln!("  output_file: Output file path (default: stdout)");
        std::process::exit(1);
    }
    
    let num_nodes: usize = args[1]
        .parse()
        .expect("Number of nodes must be a positive integer");
    
    let edge_density: f64 = args.get(2)
        .map(|s| s.parse::<f64>().expect("Edge density must be a number between 0.0 and 1.0"))
        .unwrap_or(0.1)
        .clamp(0.0, 1.0);
    
    let output_file = args.get(3);
    
    let mut output: Box<dyn Write> = match output_file {
        Some(path) => Box::new(
            std::fs::File::create(path)
                .expect(&format!("Failed to create file: {}", path))
        ),
        None => Box::new(io::stdout()),
    };
    
    // Write number of nodes
    writeln!(output, "{}", num_nodes).unwrap();
    
    // Write node names (Node0, Node1, ...)
    for i in 0..num_nodes {
        writeln!(output, "Node{}", i).unwrap();
    }
    
    // Generate edges
    let mut rng = fastrand::Rng::new();
    let mut edge_count = 0;
    
    for i in 0..num_nodes {
        for j in (i + 1)..num_nodes {
            if rng.f64() < edge_density {
                // Generate random weight between 1 and 100
                let weight = rng.u32(1..=100);
                writeln!(output, "Node{}|Node{}|{}", i, j, weight).unwrap();
                edge_count += 1;
            }
        }
    }
    
    if output_file.is_none() {
        eprintln!("\nGenerated graph with {} nodes and {} edges", num_nodes, edge_count);
    } else {
        eprintln!("Generated graph with {} nodes and {} edges to {}", num_nodes, edge_count, output_file.unwrap());
    }
}
