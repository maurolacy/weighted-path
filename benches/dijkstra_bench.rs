use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::fs;
use weighted_path::dijkstra::{
    dijkstra_fibonacci, dijkstra_fibonacci_unsafe, dijkstra_pairing, find_shortest_path,
    find_shortest_path_directed, parse_graph,
};

fn generate_test_graph(num_nodes: usize, edge_density: f64, directed: bool) -> Vec<String> {
    generate_test_graph_with_seed(num_nodes, edge_density, directed, None)
}

fn generate_test_graph_with_seed(
    num_nodes: usize,
    edge_density: f64,
    directed: bool,
    seed: Option<u64>,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(num_nodes.to_string());

    // Add node names
    for i in 0..num_nodes {
        lines.push(format!("Node{}", i));
    }

    // Generate edges with optional fixed seed
    let mut rng = if let Some(s) = seed {
        fastrand::Rng::with_seed(s)
    } else {
        fastrand::Rng::new()
    };
    if directed {
        // For directed graphs, generate edges in both directions independently
        for i in 0..num_nodes {
            for j in 0..num_nodes {
                if i != j && rng.f64() < edge_density {
                    let weight = rng.u32(1..=100);
                    lines.push(format!("Node{}|Node{}|{}", i, j, weight));
                }
            }
        }
    } else {
        // For undirected graphs, only generate edges in one direction (upper triangle)
        // The parser will make them bidirectional
        for i in 0..num_nodes {
            for j in (i + 1)..num_nodes {
                if rng.f64() < edge_density {
                    let weight = rng.u32(1..=100);
                    lines.push(format!("Node{}|Node{}|{}", i, j, weight));
                }
            }
        }
    }

    lines
}

fn benchmark_dijkstra_algorithm(c: &mut Criterion) {
    let mut group = c.benchmark_group("dijkstra_algorithm");

    for size in [10, 50, 100, 500, 1000, 2000].iter() {
        let graph = generate_test_graph(*size, 0.1, false);
        let graph_refs: Vec<&str> = graph.iter().map(|s| s.as_str()).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &graph_refs,
            |b, graph| b.iter(|| black_box(find_shortest_path(black_box(graph.clone())))),
        );
    }

    group.finish();
}

fn benchmark_different_densities(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_density");
    let num_nodes = 500;

    for density in [0.01, 0.05, 0.1, 0.2, 0.5].iter() {
        let graph = generate_test_graph(num_nodes, *density, false);
        let graph_refs: Vec<&str> = graph.iter().map(|s| s.as_str()).collect();

        group.bench_with_input(
            BenchmarkId::new("density", format!("{:.2}", density)),
            &graph_refs,
            |b, graph| b.iter(|| black_box(find_shortest_path(black_box(graph.clone())))),
        );
    }

    group.finish();
}

fn benchmark_real_world_graphs(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_graphs");

    // Test with actual test files if they exist
    let test_files = [
        "testdata/input1.txt",
        "testdata/input12.txt",
        "testdata/input17.txt",
    ];

    for file_path in test_files.iter() {
        if let Ok(content) = fs::read_to_string(file_path) {
            let lines: Vec<&str> = content.lines().collect();

            group.bench_with_input(
                BenchmarkId::from_parameter(file_path),
                &lines,
                |b, graph| b.iter(|| black_box(find_shortest_path(black_box(graph.clone())))),
            );
        }
    }

    group.finish();
}

fn benchmark_directed_vs_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("directed_vs_undirected");
    let num_nodes = 500;
    let edge_density = 0.1;

    // Generate a single graph and test it in both modes
    let graph = generate_test_graph(num_nodes, edge_density, false);
    let graph_refs: Vec<&str> = graph.iter().map(|s| s.as_str()).collect();

    // Benchmark as undirected (bidirectional = true)
    group.bench_with_input(
        BenchmarkId::new("graph_type", "undirected"),
        &graph_refs,
        |b, graph| {
            b.iter(|| black_box(find_shortest_path_directed(black_box(graph.clone()), true)))
        },
    );

    // Benchmark as directed (bidirectional = false) - same input!
    group.bench_with_input(
        BenchmarkId::new("graph_type", "directed"),
        &graph_refs,
        |b, graph| {
            b.iter(|| black_box(find_shortest_path_directed(black_box(graph.clone()), false)))
        },
    );

    group.finish();
}

