# External-text test: frozen inputs

These inputs were committed before scoring. No ranker tuning is part of this study.

Ten queries are excerpts from SWC Registry descriptions at commit
`1b6227074ecd180b374e4844153afdda0332f979`. The first eight address classes already
represented in the starter corpus; two compiler-related classes are absent and serve
as out-of-catalog negative controls. Excerpts omit code, examples, and reproductions.
Extraction lengths are fixed in `provenance.json`. Labels map SWC classes to existing
Buggraph IDs and were proposed by Codex; no independent adjudication is claimed.

Two corpora contain the same 16 mapped failure IDs and the same surviving graph edges:

- `source-corpus.json`: the title and Description section from the pinned OWASP files,
  with no added facets, applicability notes, or exclusions.
- `normalized-corpus.json`: the corresponding Buggraph records, preserving their
  existing summaries, definitions, facets, applicability, and exclusions.

Both use the same Rust BM25 implementation, tokenizer, source attribution, and
serialization. The source corpus is an **OWASP-text lexical proxy**, not the actual
UltraFuzz planner or retrieval policy. Its source files come from UltraFuzz's pinned
public test fixture, whose provenance is recorded alongside the queries. The
source-selection boundary is matched; text lengths and editorial annotations differ
by design. This is a presentation comparison, not a pure graph comparison.

The graph comparison is within each corpus: BM25 versus the same text with ancestor
expansion. Only specific classes are labeled; ancestors receive no extra credit.

Use `source-suite.json` and `normalized-suite.json` with their respective corpora.
Run `gpt-4o` with returned-context caps 128, 256, and 512, and k=1 and k=3. These
settings examine single-record capacity and a small multi-record context window.
Retain every configuration and per-case result; do not select only favorable rows.

Neither source corpus, query source, nor class crosswalk is an independent real-world
audit holdout. SWC and OWASP have common classification ancestry. The tiny selected
sample cannot support population-level detection claims. Describe it as externally
authored text, not an independently validated benchmark.

SWC excerpts retain the attribution and MIT terms in `LICENSE-SWC`. OWASP adaptations
retain the parent directory's CC-BY-SA-4.0 attribution. See `provenance.json` for URLs,
revisions, extraction rules, and hashes. New experiment metadata uses CC-BY-SA-4.0.
