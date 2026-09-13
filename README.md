# buggraph

[![CI](https://github.com/figtracer/buggraph/actions/workflows/ci.yml/badge.svg)](https://github.com/figtracer/buggraph/actions/workflows/ci.yml)

smart contract security references. local retrieval, exact token budgets.

[getting started](#getting-started) · [references](data/owasp.json) · [encoding](docs/packing.md) · [evaluation](docs/evaluation.md) · [contributing](CONTRIBUTING.md)

A Rust library and CLI for fetching reference material in one call. Search **156
pinned OWASP descriptions**, return summaries or full records with citations, and
compress repeated structure without changing descriptions or code.

The reference catalog has stable IDs and category filters. A separate
[starter taxonomy](data/curated.json) adds typed relationships across 19 failure modes
and four properties. Code excerpts retain their source and line location; the catalog
currently includes one pinned defensive example.

## Why use it

| Feature | What it provides |
| --- | --- |
| Local search | BM25 ranking and category filters without model calls or network access. |
| Token budgets | Whole records packed under an exact model-token cap. |
| Lossless encoding | Shared metadata and schema; exact descriptions, code, and citation reconstruction. |
| Source provenance | Pinned URLs, licenses, review status, and code line locations. |
| Optional graph | Typed relationships and deduplicated ancestor expansion. |

On the complete 156-record payload (`gpt-4o` encoding):

| Payload | Plain bundle | Compact bundle |
| --- | ---: | ---: |
| Summaries | 14,286 tokens | 4,887 tokens (−66%) |
| Full descriptions and code | 28,915 tokens | 20,752 tokens (−28%) |

Decoded records match exactly. Three separate agents each passed 36/36 extraction
questions across synthetic fixtures and pinned references, including exact code and
similar identifiers. This is a small single-model check. Compact encoding trades additional CPU work for fewer tokens.

## Getting started

Install [Rust](https://rustup.rs), then build with the pinned toolchain:

```sh
git clone https://github.com/figtracer/buggraph.git
cd buggraph
cargo install --path . --locked

buggraph validate data/owasp.json
buggraph bundle data/owasp.json bm25 gpt-4o 2048 full "contract architecture" --compact
buggraph explore data/owasp.json gpt-4o 2048 summary 8 2 "contract architecture"
buggraph serve data/owasp.json gpt-4o
buggraph show data/owasp.json scwe:143
```

Use `summary` instead of `full` for an overview. `--compact` compares reversible
representations and includes its decoding guide in the budget. Omit the flag for
ordinary JSON. Source numbers index the shared citation table.

| Command | What it does |
| --- | --- |
| `validate` | Check IDs, provenance, edge types, and specialization cycles. |
| `bundle` | Fetch summaries or complete records within a token budget. |
| `explore` | Pack direct matches, then bounded graph context. |
| `serve` | Reuse one compiled graph and tokenizer over JSON lines. |
| `expand` | Decode a saved compact bundle into ordinary JSON. |
| `search` | Return ranked summaries as JSONL. |
| `show` | Open a record with its code, sources, and relationships. |
| `descendants` | Explore a specialization subtree. |
| `context` | Fetch facet-filtered summaries within a byte budget. |
| `coverage` | Read evidence-backed assessment states for an explicit scope. |
| `eval` | Compare retrieval modes against labeled queries. |

Bundle writes only its payload to stdout. Token counts cover the complete response;
callers reserve their own message and tool overhead. The tokenizer runs locally and
needs no API key. The persistent protocol adds direct-first, depth-bounded exploration
without letting ancestors displace lexical matches. See [retrieval](docs/retrieval.md)
and [encoding](docs/packing.md).

## Evaluation

```sh
buggraph eval data/curated.json data/eval.json gpt-4o 2048 3
```

The authored diagnostic suite reports precision, recall, MRR, nDCG, negative-query
behavior, and context tokens. See [metric definitions](docs/evaluation.md) and
[benchmark instructions](BENCHMARKS.md).

## Development

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

CI runs on Linux and macOS. Code is MIT; data is CC-BY-SA-4.0. See
[agent instructions](AGENTS.md), [contributing](CONTRIBUTING.md), and
[source attribution](data/ATTRIBUTION.md).