fn benchmark_directed_different_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("directed_graphs");
    let edge_density = 0.05; // Lower density for directed graphs to keep edge count reasonable

    for size in [100, 500, 1000].iter() {
        let graph = generate_test_graph(*size, edge_density, true);
        let graph_refs: Vec<&str> = graph.iter().map(|s| s.as_str()).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &graph_refs,
            |b, graph| b.iter(|| black_box(find_shortest_path(black_box(graph.clone())))),
        );
    }

    group.finish();
}

// Reference benchmark for quick performance checks
// Uses fixed seed (42) for reproducible results on specific scenarios:
// - Sparse graph: 500 nodes, density 0.1
// - Dense graph: 1000 nodes, density 0.3
// Each benchmark runs the three Dijkstra variants on the *same* generated graph.
fn benchmark_reference(c: &mut Criterion) {
    let mut group = c.benchmark_group("reference");
    group.sample_size(80); // Smaller sample for faster runs, but enough for stable ratios

    let test_cases = vec![
        (500, 0.1, "sparse_500_d0.1"),
        (1000, 0.3, "dense_1000_d0.3"),
    ];

    for (nodes, density, name) in test_cases {
        // Generate a reproducible graph for this (nodes, density) pair
        let graph = generate_test_graph_with_seed(nodes, density, false, Some(42));
        let graph_refs: Vec<&str> = graph.iter().map(|s| s.as_str()).collect();

        // Build adjacency list once, then reuse it for the Fibonacci variants
        let parsed =
            parse_graph(&graph_refs, true).expect("Failed to parse generated graph for benchmark");
        let adj_list = parsed.graph;

        // Binary heap: end-to-end (parse + Dijkstra via public API)
        group.bench_with_input(
            BenchmarkId::new("binary_heap", name),
            &graph_refs,
            |b, lines| {
                b.iter(|| {
                    black_box(find_shortest_path(black_box(
                        lines.clone(), // Vec<&str> clone is cheap
                    )))
                })
            },
        );

        // Safe Fibonacci heap: Dijkstra over pre-built adjacency list
        group.bench_with_input(BenchmarkId::new("fib_safe", name), &adj_list, |b, graph| {
            b.iter(|| black_box(dijkstra_fibonacci(0, graph.len() - 1, black_box(graph))))
        });

        // Unsafe Fibonacci heap: Dijkstra over same adjacency list
        group.bench_with_input(
            BenchmarkId::new("fib_unsafe", name),
            &adj_list,
            |b, graph| {
                b.iter(|| {
                    black_box(dijkstra_fibonacci_unsafe(
                        0,
                        graph.len() - 1,
                        black_box(graph),
                    ))
                })
            },
        );

        // Pairing heap: Dijkstra over same adjacency list
        group.bench_with_input(BenchmarkId::new("pairing", name), &adj_list, |b, graph| {
            b.iter(|| black_box(dijkstra_pairing(0, graph.len() - 1, black_box(graph))))
        });
    }

    group.finish();
}

