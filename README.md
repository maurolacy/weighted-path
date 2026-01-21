# Weighted Path

A Rust implementation of Dijkstra's shortest path algorithm that finds the optimal path between two nodes in a weighted, undirected graph.

## Description

This program takes a graph definition as input and finds the shortest weighted path from the first node to the last node using Dijkstra's algorithm. The graph is undirected (edges work in both directions) and edges have positive weights.

## Building

```bash
cargo build --release
```

## Usage

```bash
cargo run --bin weighted_path <input_file>
```

Or using the compiled binary:

```bash
./target/release/weighted_path <input_file>
```

By default, the binary uses the **binary heap** implementation of Dijkstra.

### Selecting the Dijkstra / heap implementation (lab mode)

The binary also exposes a small "lab mode" flag that lets you choose which
underlying Dijkstra / heap implementation to use. This is mainly intended for
experimentation, benchmarking, and regression checks; for typical one-off runs
you will not notice a visible difference, because **file I/O and graph parsing
dominate the total runtime**.

Usage:

```bash
weighted_path [--heap <binary|bin|fib|fib-unsafe>] <input_file>
```

- `binary` / `bin`: Standard binary-heap Dijkstra (default)
- `fib`: Safe Fibonacci heap (`dijkstra_fibonacci`, `Rc<RefCell>`-based)
- `fib-unsafe`: Unsafe Fibonacci heap (`dijkstra_fibonacci_unsafe`, raw pointers)

Examples:

```bash
# Default (binary heap)
weighted_path graph.txt

# Explicit binary heap
weighted_path --heap binary graph.txt

# Safe Fibonacci heap
weighted_path --heap fib graph.txt

# Unsafe Fibonacci heap
weighted_path --heap fib-unsafe graph.txt
```

## Input Format

The input file should follow this format:

1. **First line**: Number of nodes (N) as a positive integer
2. **Next N lines**: Node names (one per line)
3. **Remaining lines**: Edges in the format `node1|node2|weight`
   - `node1` and `node2` are node names (must match nodes defined above)
   - `weight` is a positive integer representing the edge weight
   - Edges are bidirectional

### Example Input

```
4
A
B
C
D
A|B|2
C|B|11
C|D|3
B|D|2
```

## Output Format

The program outputs the shortest path from the first node to the last node as a dash-separated string of node names, or `-1` if no path exists.

### Example Output

```
A-B-D
```

This means the shortest path from node `A` to node `D` goes through node `B`.

## Running Tests

```bash
cargo test --lib
```

This runs unit tests, including file-based test cases in `testdata/` (when present).

## Algorithm

The program uses Dijkstra's algorithm to find the shortest path:

1. Parse the graph and build an adjacency list
2. Use Dijkstra's algorithm to find the shortest path from the first node to the last node
3. Return the path as a dash-separated string, or `-1` if no path exists

## Edge Cases

- **Single node**: Returns the node name itself
- **No path exists**: Returns `-1`
- **Empty input**: Returns `-1`
- **Zero nodes**: Returns `-1`

## Test Data

Test cases are located in the `testdata/` directory:
- `input0.txt` through `input18.txt`: Test input files
- `output0.txt` through `output18.txt`: Expected output files

## Error Handling

The program validates input and provides clear error messages for:
- Missing command-line arguments
- File not found or unreadable
- Invalid number format
- Missing nodes in edge definitions
- Malformed edge lines
- Duplicate node names

## Benchmarking

The project includes benchmarking support using Criterion.rs to measure performance across different graph sizes and edge densities.

### Generating Test Graphs

A graph generator utility is included to create large graphs for benchmarking:

```bash
# Generate a graph with 1000 nodes and 10% edge density
cargo run --bin generate_graph 1000 0.1 output.txt
```

**Graph Generator Usage:**
```bash
cargo run --bin generate_graph <num_nodes> [edge_density] [output_file] [--directed]
```

- `num_nodes`: Number of nodes in the graph (required)
- `edge_density`: Probability of edge between nodes (0.0-1.0, default: 0.1)
- `output_file`: Output file path (default: stdout)
- `--directed`: Generate a directed graph (default: undirected/bidirectional)

**Examples:**
```bash
# Generate undirected graph with 1000 nodes
cargo run --bin generate_graph 1000 0.1 large_graph.txt

# Generate directed graph with 500 nodes
cargo run --bin generate_graph 500 0.1 directed_graph.txt --directed
```

