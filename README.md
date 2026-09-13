# buggraph

[![CI](https://github.com/figtracer/buggraph/actions/workflows/ci.yml/badge.svg)](https://github.com/figtracer/buggraph/actions/workflows/ci.yml)

smart contract security references. local retrieval, exact token budgets.

[getting started](#getting-started) · [references](data/owasp.json) · [encoding](docs/packing.md) · [evaluation](docs/evaluation.md) · [contributing](CONTRIBUTING.md)

A Rust library and CLI for fetching reference material in one call. Search **156
pinned OWASP documents**, return summaries or complete Markdown with citations, and
compress repeated structure without changing descriptions or code. Source-labeled
Bastet CSV releases can be imported locally without redistributing their data.

The reference catalog has stable IDs and category filters. A separate
[starter taxonomy](data/curated.json) adds typed relationships across 19 failure modes
and four properties. The OWASP snapshot retains remediation and fenced examples in
place; 155 of its documents contain code.

## Why use it

| Feature | What it provides |
| --- | --- |
| Semantic inventory | Every ID, source-derived synopsis, facet, and edge in one model-counted response. |
| Layered taxonomy | Route over classes first, then fetch ranked concrete instances from selected facets. |
| Local search | BM25 ranking and category filters without model calls or network access. |
| Token budgets | Whole records packed under an exact model-token cap. |
| Lossless encoding | Shared metadata and schema; exact descriptions, code, and citation reconstruction. |
| Source provenance | Pinned URLs, revisions, licenses, and review status. |
| Optional graph | Typed relationships and deduplicated ancestor expansion. |

On the source-complete 156-record OWASP snapshot (`gpt-4o` encoding):

| Model input | Tokens |
| --- | ---: |
| UltraFuzz planner catalog | 143,407 |
| Complete Buggraph routing inventory | 7,156 (−95%) |
| Inventory plus eight retrieved complete documents (mean) | 13,060 (−91%) |

All 156 inventory IDs resolve to byte-identical pinned Markdown. In a blinded 12-case
routing replay, the inventory agreed with the full catalog on 10 top-ranked classes;
their top-three sets overlapped by 72% on average. This measures routing preservation,
not vulnerability detection.

On a pinned external [Bastet CSV snapshot](https://drive.google.com/file/d/19YBeCmPwx3aLZ9PZVGjjRDSYifBYpbLe/view)
(`27d82e…d4a1d`), import produced 104 classes and 572 finding
instances. The complete class taxonomy is 6,980 `gpt-4o` tokens versus 56,481 for all
676 records and 846 edges (−88%); findings remain available through `instances`.

## Getting started

Install [Rust](https://rustup.rs), then build with the pinned toolchain:

```sh
git clone https://github.com/figtracer/buggraph.git
cd buggraph
cargo install --path . --locked

buggraph validate data/owasp.json
buggraph inventory data/owasp.json gpt-4o
buggraph taxonomy data/owasp.json gpt-4o
buggraph bundle data/owasp.json bm25 gpt-4o 2048 full "contract architecture" --compact
buggraph instances corpus.json bm25 gpt-4o 4096 full "dust liquidation" tag:dos --compact
buggraph explore data/owasp.json gpt-4o 2048 summary 8 2 "contract architecture"
buggraph serve data/owasp.json gpt-4o
buggraph show data/owasp.json scwe:143
# Rebuild data/owasp.json from a pinned OWASP checkout:
buggraph import-owasp ../owasp-scs fefd476b83074666ada2d816f103436a18e1ece4 data/owasp.json
# Import a separately downloaded Bastet release, pinned by its SHA-256:
buggraph import-bastet dataset.csv SHA256 https://example.com/dataset.csv corpus.json
```

Use `summary` instead of `full` for an overview. `--compact` compares reversible
representations and includes its decoding guide in the budget. Omit the flag for
ordinary JSON. Source numbers index the shared citation table.

| Command | What it does |
| --- | --- |
| `validate` | Check IDs, provenance, edge types, and specialization cycles. |
| `inventory` | Return every semantic synopsis and relationship in one compact response. |
| `taxonomy` | Return classes and their relationships without concrete findings. |
| `bundle` | Fetch summaries or complete records within a token budget. |
| `instances` | Fetch concrete findings by query and taxonomy facets. |
| `explore` | Pack direct matches, then bounded graph context. |
| `serve` | Reuse one compiled graph and tokenizer over JSON lines. |
| `import-owasp` | Reproduce a source-complete corpus from a pinned checkout. |
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

CI runs on Linux and macOS. Code is MIT; bundled data is CC-BY-SA-4.0. Bastet
inputs remain external under [CC-BY-NC-4.0](https://arxiv.org/html/2606.03387v1#S5). See
[agent instructions](AGENTS.md), [contributing](CONTRIBUTING.md), and
[source attribution](data/ATTRIBUTION.md).
