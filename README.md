# buggraph

[![CI](https://github.com/figtracer/buggraph/actions/workflows/ci.yml/badge.svg)](https://github.com/figtracer/buggraph/actions/workflows/ci.yml)

smart contract failure modes, structured for agents. local retrieval first.

[getting started](#getting-started) · [taxonomy](data/curated.json) · [retrieval](docs/retrieval.md) · [evaluation](docs/evaluation.md) · [agent instructions](AGENTS.md) · [contributing](CONTRIBUTING.md)

Buggraph organizes failure modes into a typed graph, retrieves relevant context within
a model-token budget, and records assessment coverage separately. A Rust library and
CLI share the same validated index.

The starter corpus has 19 failure modes, four properties, and 16 pinned OWASP sources.
The evaluation includes authored diagnostic queries.
Source checking was AI-assisted; independent expert review is still needed.

## Why use it

Buggraph makes security knowledge easier to retrieve, cite, and budget. It does not
currently establish that an agent finds more vulnerabilities.

| Benefit | What is available now |
| --- | --- |
| Smaller context | Retrieve relevant summaries within an exact token cap, then expand selected records. |
| Reusable knowledge | Stable failure-mode IDs, source references, applicability notes, and exclusions. |
| Explainable relationships | Inspect broader classes and shared properties without duplicating records. |
| Auditable assessment | Record evidence and unresolved questions separately from nodes visited. |

Local diagnostics found no graph-specific recall or lookup-speed advantage over flat
records. UltraFuzz already
[caches OWASP references](https://github.com/monad-developers/ultrafuzz/blob/89b57c9a7c5aa22af15e1ae9625b8ab3a2c6f810/packages/references/src/index.ts#L328-L359);
its end-to-end benefit from Buggraph remains unproven.

On 156 OWASP records, warm BM25 ranking is **1.69× faster** than `16afb0d` in four
alternating local release-run pairs. Scores and output are unchanged; index construction
is about 3% slower and token-budgeted retrieval is roughly unchanged. This excludes
process and tokenizer startup.

## Getting started

Install [Rust](https://rustup.rs), then build with the pinned toolchain:

```sh
git clone https://github.com/figtracer/buggraph.git
cd buggraph
cargo install --path . --locked

buggraph validate data/curated.json
buggraph search data/curated.json bm25 gpt-4o 1024 "stale oracle response"
buggraph show data/curated.json fm:oracle-response-validity
```

The model name selects a local tokenizer. Search makes no model calls and needs no
API key, database server, or network connection after dependencies are installed.

| Command | What it does |
| --- | --- |
| `validate` | Check IDs, source references, edge types, and specialization cycles. |
| `search` | Rank failure modes and return summaries within a token budget. |
| `show` | Expand a record with applicability, exclusions, sources, and relationships. |
| `descendants` | Explore a specialization subtree with shared nodes deduplicated. |
| `context` | Retrieve facet-filtered summaries within a byte budget. |
| `coverage` | Count evidence-backed assessment states against an explicit scope. |
| `eval` | Compare ID ordering, BM25, and BM25 with ancestor expansion. |

Search writes JSONL to stdout and token counts and ranking metadata to stderr.
Budgets cover returned text; callers reserve their own message and tool overhead.
See [retrieval](docs/retrieval.md) for model support and [the schema](docs/schema.md)
for graph and coverage semantics.

## Evaluation

```sh
buggraph eval data/curated.json data/eval.json gpt-4o 2048 3
```

The suite reports precision, recall, MRR, nDCG, negative-query behavior, and context
tokens. These are small retrieval diagnostics, not vulnerability-detection results.
See [metric definitions](docs/evaluation.md) and [benchmark instructions](BENCHMARKS.md).

## Development

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

CI runs checks on Linux and macOS. Code is MIT; the starter data is CC-BY-SA-4.0.
See [contributing](CONTRIBUTING.md) and [source attribution](data/ATTRIBUTION.md).
