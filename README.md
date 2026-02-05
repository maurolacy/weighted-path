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
weighted_path [--heap <binary|bin|fib|fib-unsafe|pairing|pair|radix|dial>] <input_file>
```

- `binary` / `bin`: Standard binary-heap Dijkstra (default).
- `fib`: Fibonacci heap (`dijkstra_fibonacci`, `Rc<RefCell>`-based).
- `fib-unsafe`: Unsafe Fibonacci heap (`dijkstra_fibonacci_unsafe`, raw pointers).
- `pairing` / `pair`: Pairing heap (`dijkstra_pairing`).
- `radix`: Radix heap (`dijkstra_radix`).
- `dial`: Dial's algorithm (`dijkstra_dial`, bucket-based).

Examples:

```bash
# Default (binary heap)
weighted_path graph.txt

# Explicit binary heap
weighted_path --heap binary graph.txt

# Fibonacci heap
weighted_path --heap fib graph.txt

# Unsafe Fibonacci heap
weighted_path --heap fib-unsafe graph.txt

# Pairing heap
weighted_path --heap pairing graph.txt

# Radix heap
weighted_path --heap radix graph.txt

# Dial's algorithm
weighted_path --heap dial graph.txt
```

## Input Format

The input file should follow this format:

1. **First line**: Number of nodes (N) as a positive integer.
2. **Next N lines**: Node names (one per line).
3. **Remaining lines**: Edges in the format `node1|node2|weight`.
   - `node1` and `node2` are node names (must match nodes defined above).
   - `weight` is a positive integer representing the edge weight.
   - Edges are bidirectional.

### Example Input

```text
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

The program outputs the shortest path from the first node to the last node as a dash-separated
string of node names, followed by the total distance in parentheses, or `-1` if no path exists.

### Example Output

```text
A-B-D (weight: 4)
```

This means the shortest path from node `A` to node `D` goes through node `B` and has total
weight `4`.

## Running Tests

```bash
cargo test --lib
```

This runs unit tests, including file-based test cases in `testdata/` (when present).

## Algorithm

The program uses Dijkstra's algorithm to find the shortest path:

1. Parse the graph and build an adjacency list.
2. Use Dijkstra's algorithm to find the shortest path from the first node to the last node and
   its total distance.
3. Return the path as a dash-separated string together with the total distance, or `-1` if no
   path exists.

## Edge Cases

- **Single node**: Returns the node name itself with distance 0 (e.g. `A (weight: 0)`).
- **No path exists**: Returns `-1`.
- **Empty input**: Returns `-1`.
- **Zero nodes**: Returns `-1`.

## Test Data

Test cases are located in the `testdata/` directory:

- `input0.txt` through `input18.txt`: Test input files.
- `output0.txt` through `output18.txt`: Expected output files.

## Error Handling

The program validates input and provides clear error messages for:

- Missing command-line arguments.
- File not found or unreadable.
- Invalid number format.
- Missing nodes in edge definitions.
- Malformed edge lines.
- Duplicate node names.

## Benchmarking

The project includes benchmarking support using Criterion.rs to measure performance across different graph sizes, edge densities, and heap impls.

### Generating Test Graphs

A graph generator utility is included to create large graphs for benchmarking.

**Graph Generator Usage:**

```bash
cargo run --bin generate_graph <num_nodes> [edge_density] [output_file] [--directed]
```

- `num_nodes`: Number of nodes in the graph (required).
- `edge_density`: Probability of edge between nodes (0.0-1.0, default: 0.1).
- `output_file`: Output file path (default: stdout).
- `--directed`: Generate a directed graph (default: undirected/bidirectional).

**Examples:**

```bash
# Generate undirected graph with 1000 nodes
cargo run --bin generate_graph 1000 0.1 large_graph.txt

# Generate directed graph with 500 nodes
cargo run --bin generate_graph 500 0.1 directed_graph.txt --directed
```

**Note on Directed vs Undirected:**

- **Default behaviour**: By default, edges are treated as bidirectional (undirected). When an edge `A|B|w` is specified, both `A→B` and `B→A` are created with weight `w`.
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

- **Graph parsing performance** across different graph sizes (10, 50, 100, 500, 1000 nodes).
- **Dijkstra algorithm performance** across different graph sizes (10, 50, 100, 500, 1000, 2000 nodes).
- **Edge density impact** on performance (0.01, 0.05, 0.1, 0.2, 0.5 density with 500 nodes).
- **Directed vs Undirected graphs** comparison: Same graph structure tested with `bidirectional=true` (undirected) and `bidirectional=false` (directed) for fair comparison.
- **Directed graph performance** across different sizes (100, 500, 1000 nodes).
- **Real-world graph performance** using actual test files.

Benchmark results are saved in `target/criterion/` and include HTML reports with detailed statistics and graphs.

### Performance Characteristics

The current implementation uses:

- **Adjacency list**: O(V + E) space complexity where V is vertices and E is edges (much better for sparse graphs).
- **Dijkstra's algorithm**: O((V + E) log V) time complexity with binary heap.
- **Binary heap priority queue**: Used for efficient minimum distance extraction.
- **Efficient priority queue**: Uses `Reverse` wrapper to convert BinaryHeap (max-heap) into a min-heap (zero-cost in release builds).

### Advanced Heap Implementations

**Multiple heap implementations are available:**

- **Binary heap** (`dijkstra_binary`): Standard implementation, good general-purpose choice.
- **Fibonacci heap** implementations:
  - `dijkstra_fibonacci`: `Rc<RefCell>`-based heap (memory-safe but slower).
  - `dijkstra_fibonacci_unsafe`: raw-pointer heap (fastest, but uses `unsafe`).