**Note on Directed vs Undirected:**
- **Default behavior**: By default, edges are treated as bidirectional (undirected). When an edge `A|B|w` is specified, both A→B and B→A are created with weight `w`.
- **Directed graphs**: The `find_shortest_path_directed` function accepts a `bidirectional` boolean parameter. When `false`, edges are one-way only (A→B exists, but B→A does not unless explicitly specified).
- **Input format**: The same input can be processed as either directed or undirected by using `find_shortest_path_directed(lines, bidirectional)`. If a directed graph is processed with `bidirectional=true`, reverse edges will be created.
- **Benchmarking**: This allows testing the same graph structure in both modes for fair performance comparison. The benchmark suite includes a `directed_vs_undirected` benchmark that tests the same graph with both settings.
- **Performance**: Directed graphs typically have fewer edges (only forward direction), while undirected graphs have symmetric edges. The same input processed as undirected will have more edges and may be slightly slower.

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --bench dijkstra_bench

# Run reference benchmark (quick performance check)
cargo bench --bench dijkstra_bench -- reference

# Run specific benchmark group
cargo bench --bench dijkstra_bench -- dijkstra_algorithm
```

The benchmarks test:
- **Graph parsing performance** across different graph sizes (10, 50, 100, 500, 1000 nodes)
- **Dijkstra algorithm performance** across different graph sizes (10, 50, 100, 500, 1000, 2000 nodes)
- **Edge density impact** on performance (0.01, 0.05, 0.1, 0.2, 0.5 density with 500 nodes)
- **Directed vs Undirected graphs** comparison: Same graph structure tested with `bidirectional=true` (undirected) and `bidirectional=false` (directed) for fair comparison
- **Directed graph performance** across different sizes (100, 500, 1000 nodes)
- **Real-world graph performance** using actual test files

Benchmark results are saved in `target/criterion/` and include HTML reports with detailed statistics and graphs.

### Performance Characteristics

The current implementation uses:
- **Adjacency list**: O(V + E) space complexity where V is vertices and E is edges (much better for sparse graphs)
- **Dijkstra's algorithm**: O((V + E) log V) time complexity with binary heap
- **Binary heap priority queue**: Used for efficient minimum distance extraction

**Performance Optimizations:**
- **Adjacency list**: Only stores actual edges, not empty slots. Much more efficient for sparse graphs (typical case)
- **Early termination**: Algorithm stops immediately when the target node is reached
- **Optimized string building**: Uses `join()` instead of repeated `format!()` calls
- **Efficient priority queue**: Uses `Reverse` wrapper to convert BinaryHeap (max-heap) into a min-heap (zero-cost in release builds)

**Space Comparison:**
- **Adjacency matrix**: O(V²) - always allocates space for all possible edges
- **Adjacency list**: O(V + E) - only stores actual edges
- For a sparse graph with 1000 nodes and 5000 edges: matrix = 1M entries, list = 6000 entries

### Fibonacci Heap Implementation and Results

**Fibonacci heap implementations:**
- Fibonacci heap implementations are available for benchmarking:
  - `dijkstra_fibonacci`: safe `Rc<RefCell>` heap (clear/ergonomic but slower)
  - `dijkstra_fibonacci_unsafe`: raw-pointer heap (fastest, but uses `unsafe`)
- Provides O(E + V log V) amortized time complexity vs O((V + E) log V) for binary heap
- **Performance improvements observed:**
  - ~20x faster for sparse graphs (500 nodes)
  - >100x faster for dense graphs (1000 nodes)
- The unsafe variant uses raw pointers to avoid `RefCell` borrow conflicts and overhead, and implements `decrease_key` for optimal performance

For very large graphs (10,000+ nodes), the Fibonacci heap implementation is recommended for best performance.

**Summary of benchmark results (indicative):**

These numbers come from the Criterion benchmark suite in this repository and are
intended as an order-of-magnitude guide rather than exact guarantees:

| **Graph type**        | **Nodes** | **Density** | **Binary heap (baseline)** | **Safe Fib heap (`fib`)** | **Unsafe Fib heap (`fib-unsafe`)** |
|-----------------------|-----------|-------------|----------------------------|---------------------------|-------------------------------------|
| Sparse graph          | 500       | 0.1         | 1×                         | ~5–10× faster             | ~10–20× faster                      |
| Dense graph           | 1000      | 0.3         | 1×                         | >30× faster               | >100× faster                        |

Exact timings may vary by machine and compiler version; for precise numbers,
run the benchmarks locally:

```bash
cargo bench --bench dijkstra_bench -- reference heap_comparison
```

Criterion will generate detailed HTML reports (including plots) under
`target/criterion/`, which you can open in a browser for full performance
breakdowns.

## Module Layout

- `src/dijkstra/`: Dijkstra implementations (binary heap, Fibonacci heap safe/unsafe) + parsing/validation + tests
- `src/fibonacci/`: Fibonacci heap implementations
  - `heap.rs`: safe heap (`Node`, `FibonacciHeap`)
  - `heap_unsafe.rs`: unsafe heap (`UnsafeNode`, `UnsafeFibonacciHeap`)
