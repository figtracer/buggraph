# buggraph

[![CI](https://github.com/figtracer/buggraph/actions/workflows/ci.yml/badge.svg)](https://github.com/figtracer/buggraph/actions/workflows/ci.yml)

smart contract failure modes, structured for agents. local retrieval first.

[getting started](#getting-started) · [references](data/owasp.json) · [retrieval](docs/retrieval.md) · [evaluation](docs/evaluation.md) · [agent instructions](AGENTS.md) · [contributing](CONTRIBUTING.md)

Buggraph is a local reference library with token-budgeted retrieval and optional typed
graph relationships. A Rust library and CLI share the same validated index.

The reference corpus includes titles and complete Description sections for all **156
SCWE entries** in a pinned OWASP snapshot, with stable IDs, category filters, and
citations. Imported records are distinguished from checked adaptations. The separate
[starter taxonomy](data/curated.json) contains 19 failure modes and four properties;
its source checking was AI-assisted and still needs independent expert review.

## Why use it

Buggraph makes security knowledge easier to retrieve, cite, and budget. It does not
currently establish that an agent finds more vulnerabilities.

| Benefit | What is available now |
| --- | --- |
| Smaller context | Fetch summaries or complete descriptions in one token-capped bundle; shared citations appear once. |
| Reusable knowledge | Stable failure-mode IDs, source references, applicability notes, and exclusions. |
| Explainable relationships | Inspect broader classes and shared properties without duplicating records. |
| Auditable assessment | Record evidence and unresolved questions separately from nodes visited. |

Local diagnostics found no graph-specific recall or lookup-speed advantage over flat
records. UltraFuzz already
[caches OWASP references](https://github.com/monad-developers/ultrafuzz/blob/89b57c9a7c5aa22af15e1ae9625b8ab3a2c6f810/packages/references/src/index.ts#L328-L359);
its end-to-end benefit from Buggraph remains unproven.

Across 19 local queries on the 156-entry catalog, summary bundles used **11–12%
fewer tokens** than legacy JSONL with the same selected records and summary fields
(`gpt-4` and `gpt-4o` encodings). This measures serialization savings, not relevance
or end-to-end agent performance.

## Getting started

Install [Rust](https://rustup.rs), then build with the pinned toolchain:

```sh
git clone https://github.com/figtracer/buggraph.git
cd buggraph
cargo install --path . --locked

buggraph validate data/owasp.json
buggraph bundle data/owasp.json bm25 gpt-4o 2048 full "contract architecture"
buggraph show data/owasp.json scwe:001
```

The model name selects a local tokenizer. Search makes no model calls and needs no
API key, database server, or network connection after dependencies are installed.

| Command | What it does |
| --- | --- |
| `validate` | Check IDs, source references, edge types, and specialization cycles. |
| `bundle` | Fetch summaries or full descriptions in one compact, token-capped JSON object. |
| `search` | Rank failure modes and return summaries within a token budget. |
| `show` | Expand a record with applicability, exclusions, sources, and relationships. |
| `descendants` | Explore a specialization subtree with shared nodes deduplicated. |
| `context` | Retrieve facet-filtered summaries within a byte budget. |
| `coverage` | Count evidence-backed assessment states against an explicit scope. |
| `eval` | Compare ID ordering, BM25, and BM25 with ancestor expansion. |

Bundle writes one JSON object and no success diagnostics; use `summary` in place of
`full` for a smaller overview. Source numbers index the shared citation table.
Search retains its JSONL interface and writes ranking metadata to stderr. Budgets
cover stdout; callers reserve their own message and tool overhead.
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

CI runs checks on Linux and macOS. Code is MIT; the data is CC-BY-SA-4.0.
See [contributing](CONTRIBUTING.md) and [source attribution](data/ATTRIBUTION.md).
