# Buggraph

Standalone Rust prototype for organizing and retrieving reviewed security knowledge.
It is a library and a local CLI. Example records are illustrative classifications,
not verified findings or a populated vulnerability database.

## Run

```sh
cargo test --offline
cargo run --offline -- validate data/example.json
cargo run --offline -- context data/example.json 4096 operation:liquidation
cargo run --offline -- show data/example.json fm:liquidation-liveness
cargo run --offline -- descendants data/example.json fm:liveness
cargo run --offline -- coverage data/example.json data/ledger.json
cargo run --offline --release --example bench -- 10000 1000
```

`context` writes complete JSONL records to stdout and selection counts to stderr.
The budget counts actual UTF-8 output bytes, including escaping and newlines.
It is **not a token budget**. Model-specific token counting belongs in an adapter.
Other commands emit JSON. Invalid data or arguments exit unsuccessfully.

## Representation

JSON is the editable source format. Compilation sorts nodes by stable external ID,
resolves edges to integer indices for traversal, validates endpoint types, rejects
duplicate IDs/edges and dangling references, and checks specialization acyclicity
using iterative topological processing. Other edge types can contain cycles.

The immutable index stores explicit facet posting lists and pre-encoded summaries.
AND queries start with the smallest posting list and check the remaining lists.
Facet absence is not evidence of non-applicability. Facets are not inherited.
Stable ID ordering makes output deterministic; it is not a relevance model.
Packing skips records that do not fit and does not promise an optimal subset.
`show` expands a record with its definition, exclusions, sources, and incident edges.

The library can compile once and reuse the graph across queries. Each CLI invocation
loads JSON and rebuilds its index; cold-start measurements must include that cost.
No database server, model calls, network access, unsafe code, or async runtime is
required. The benchmark measures warm filtering only, with identical output checks,
and reports compilation separately. It does not establish end-to-end agent quality.

## Coverage semantics

A ledger binds a scope to a corpus revision and a reviewed design/scope revision.
Every scoped ID must be a unique failure mode. Missing records remain unassessed.
Every explicit assessment requires an evidence reference or explanation. Counts
preserve unresolved applicability and exclusions separately; there is no fabricated
"percent secure" score. Evidence references are recorded, not independently verified.
Revision strings are caller supplied, not cryptographic integrity proofs.

## Next work, guided by measurements

1. Curate a small, reviewed corpus with provenance, licenses, applicability conditions,
   stable identity rules, and independent findings. Keep examples out of evaluation.
2. Evaluate retrieval against flat catalogs on held-out, deduplicated published data;
   measure recall, attribution, unsupported matches, and actual model token cost.
3. Add a tokenizer adapter and explicit progressive-detail requests when integrating
   a consumer. Do not couple the taxonomy to automated protocol scanning.
4. Measure load time and peak memory before choosing a serialized binary index,
   compressed adjacency, bitmap postings, or memory mapping. Current storage includes
   both authoring records and cached summaries, trading memory for query work.

This is a working foundation, not evidence of the fastest possible implementation.
The hard research problem remains consistent concepts and useful, evaluated retrieval.
