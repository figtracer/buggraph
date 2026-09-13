# Retrieval

The library compiles JSON into an immutable graph with integer node references,
facet and lexical posting lists, and cached JSONL summaries. Load once and reuse
`Graph` in a long-lived consumer. The CLI reloads and compiles on each invocation.

## Modes

`id_order` returns failure modes in stable ID order, the original flat baseline.
`bm25` ranks positive lexical matches. `bm25_ancestors` interleaves each match with
its unique broader failure modes, nearest ancestors first. Ancestors retain the
originating score; they are structural context, not independent semantic matches.

BM25 indexes summary, definition, applicability, and facets. Exclusions and source
titles are not positive relevance evidence. Terms are Unicode alphanumeric runs,
lowercased; query terms are deduplicated. There is no stemming, learned synonym
expansion, embedding service, relevance threshold, or semantic negation handling.
Ties resolve by stable ID. Shared ancestors appear once.

Fixed parameters `k1=1.2`, `b=0.75`, and positive log IDF follow conventional
[Lucene BM25 defaults](https://lucene.apache.org/core/9_12_1/core/org/apache/lucene/search/similarities/BM25Similarity.html).
They were not fitted to the suite. Unfiltered lexical queries score only matching
posting lists. Each posting's BM25 contribution is computed once when the immutable
index is built; queries add those contributions in sorted term order. Rebuilding the
graph recomputes weights for the new corpus. Sorting costs depend on matching document
count. Plain ranked lookup needs no ancestor-deduplication set.

## Token budgets

`TokenCounter::for_model` uses the locked `tiktoken-rs` model-to-encoding mapping.
Unknown models fail. Tests exercise `gpt-4` (`cl100k_base`) and `gpt-4o` (`o200k_base`).
Model-name support is exactly that of the locked dependency, not all providers.

Content is ordinary text, including strings resembling special token delimiters.
Packing considers whole JSONL records in ranking order, skips records that do not
fit, and recounts the complete candidate text. Independently counted BPE lengths
are not assumed additive. Counts cover stdout, including source URLs and JSON syntax.

The caller must separately reserve messages, instructions, tools, API envelopes, and
completion tokens. See official [token counting guidance](https://developers.openai.com/cookbook/examples/how_to_count_tokens_with_tiktoken).
The byte-budgeted `context` command retains deterministic ID ordering.

## Compact bundles

`bundle CORPUS MODE MODEL MAX_TOKENS DETAIL QUERY [facets ...]` uses the same
ranking as `search`. `DETAIL` is `summary` or `full`. It returns one compact JSON
object containing `revision`, `records`, a `sources` array of citation URLs, and
`omitted` (ranked candidates that did not fit). Each record’s integer `sources`
values are zero-based indexes into that response’s citation table, not stable IDs.
Record IDs remain stable. The complete source registry is available through `show`.

The corpus revision and identical citation URLs appear once. Empty optional fields
are omitted. Full records add complete definitions, applicability, exclusions, and
external mappings, and attached code excerpts. Code text is kept verbatim; excerpts
carry their language, one-based source start line, and citation-table source index.
Relationships appear under `edges` only when both endpoints are
selected; omission does not establish that no other relationships exist. Graph
expansion remains opt-in through `bm25_ancestors`. A DAG and semantic similarity
search are separate concepts; this engine currently uses lexical ranking.

Packing recounts the entire serialized object, including citations, edges, omission
count, JSON syntax, and trailing newline. Definitions are never truncated. Records
that do not fit are skipped. If no record matches or fits, stdout is empty. Successful
bundle calls produce no stderr diagnostics, so ranking metadata cannot silently add
tokens to the returned context. The library also exposes exact token counts. A single
record can cost more than legacy JSONL because of the envelope; savings depend on
record count, repetition, and the selected detail level.

Use a full bundle when the question needs descriptions immediately; a summary bundle
followed by `show` is useful when only a few records will need expansion. Fetching the
entire corpus or expanding all ancestors is not inherently token-efficient.

Append `--compact` to opt into lossless structural compression. It compares ordinary
JSON with factored and tabular JSON, counts their complete decoding guides, and
keeps the smallest candidate. See [encoding and code preservation](packing.md).
The existing `Graph::bundle` keeps ordinary JSON; `bundle_with_options` accepts an
explicit `BundleFormat`.

## Performance boundaries

Packing is greedy, not an optimal relevance-per-token solver. Recounting growing
text trades CPU for exact budgeting and can be expensive for many candidates.
Tokenizer initialization is cached per process. No disk index, memory mapping,
embedding database, or parallel retrieval is required. Measure representative
workloads before adding these. See [benchmark instructions](../BENCHMARKS.md).
