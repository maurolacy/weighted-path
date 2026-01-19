use std::env;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: generate_graph <num_nodes> [edge_density] [output_file] [--directed]");
        eprintln!("  num_nodes: Number of nodes in the graph (required)");
        eprintln!("  edge_density: Probability of edge between nodes (0.0-1.0, default: 0.1)");
        eprintln!("  output_file: Output file path (default: stdout)");
        eprintln!("  --directed: Generate directed graph (default: undirected/bidirectional)");
        std::process::exit(1);
    }

    let num_nodes: usize = args[1]
        .parse()
        .expect("Number of nodes must be a positive integer");

    // Check for --directed flag
    let is_directed = args.iter().any(|arg| arg == "--directed");

    // Parse arguments, skipping --directed if present
    let mut arg_iter = args.iter().skip(2).filter(|&arg| arg != "--directed");

    let edge_density: f64 = arg_iter
        .next()
        .map(|s| {
            s.parse::<f64>()
                .expect("Edge density must be a number between 0.0 and 1.0")
        })
        .unwrap_or(0.1)
        .clamp(0.0, 1.0);

    let output_file = arg_iter.next();

    let mut output: Box<dyn Write> = match output_file {
        Some(path) => Box::new(
            std::fs::File::create(path).expect(&format!("Failed to create file: {}", path)),
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

    if is_directed {
        // For directed graphs, generate edges in both directions independently
        for i in 0..num_nodes {
            for j in 0..num_nodes {
                if i != j && rng.f64() < edge_density {
                    let weight = rng.u32(1..=100);
                    writeln!(output, "Node{}|Node{}|{}", i, j, weight).unwrap();
                    edge_count += 1;
                }
            }
        }
    } else {
        // For undirected graphs, only generate edges in one direction (upper triangle)
        // The parser will automatically make them bidirectional
        for i in 0..num_nodes {
            for j in (i + 1)..num_nodes {
                if rng.f64() < edge_density {
                    let weight = rng.u32(1..=100);
                    writeln!(output, "Node{}|Node{}|{}", i, j, weight).unwrap();
                    edge_count += 1;
                }
            }
        }
    }

    let graph_type = if is_directed {
        "directed"
    } else {
        "undirected"
    };
    if output_file.is_none() {
        eprintln!(
            "\nGenerated {} graph with {} nodes and {} edges",
            graph_type, num_nodes, edge_count
        );
    } else {
        eprintln!(
            "Generated {} graph with {} nodes and {} edges to {}",
            graph_type,
            num_nodes,
            edge_count,
            output_file.unwrap()
        );
    }
}