// Benchmark comparing binary heap vs Fibonacci heap
fn benchmark_heap_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("heap_comparison");

    // Test with different graph sizes and densities
    let test_cases = vec![
        (500, 0.1, "sparse_500"),
        (1000, 0.1, "sparse_1000"),
        (500, 0.3, "dense_500"),
        (1000, 0.3, "dense_1000"),
    ];

    for (nodes, density, name) in test_cases {
        let graph = generate_test_graph_with_seed(nodes, density, false, Some(42));
        let graph_refs: Vec<&str> = graph.iter().map(|s| s.as_str()).collect();

        // Build adjacency list for direct Dijkstra calls using the shared parser
        let parsed =
            parse_graph(&graph_refs, true).expect("Failed to parse generated graph for benchmark");
        let adj_list = parsed.graph;

        // Benchmark binary heap (via find_shortest_path)
        group.bench_with_input(
            BenchmarkId::new("binary_heap", name),
            &graph_refs,
            |b, graph| b.iter(|| black_box(find_shortest_path(black_box(graph.clone())))),
        );

        // Benchmark Fibonacci heap
        group.bench_with_input(
            BenchmarkId::new("fibonacci_heap", name),
            &adj_list,
            |b, graph| {
                b.iter(|| black_box(dijkstra_fibonacci(0, graph.len() - 1, black_box(graph))))
            },
        );

        // Benchmark Pairing heap
        group.bench_with_input(
            BenchmarkId::new("pairing_heap", name),
            &adj_list,
            |b, graph| b.iter(|| black_box(dijkstra_pairing(0, graph.len() - 1, black_box(graph)))),
        );
    }

    group.finish();
}

fn benchmark_fibonacci_heap_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci_heap_comparison");

    let test_cases = vec![
        (500, 0.1, "sparse_500"),
        (1000, 0.1, "sparse_1000"),
        (500, 0.3, "dense_500"),
        (1000, 0.3, "dense_1000"),
    ];

    for (nodes, density, name) in test_cases {
        let graph = generate_test_graph_with_seed(nodes, density, false, Some(42));
        let graph_refs: Vec<&str> = graph.iter().map(|s| s.as_str()).collect();

        // Build adjacency list using the shared parser
        let parsed =
            parse_graph(&graph_refs, true).expect("Failed to parse generated graph for benchmark");
        let adj_list = parsed.graph;

        // Verify all implementations produce the same result
        let res_safe = dijkstra_fibonacci(0, adj_list.len() - 1, &adj_list);
        let res_unsafe = dijkstra_fibonacci_unsafe(0, adj_list.len() - 1, &adj_list);
        let res_pairing = dijkstra_pairing(0, adj_list.len() - 1, &adj_list);

        assert_eq!(
            res_unsafe, res_safe,
            "Mismatch for {}: unsafe={:?}, safe={:?}",
            name, res_unsafe, res_safe
        );
        assert_eq!(
            res_pairing, res_safe,
            "Mismatch for {}: pairing={:?}, safe={:?}",
            name, res_pairing, res_safe
        );

        // Benchmark unsafe (raw pointers) version
        group.bench_with_input(
            BenchmarkId::new("unsafe_raw_pointers", name),
            &adj_list,
            |b, graph| {
                b.iter(|| {
                    black_box(dijkstra_fibonacci_unsafe(
                        0,
                        graph.len() - 1,
                        black_box(graph),
                    ))
                })
            },
        );

        // Benchmark safe (Rc<RefCell>) version
        group.bench_with_input(
            BenchmarkId::new("safe_rc_refcell", name),
            &adj_list,
            |b, graph| {
                b.iter(|| black_box(dijkstra_fibonacci(0, graph.len() - 1, black_box(graph))))
            },
        );

        // Benchmark Pairing heap
        group.bench_with_input(
            BenchmarkId::new("pairing_heap", name),
            &adj_list,
            |b, graph| b.iter(|| black_box(dijkstra_pairing(0, graph.len() - 1, black_box(graph)))),
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_dijkstra_algorithm,
    benchmark_different_densities,
    benchmark_real_world_graphs,
    benchmark_directed_vs_undirected,
    benchmark_directed_different_sizes
);

// Reference benchmark group for quick checks
criterion_group!(reference, benchmark_reference);

// Heap comparison benchmark group
criterion_group!(
    heap_compare,
    benchmark_heap_comparison,
    benchmark_fibonacci_heap_comparison
);

criterion_main!(benches, reference, heap_compare);
