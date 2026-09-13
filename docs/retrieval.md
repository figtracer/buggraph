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
posting lists. Sorting costs depend on matching document count.

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

## Performance boundaries

Packing is greedy, not an optimal relevance-per-token solver. Recounting growing
text trades CPU for exact budgeting and can be expensive for many candidates.
Tokenizer initialization is cached per process. No disk index, memory mapping,
embedding database, or parallel retrieval is required. Measure representative
workloads before adding these. See [benchmark instructions](../BENCHMARKS.md).
