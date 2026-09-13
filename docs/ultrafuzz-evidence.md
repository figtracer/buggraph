# Is Buggraph a useful addition to UltraFuzz?

**Not established.** Current results justify a further offline study of normalized
knowledge records. They do not justify an UltraFuzz integration or a graph-specific
performance claim. A positive outcome is not guaranteed or required of the study.

## The baseline already has substantial structure

At UltraFuzz commit `89b57c9a7c5aa22af15e1ae9625b8ab3a2c6f810`, the public
[planner-catalog schema](https://github.com/monad-developers/ultrafuzz/blob/89b57c9a7c5aa22af15e1ae9625b8ab3a2c6f810/packages/artifacts/schema/vulnerability-database-planner-catalog.schema.json)
supports stable IDs, primary and secondary categories, required/optional/incompatible
capabilities, sources, review status, and source hashes. Its optional guidance object
supports preconditions, broken invariants, impact, and false-positive boundaries.
The [snapshot schema](https://github.com/monad-developers/ultrafuzz/blob/89b57c9a7c5aa22af15e1ae9625b8ab3a2c6f810/packages/artifacts/schema/vulnerability-database-snapshot.schema.json)
also defines content hashes and source identity for selected records.

| Capability | Evidence from inspected UltraFuzz schemas | Buggraph |
| --- | --- | --- |
| Stable IDs and categories | Already supported. | Supported. |
| Source provenance and review status | Already supported, including hash fields. | Source registry and revision labels; the CLI does not authenticate source contents. |
| Preconditions and boundaries | Supported by the guidance object. | Applicability and exclusions in expanded records. |
| Capability compatibility | Explicit required, optional, and incompatible sets. | Facets and prose applicability; no equivalent compatibility solver. |
| Specialization relationships | No specialization edges in these two schemas; not an exhaustive codebase finding. | Explicit DAG with multiple parents. |
| Exact returned-text token cap | Not established by this schema inspection. | Implemented and tested. |

Schema support does not prove every upstream record uses every field or that every
runtime path validates it. It does establish that these metadata ideas cannot be
presented as novel incremental benefits over UltraFuzz.

## A frozen external-text comparison

Inputs were committed as `69bb7d2203fbf0109b5f1231a0e70013e67a00c4` before scoring.
The engine remained `335ffd5`. Eight positive queries and two out-of-catalog controls
come from SWC Registry descriptions. The test removes our authored query wording,
but labels remain an AI-proposed crosswalk, and SWC/OWASP share taxonomy ancestry.

The matched corpora contain the same 16 classes and seven specialization edges.
The source-text proxy uses OWASP titles and descriptions extracted from the exact
files in UltraFuzz's public pinned test fixture. The normalized variant uses the
corresponding previously frozen Buggraph records. Both use the same Rust ranker,
tokenizer, source URLs, and output serializer. This **does not run or reproduce
UltraFuzz's actual selection policy**.

`gpt-4o` selects a local tokenizer; no model was called. There were 12 configurations,
each comparing three retrieval modes. Reusing ten queries across configurations does
not create additional independent observations.

| Returned cap | k | Source BM25 recall | Normalized BM25 recall | Source mean tokens | Normalized mean tokens |
| --- | --- | ---: | ---: | ---: | ---: |
| 128 | 1 | 4/8 | 6/8 | 92.2 | 113.5 |
| 256 | 1 | 4/8 | 6/8 | 92.2 | 113.5 |
| 512 | 1 | 4/8 | 6/8 | 92.2 | 113.5 |
| 128 | 3 | 4/8 | 6/8 | 92.2 | 113.5 |
| 256 | 3 | 6/8 | 7/8 | 184.5 | 227.4 |
| 512 | 3 | 7/8 | 7/8 | 277.2 | 340.8 |

Mean tokens includes all ten queries. The normalized records cost more per returned
record. They improve first-result hits in two cases with no losses; an exploratory
two-sided paired sign test gives p=0.5. That is weak evidence, not a superiority
finding. These selected cases are not a representative random sample.

At k=3, normalized context capped at 256 reaches the same positive hit set as source
context capped at 512, using 227.4 versus 277.2 mean tokens, about 18% less returned
text. This is a sample-specific retrieval tradeoff. It does not measure information
adequacy, reviewer time, total prompts, model cost, or an UltraFuzz benefit. It also
does not establish that a 256-token cap is sufficient for other data.

Both methods return nonempty results for both out-of-catalog controls. This is a
failure to abstain under these classification labels, **not a false vulnerability
finding**. The source's SWC-107 excerpt lacks an explicit reentry/unfinished-state
condition; its unchanged label is therefore uncertain, and its miss must not be
treated as an unambiguous retrieval failure.

Ancestor expansion provides no normalized-corpus specific-recall gain. In the source
proxy at cap=512/k=3 it reduces recall from 7/8 to 6/8: an ancestor displaces the
message-replay result. This is evidence against automatic ancestor interleaving for
this objective. It says nothing definitive about other uses of taxonomy relationships.

## Reproduce and inspect

The input rules and licenses are in [the frozen study](../data/external/README.md).
[Source results](../data/external/source-results.json),
[normalized results](../data/external/normalized-results.json), and
[paired comparisons](../data/external/paired-results.json) retain all cases and settings.

```sh
cargo build --release --locked
python3 scripts/experiment.py --binary target/release/buggraph \
  --engine-revision 335ffd5 --corpus data/external/source-corpus.json \
  --suite data/external/source-suite.json --models gpt-4o \
  --budgets 128 256 512 --k 1 3 --variants original \
  --output /tmp/buggraph-source-results.json
python3 scripts/experiment.py --binary target/release/buggraph \
  --engine-revision 335ffd5 --corpus data/external/normalized-corpus.json \
  --suite data/external/normalized-suite.json --models gpt-4o \
  --budgets 128 256 512 --k 1 3 --variants original \
  --output /tmp/buggraph-normalized-results.json
python3 scripts/compare_results.py \
  --source-report /tmp/buggraph-source-results.json \
  --candidate-report /tmp/buggraph-normalized-results.json \
  --source-suite data/external/source-suite.json \
  --candidate-suite data/external/normalized-suite.json \
  --output /tmp/buggraph-paired-results.json
```

## Decision and next discriminating test

A separate Astra consultation reviewed the methodology and supplied observations
without rerunning the experiment. It agreed that the narrow normalization result is
promising but UltraFuzz-specific value remains unproven. This is an AI second opinion,
not independent human adjudication or approval.

The strongest simpler alternative is concise descriptions in UltraFuzz's existing
catalog, with no additional graph or system. The next frozen comparison must allow
that alternative to win:

1. Existing source descriptions.
2. Query-blind concise summaries under a fixed editorial rule and comparable lengths.
3. Frozen Buggraph normalized records, without automatic ancestor expansion.

Use a new batch of already-published retrospective finding descriptions selected by
declared source/date rules. Freeze the batch size, labels, ranker, one primary budget,
and analysis before scoring. Have labels checked without retrieval outputs, allowing
multiple acceptable classes and insufficient information. Report unbudgeted ranks,
budgeted results, unsupported matches, returned tokens, and whether the returned text
supports the label. Keep negative and ambiguous cases. Repeating sweeps or editing
queries on the existing suites is development, not confirmation.

For an **UltraFuzz-specific** adoption claim, an offline comparison against its actual
knowledge-selection outputs is still necessary. Neither those outputs nor independent
human relevance judgments were available in this study. The decision remains open;
this repository does not include an adapter or launch autonomous hunting campaigns.

Stop with a tie, loss, or inconclusive result when warranted. Extending sampling or
tuning until a positive result appears is not a valid route to proof. Any useful
addition must survive a matched strong baseline, externally checked judgments,
consumer-relevant cost accounting, and explicit reporting of regressions.
