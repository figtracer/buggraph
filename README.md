# buggraph

[![CI](https://github.com/figtracer/buggraph/actions/workflows/ci.yml/badge.svg)](https://github.com/figtracer/buggraph/actions/workflows/ci.yml)

Fast, token-budgeted retrieval for smart contract security knowledge.

Buggraph gives agents a compact map of vulnerability classes, then fetches detailed
descriptions, code examples, and concrete findings only when selected. The Rust CLI
runs locally with stable IDs, exact model-token budgets, BM25 search, and typed
taxonomy relationships.

## Why use it

- Cover the full vulnerability inventory without loading every document.
- Route from categories to failure modes and concrete audit findings in one call.
- Fetch selected records by ID with deterministic ordering and citations.
- Keep retrieval local, fast, reproducible, and bounded to the agent's token budget.
- Measure taxonomy coverage from the nodes and relationships explored.

## Results

On the 156-record OWASP snapshot using the `gpt-4o` tokenizer:

| Input | Tokens |
| --- | ---: |
| UltraFuzz planner catalog | 143,407 |
| Buggraph routing inventory | 7,868 (−95%) |
| UltraFuzz routing inventory with all 11 capabilities | 9,290 (−94%) |
| Threat-model capability registry | 272 (>99% smaller) |
| Routing inventory plus eight complete documents | 14,046 (−90%) |

The inventory retains all 156 class IDs. In a blinded 12-case replay, it selected
the same top-ranked class as the full catalog in 10 cases, with 64% mean overlap
between their top-three results.

Importing the source-labeled [Bastet dataset](https://drive.google.com/file/d/19YBeCmPwx3aLZ9PZVGjjRDSYifBYpbLe/view)
produced 104 classes, 572 findings, and 846 edges. Its complete class taxonomy uses
7,916 tokens, compared with 56,689 tokens for all 676 records (−86%).

## Use

```sh
git clone https://github.com/figtracer/buggraph.git
cd buggraph
cargo install --path . --locked

buggraph inventory data/owasp.json gpt-4o
buggraph bundle data/owasp.json bm25 gpt-4o 2048 full "liquidation denial of service" --compact
buggraph resolve data/owasp.json gpt-4o 8192 full scwe:037 scwe:141 --compact
buggraph instances corpus.json bm25 gpt-4o 4096 full "dust liquidation" tag:dos --compact
```

See [retrieval](docs/retrieval.md), [encoding](docs/packing.md),
[evaluation](docs/evaluation.md), and [contributing](CONTRIBUTING.md).

## Development

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

Code is MIT. Bundled data is CC-BY-SA-4.0. External Bastet inputs use
[CC-BY-NC-4.0](https://arxiv.org/html/2606.03387v1#S5).
