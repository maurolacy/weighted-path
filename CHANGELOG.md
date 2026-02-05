# Changelog

All notable changes to this project will be documented in this file.

The format is loosely inspired by [Keep a Changelog](https://keepachangelog.com/),
and the project follows Semantic Versioning.

## [0.6.0] - 2026-01-22

### Changed

- Add comprehensive "Library Usage" section with examples for:
  - Basic usage with high-level helpers.
  - Different heap implementations.
  - Parsing graphs and using the low-level API.
  - Directed vs undirected graphs.
  - Generic Dijkstra function with custom heaps.
- Update `Cargo.toml`:
  - Add explicit `[lib]` section.
  - Add `keywords` and `categories` for better crates.io discoverability.

### Documentation

- Add doc tests for working / tested examples.
- Add example showing the full workflow: parsing graph strings with `parse_graph`, calling underlying Dijkstra functions, and converting path indices back to node names.
- Clarify separation between library API (returns `Result` types) and command-line tool concerns (file I/O, argument parsing).

## [0.5.1] - 2026-02-05

### Changed

- Doc / package adjustments.

## [0.5.0] - 2026-01-28

### Breaking

- Change the core Dijkstra API to return both **path and distance**:
  `dijkstra<Q: PriorityQueue>(...) -> (Vec<usize>, Option<u32>)` instead of just a path.
- Update all heap-specific wrappers (`dijkstra_binary`, `dijkstra_fibonacci`,
  `dijkstra_fibonacci_unsafe`, `dijkstra_pairing`, `dijkstra_radix`, `dijkstra_dial`) to
  return `(path, distance)` as well.

### Added

- Extend the high-level helpers (`find_shortest_path*`) to return
  `(String, Option<u32>)`, so callers get the human-readable path **and** total
  distance in one call.

### Changed

- `weighted_path` binary now uses the helpers and reports:
  `Node0-…-NodeN (weight: X)` in normal mode.
- Benchmarks and tests updated to:
  - use the new `(path, distance)` / `(String, Option<u32>)` signatures,
  - compare heaps by **distance** (paths may legitimately differ when multiple
    shortest paths exist).

## [0.4.2] - 2026-01-28

### Fixed

- Fix radix heap bucket indexing bug: use MSB of `key ^ min_key` (XOR) instead of
  `key - min_key` (subtraction) to maintain the radix heap invariant. This ensures
  correctness on large graphs where the subtraction-based approach could place nodes
  in incorrect buckets, leading to suboptimal shortest paths.

### Changed

- Standardise spelling to British English throughout documentation and code comments.

## [0.4.1] - 2026-01-26

### Fixed

- Add an explicit upper bound to Dial's bucket count and panic with a clear
  message when a distance would exceed that (`MAX_BUCKETS` guard), instead of
  risking huge allocations or silent misbehaviour.

### Changed

- Simplify `DialHeap` internals (position updates / bookkeeping) while keeping
  behaviour and benchmarks intact.

## [0.4.0] - 2026-01-26

### Added

- Implement **Dial's algorithm** as `DialHeap`.
- Integrate Dial-based Dijkstra into:
  - the core `dijkstra` module,
  - the `weighted_path` binary (`--heap dial`),
  - the Criterion benchmarks.

### Documentation

- Extend `README.md` with Dial heap details, complexity, and benchmark numbers.

## [0.3.1] - 2026-01-25

### Added

- `--profile` / `-p` flag in the `weighted_path` binary to run Dijkstra
  repeatedly on the same graph for profiling.

### Changed

- Minor code-style and documentation polish (iterator syntax, README bullets).

## [0.3.0] - 2026-01-24

### Added

- Implement **radix heap** and a Dijkstra variant using it.
- Integrate the radix-heap Dijkstra into:
  - the `weighted_path` binary (`--heap radix`),
  - the benchmarking suite.
- Add an MIT `LICENSE`.

### Performance

- Replace `HashMap` node-position tracking in the radix heap with a
  `Vec<Option<(bucket_idx, pos)>>` for dense node IDs.
- Pre-allocate node positions with `with_capacity` to avoid resizes.
- Use `swap_remove` inside buckets to avoid O(n) removals.

### Documentation

- Update `README.md` with radix heap description and benchmark results.

## [0.2.0] - 2026-01-22

### Added

- **Generic Dijkstra** over an abstract `PriorityQueue` trait, so all heap
  implementations can share a single Dijkstra core.

### Changed

- Implement `PriorityQueue` directly for the existing heaps instead of wrapping
  them in adapter structs.
- Remove the old `generic` module and rename `dijkstra_generic` to just
  `dijkstra`.
- Update `README.md` to reflect the new structure and heap abstraction.

### Maintenance

- `cargo update` and `cargo clippy` cleanups.

## [0.1.1] - 2026-01-22

### Changed

- Small documentation and README tweaks.

### Maintenance

- `cargo update` and `cargo clippy` to keep dependencies and linting fresh.

## [0.1.0] - 2026-01-22

Initial tagged release.

### Added

- Core **weighted shortest path** functionality using Dijkstra's algorithm.
- Graph parsing utilities and simple CLI (`weighted_path`) that:
  - reads graphs from a file,
  - prints shortest paths between nodes.
- Initial **BinaryHeap-based** Dijkstra implementation.
- **Fibonacci heap** implementations:
  - an unsafe/raw-pointer version,
  - a safe `Rc<RefCell<…>>` version.
- Dijkstra variants using both Fibonacci heaps.
- Reference benchmarks and random graph generators (sparse / dense) for
  comparing heap variants.
- A basic `README.md` describing usage and early benchmark results.

