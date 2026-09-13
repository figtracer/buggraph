# Retrieval evaluation

The checked-in suite contains 24 AI-authored diagnostic queries: 19 positive queries
and five negative queries, including two near misses describing exclusions. It is
not an independent audit-report benchmark. See [attribution](../data/ATTRIBUTION.md).

```sh
cargo run --locked -- eval data/curated.json data/eval.json gpt-4o 2048 3
```

Choose the token budget and record cap explicitly. All modes use the same corpus,
judgments, token budget, and record cap. Redirect stdout to retain a local JSON report
with every retrieved ID and per-case metric. Keep experiment outputs outside the repo.

## Metrics and validation

Metrics use the records actually returned after token packing, capped at k. Precision
divides relevant returned records by k; recall divides them by the labeled relevant
set. MRR is the reciprocal rank of the first relevant result within k. nDCG uses
binary relevance with logarithmic discount and an ideal ranking capped at k.

Positive-query metrics are macro-averaged over nonempty relevance sets. Negative
queries are reported separately as the fraction returning no records. Mean context
tokens includes all queries in the split. Missing split populations yield null metrics.

The harness rejects stale corpus revisions, duplicate case IDs, duplicate normalized
queries, groups appearing in both splits, unknown relevance IDs, and labels that
contradict facet filters. Topic-group separation is explicit metadata, not automatic
semantic leakage detection.

## What this does not establish

The ID-order baseline is the original packer, not a competitive semantic retriever.
BM25 is the meaningful lexical baseline. The graph mode only expands ancestors;
it does not establish the usefulness of all graph relationships.

Before making detection claims, add independently labeled public finding summaries,
keep related reports and protocol families within one split, preserve temporal
holdouts, include adversarial near misses, and compare token budgets and representative
semantic baselines. Report per-family errors and uncertainty. Retrieval recall and
agent vulnerability-detection recall are different measurements.
