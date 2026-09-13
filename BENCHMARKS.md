# Initial measurement

One local run on 2026-09-13 using rustc 1.98.0, Cargo release profile with thin LTO.

```sh
cargo run --offline --release --example bench -- 10000 1000
```

| Workload | Result |
| --- | ---: |
| Synthetic records | 10,000 |
| Queries per implementation | 1,000 |
| Matching records per query | 100 |
| Index compilation including lexical indexing and summary encoding | 24.652 ms |
| Flat scan, mean per query | 22.603 µs |
| Indexed intersection, mean per query | 1.004 µs |

Both implementations materialize the same matching integer IDs. This single warm
filtering comparison excludes parsing, process startup, context packing, and model
execution. It is not a statistical performance guarantee or an agent recall result.
The synthetic corpus has two facets per node and no edges, so it does not measure
DAG traversal. Memory usage has not been measured. Reproduce on representative
corpora before making architectural or deployment performance claims.
