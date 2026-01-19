#!/bin/bash
# Script to generate graphs of various sizes for benchmarking

set -e

BENCH_DIR="testdata/bench"

# Create benchmark directory if it doesn't exist
mkdir -p "$BENCH_DIR"

echo "Generating benchmark graphs..."

# Generate graphs of various sizes
# Format: nodes, edge_density, output_file
declare -a graphs=(
    "100 0.1 bench_100.txt"
    "500 0.1 bench_500.txt"
    "1000 0.1 bench_1000.txt"
    "2000 0.05 bench_2000.txt"
    "5000 0.02 bench_5000.txt"
    "10000 0.01 bench_10000.txt"
)

for graph_spec in "${graphs[@]}"; do
    read -r nodes density filename <<< "$graph_spec"
    output_file="$BENCH_DIR/$filename"
    
    echo "Generating graph with $nodes nodes (density: $density)..."
    cargo run --quiet --bin generate_graph -- "$nodes" "$density" "$output_file" 2>&1 | grep "Generated"
done

echo ""
echo "Benchmark graphs generated in $BENCH_DIR/"
echo "You can now run: cargo bench --bench dijkstra_bench"
