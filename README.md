# buggraph

[![CI](https://github.com/figtracer/buggraph/actions/workflows/ci.yml/badge.svg)](https://github.com/figtracer/buggraph/actions/workflows/ci.yml)

smart contract failure modes, structured for agents. local retrieval first.

[getting started](#getting-started) · [taxonomy](data/curated.json) · [retrieval](docs/retrieval.md) · [evaluation](docs/evaluation.md) · [agent instructions](AGENTS.md) · [contributing](CONTRIBUTING.md)

Buggraph organizes failure modes into a typed graph, retrieves relevant context within
a model-token budget, and records assessment coverage separately. A Rust library and
CLI share the same validated index.

The starter corpus has 19 failure modes, four properties, and 16 pinned OWASP sources.
The evaluation includes authored diagnostic queries and publishes every result.
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

In [32 follow-up evaluations](docs/experiments.md), reducing the cap from 512 to 128
tokens preserved specific-class recall on the small authored test set while reducing
mean returned context by 68%. Graph expansion added broader context but **no additional
specific-class recall**. Both lexical modes still returned matches for exclusion
near misses. These are retrieval results, not proven improvements in an agent's
reasoning, total token spend, or vulnerability detection.

Ultrafuzz is a potential consumer. The [comparison plan](docs/experiments.md#ultrafuzz-comparison)
keeps knowledge retrieval separate from campaign execution; no Ultrafuzz integration
or end-to-end result is claimed.

The [UltraFuzz evidence review](docs/ultrafuzz-evidence.md) finds that its current
catalog already supports much of this metadata. A frozen external-text comparison
shows a modest normalization benefit over an OWASP-text proxy, but no demonstrated
graph advantage. Incremental usefulness to UltraFuzz remains **unproven**.

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
| `search` | Rank failure modes and return complete records within a token budget. |
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
See [results and limitations](docs/evaluation.md) and [performance measurements](BENCHMARKS.md).
The [follow-up experiments](docs/experiments.md) test budget sensitivity and remove
redundant ancestor labels to distinguish broader context from specific-class recall.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
python3 -m unittest discover -s scripts -p 'test_*.py'
```

CI runs checks on Linux and macOS. Code is MIT; the starter data is CC-BY-SA-4.0.
See [contributing](CONTRIBUTING.md) and [source attribution](data/ATTRIBUTION.md).
