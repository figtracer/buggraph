# Contributing

Small, reviewable changes are welcome. Describe the problem and observable behavior
in a short PR body. The initial implementation and starter data were created with
OpenAI Codex assistance; disclose material AI assistance in later contributions too.

## Code

Use the pinned Rust toolchain and committed lockfile. Keep the library usable without
the CLI. Add tests for behavior changes, preserve stable external IDs, and benchmark
the affected path before making performance claims. Do not force-push published work.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo run --locked -- validate data/curated.json
cargo run --locked -- eval data/curated.json data/eval.json gpt-4o 2048 3
```

## Data

Read [attribution](data/ATTRIBUTION.md) and [the schema](docs/schema.md). A useful
failure mode has a narrow definition, applicability, exclusions, a violated property,
and a pinned source. Several published findings may support the same stable concept.
Do not turn every report into a new failure mode. Source checking is not expert review.

Preserve source license terms and clearly mark adaptations. Do not submit confidential
reports, copied exploit payloads, or operational attack instructions. Findings should
be concise retrospective classification records with public provenance.

Evaluation labels must be independent of ranking output. Keep duplicate reports and
protocol families within one split for any future audit-derived suite. The current
suite is authored diagnostics; new independent assessments belong in a separate suite.

Code and documentation contributions use MIT. Data under `data/` uses CC-BY-SA-4.0,
including adaptations of OWASP material. Linked sources retain their licenses.