- **Pairing heap** (`dijkstra_pairing`): Simpler than Fibonacci, often faster in practice.
- **Radix heap** (`dijkstra_radix`): Specialized for non-decreasing integer keys, excellent for Dijkstra's algorithm.
- **Dial's algorithm** (`dijkstra_dial`): Bucket-based algorithm optimised for small integer edge weights (1..=100). Very efficient for dense graphs.

**Complexity:**

- Binary heap: O((V + E) log V).
- Fibonacci heap: O(E + V log V) amortized.
- Pairing heap: O(E + V log V) amortized.
- Radix heap: O(E + V log C) amortized, where C is the key range (for integer keys).
- Dial's algorithm: O(V + E + C), where C is the maximum distance. Optimal when C is small compared to V log V.

**Performance characteristics:**

- Fibonacci and Pairing heaps provide significant speedups on dense graphs.
- Pairing heap often outperforms Fibonacci heap in practice due to lower constant factors.
- The unsafe Fibonacci variant is fastest but requires careful memory management.
- Radix heap is particularly effective when edge weights are bounded integers (as in this implementation, where weights are `u32` in range 1..=100).
- Dial's algorithm excels with small integer edge weights and dense graphs. Its bucket-based approach avoids priority queue overhead, making it competitive with or faster than advanced heap structures for this use case.

For very large graphs (10,000+ nodes), the advanced heap implementations are recommended for best performance.

**Summary of benchmark results (indicative):**

These numbers come from the Criterion benchmark suite in this repository and are
intended as an order-of-magnitude guide rather than exact guarantees:

| **Graph type**        | **Nodes** | **Density** | **Binary heap (baseline)** | **Fib heap (`fib`)** | **Unsafe Fib heap (`fib-unsafe`)** | **Pairing heap (`pairing`)** | **Radix heap (`radix`)** | **Dial's algorithm (`dial`)** |
|-----------------------|-----------|-------------|----------------------------|----------------------|------------------------------------|------------------------------|--------------------------|-------------------------------|
| Sparse graph          | 500       | 0.1         | 1×                         | ~5–10× faster        | ~20× faster                        | ~10-15× faster               | ~50-60× faster           | ~50-60× faster                |
| Dense graph           | 1000      | 0.3         | 1×                         | >30× faster          | >100× faster                       | ~30-35× faster               | ~50× faster              | >150× faster                  |

Exact timings may vary by machine and compiler version; for precise numbers,
run the benchmarks locally:

```bash
cargo bench --bench dijkstra_bench -- reference
```

Criterion will generate detailed HTML reports (including plots) under
`target/criterion/`, which you can open in a browser for full performance
breakdowns.

## Module Layout

- `src/dijkstra/`: Core Dijkstra implementation and heap-specific wrappers.
  - `mod.rs`: Generic `dijkstra<Q: PriorityQueue>` function (works with any heap).
  - `heap_trait.rs`: `PriorityQueue` trait for abstracting over heap implementations.
  - `binary.rs`: Binary heap wrapper (`dijkstra_binary`).
  - `fib.rs`: Fibonacci heap wrapper (`dijkstra_fibonacci`).
  - `fib_unsafe.rs`: Unsafe Fibonacci heap wrapper (`dijkstra_fibonacci_unsafe`).
  - `pairing.rs`: Pairing heap wrapper (`dijkstra_pairing`).
  - `radix.rs`: Radix heap wrapper (`dijkstra_radix`).
  - `dial.rs`: Dial's algorithm wrapper (`dijkstra_dial`).
  - Also contains graph parsing, validation, and tests.
- `src/fibonacci/`: Fibonacci heap implementations.
  - `heap.rs`: `Rc<RefCell>`-based heap (`Node`, `FibonacciHeap`).
  - `heap_unsafe.rs`: unsafe heap (`UnsafeNode`, `UnsafeFibonacciHeap`).
- `src/pairing/`: Pairing heap implementation.
  - `heap.rs`: pairing heap (`Node`, `PairingHeap`).
- `src/radix/`: Radix heap implementation.
  - `heap.rs`: radix heap (`RadixHeap`, `RadixHandle`).
- `src/dial/`: Dial's algorithm implementation.
  - `heap.rs`: bucket-based heap (`DialHeap`, `DialHandle`).

**Architecture:**

The project uses a trait-based design where all heap types implement the `PriorityQueue` trait, allowing a single generic Dijkstra implementation to work with any heap type.

## Correctness, bugs, and security

This project aims to be **correct and well-tested**, but it is still an experimental “lab”
for comparing heap implementations.

- **Correctness**: All heaps are exercised by unit tests and cross-checked in benchmarks so
  that they produce the same shortest-path **distance** on the same graphs. The unsafe
  Fibonacci heap is kept in sync with the safe variant via these tests.
- **Bugs**: If you find a bug (wrong path, wrong distance, panic, etc.), please open an
  issue with:
  - the exact input file (or a minimal reproducer),
  - which heap you were using (`--heap` value),
  - the observed output and the expected one.
  This makes it much easier to reproduce and fix the problem.
- **Security**: The code is intended for offline graph experiments, not for processing
  untrusted input in production services. There is an `unsafe` Fibonacci heap implementation;
  it is tested and benchmarked, but should be treated with the usual care that any `unsafe`
  code deserves.

If you believe you have found a **security-relevant** issue, please avoid posting full
exploits publicly; instead, report the details privately (for example via the repository’s
security contact or a private issue if available).
