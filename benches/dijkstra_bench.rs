use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs;
use weigthed_path::dijkstra::{GraphChallenge, GraphChallengeWithDirection};

fn generate_test_graph(num_nodes: usize, edge_density: f64, directed: bool) -> Vec<String> {
    generate_test_graph_with_seed(num_nodes, edge_density, directed, None)
}

fn generate_test_graph_with_seed(num_nodes: usize, edge_density: f64, directed: bool, seed: Option<u64>) -> Vec<String> {
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

fn benchmark_graph_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_parsing");

    for size in [10, 50, 100, 500, 1000].iter() {
        let graph = generate_test_graph(*size, 0.1, false);
        let graph_refs: Vec<&str> = graph.iter().map(|s| s.as_str()).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &graph_refs,
            |b, graph| {
                b.iter(|| {
                    // Just parse, don't run algorithm
                    black_box(GraphChallenge(black_box(graph.clone())))
                })
            },
        );
    }

    group.finish();
}

fn benchmark_dijkstra_algorithm(c: &mut Criterion) {
    let mut group = c.benchmark_group("dijkstra_algorithm");

    for size in [10, 50, 100, 500, 1000, 2000].iter() {
        let graph = generate_test_graph(*size, 0.1, false);
        let graph_refs: Vec<&str> = graph.iter().map(|s| s.as_str()).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &graph_refs,
            |b, graph| {
                b.iter(|| {
                    black_box(GraphChallenge(black_box(graph.clone())))
                })
            },
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
            |b, graph| {
                b.iter(|| {
                    black_box(GraphChallenge(black_box(graph.clone())))
                })
            },
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
                |b, graph| {
                    b.iter(|| {
                        black_box(GraphChallenge(black_box(graph.clone())))
                    })
                },
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
            b.iter(|| {
                black_box(GraphChallengeWithDirection(black_box(graph.clone()), true))
            })
        },
    );

    // Benchmark as directed (bidirectional = false) - same input!
    group.bench_with_input(
        BenchmarkId::new("graph_type", "directed"),
        &graph_refs,
        |b, graph| {
            b.iter(|| {
                black_box(GraphChallengeWithDirection(black_box(graph.clone()), false))
            })
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
            |b, graph| {
                b.iter(|| {
                    black_box(GraphChallenge(black_box(graph.clone())))
                })
            },
        );
    }

    group.finish();
}

// Reference benchmark for quick performance checks
// Uses fixed seed (42) for reproducible results
fn benchmark_reference(c: &mut Criterion) {
    let mut group = c.benchmark_group("reference");
    group.sample_size(100); // Smaller sample for faster runs

    // Medium-sized graph: 500 nodes, 10% density - representative workload
    // Fixed seed ensures same graph every time for consistent benchmarking
    let graph = generate_test_graph_with_seed(5000, 0.1, false, Some(42));
    let graph_refs: Vec<&str> = graph.iter().map(|s| s.as_str()).collect();

    group.bench_function("500_nodes_10pct_density", |b| {
        b.iter(|| {
            black_box(GraphChallenge(black_box(graph_refs.clone())))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_graph_parsing,
    benchmark_dijkstra_algorithm,
    benchmark_different_densities,
    benchmark_real_world_graphs,
    benchmark_directed_vs_undirected,
    benchmark_directed_different_sizes
);

// Reference benchmark group for quick checks
criterion_group!(
    reference,
    benchmark_reference
);

criterion_main!(benches, reference);
