# Benchmarking

```sh
cargo run --locked --release --example bench -- 10000 1000
```

The synthetic benchmark compares flat filtering with indexed facet intersection.
Both implementations materialize the same matching IDs. It reports index compilation
and mean warm query time; it excludes parsing, process startup, token packing, and
network or model calls. It has no graph edges and does not measure DAG traversal.

Use matched inputs, repeated runs, and representative corpus sizes. Separate cold
loading from repeated queries, and include tokenizer initialization when measuring
CLI latency. Keep raw runs outside the repository. This benchmark does not establish
UltraFuzz performance or vulnerability-detection quality.
