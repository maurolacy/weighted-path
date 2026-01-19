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

A test script is provided to run all test cases:

```bash
./tests.sh
```

This will run the program on all input files in the `testdata/` directory and compare the results with the expected outputs.

## Algorithm

The program uses Dijkstra's algorithm to find the shortest path:

1. Parse the graph and build an adjacency matrix
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
