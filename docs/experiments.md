# Follow-up experiments

Buggraph's current benefit is controlled access to reusable, attributable knowledge.
Improved agent reasoning or vulnerability detection remains a hypothesis.

## What we ran

32 local evaluations, each comparing ID order, BM25, and BM25 with ancestors:

- Two tokenizers selected by `gpt-4` and `gpt-4o`; no model calls.
- Four returned-context caps: 128, 256, 512, and 1,024 tokens.
- Two maximum record counts: k=1 and k=3.
- Original relevance labels and a most-specific-label variant.

The budgets bracket the measured size of one to several summary records. The record
caps distinguish first-result quality from the broader context in a three-result
answer. They are experimental settings, not recommended production defaults.

The original labels sometimes mark both a specific failure mode and its parent as
relevant. The variant removes a parent only if another labeled result specializes it,
following all specialization edges transitively. It removes five redundant ancestor
labels across the suite. Original inputs are unchanged. This tests whether graph
expansion retrieves additional specific classes or mainly receives credit for parents.

All runs use the same engine, text, facets, queries, and ranking parameters. Changing
labels only changes scoring. The runner retains every result and records corpus,
suite, runner, and binary hashes. The binary hash is specific to the build platform.

```sh
cargo build --release --locked
python3 scripts/experiment.py \
  --binary target/release/buggraph \
  --engine-revision 335ffd5 \
  --corpus data/curated.json --suite data/eval.json \
  --models gpt-4 gpt-4o --budgets 128 256 512 1024 --k 1 3 \
  --output /tmp/buggraph-experiment-results.json
```

The recorded run used engine commit `335ffd5`. When experimenting with another
engine revision, pass its actual commit and compare behavior, not binary hashes.
Python 3 is used only for the experiment driver. Retrieval and tokenization run
in the Rust executable. See [all raw results](../data/experiment-results.json).

## Results

The table shows the test split with most-specific labels, `gpt-4o`, and k=3.
It contains 11 positive queries and three negative queries. Mean tokens includes both.

| Cap | BM25 specific recall | With ancestors | BM25 mean tokens | With ancestors |
| --- | ---: | ---: | ---: | ---: |
| 128 | 1.000 | 1.000 | 97.7 | 97.7 |
| 256 | 1.000 | 1.000 | 194.6 | 193.6 |
| 512 | 1.000 | 1.000 | 307.4 | 304.3 |
| 1,024 | 1.000 | 1.000 | 307.4 | 304.3 |

The `gpt-4` tokenizer shows the same specific-recall pattern. Its BM25 mean context
is 96.9 tokens at the 128-token cap and 305.4 at the 512-token cap. These are
tokenizer comparisons, not comparisons of the models' reasoning abilities.

Reducing the cap from 512 to 128 cuts mean returned BM25 context by 68.2% for
`gpt-4o`, while preserving the measured specific-label recall. This does not prove
that a reader can make equally good decisions from the shorter text. Total prompts,
tool descriptions, follow-up expansions, output tokens, and model costs were not measured.

With original labels and a 512-token cap, graph recall is 1.000 versus 0.909 for BM25.
That apparent gain disappears when redundant parent labels are removed. The current
suite therefore supports graph expansion as a source of broader explanatory context,
not as evidence of better specific-class recognition. k=1 already saturates specific
test recall in both lexical modes at these budgets, showing how easy this suite is.

Both lexical modes return nonempty results for the two exclusion near misses. They
correctly return nothing for the three unrelated queries. Applicability notes and
exclusions are available through `show`, but retrieval alone does not interpret them.

## What we can claim

| Claim | Evidence or next measurement |
| --- | --- |
| Returned context respects a chosen tokenizer budget. | Enforced by recounting output; tested with two encodings. |
| Smaller context can preserve classification-label recall. | Observed on this authored suite; needs independent labels and reader evaluation. |
| The graph retrieves broader context. | Observed through ancestor expansion and the label ablation. |
| The graph finds additional specific classes. | Not demonstrated by this suite. |
| Source provenance and stable IDs make runs inspectable. | Returned source links, revisions, and recorded per-case outputs. |
| Agents produce fewer incorrect findings or spend less overall. | Not measured. |
| More graph traversal means a more secure protocol. | Not a valid interpretation of taxonomy coverage. |

Use BM25 as the working baseline. Keep ancestor expansion explicit and evaluate it
for tasks that need explanation or generalization. Do not choose it just because a
parent-inclusive metric is higher.

## Ultrafuzz comparison

[Ultrafuzz's overview](https://github.com/monad-developers/ultrafuzz/blob/89b57c9a7c5aa22af15e1ae9625b8ab3a2c6f810/README.md)
describes an agent orchestrator that collects generated tests and findings for review.
It is a plausible consumer of structured knowledge, but no Buggraph adapter or
Ultrafuzz campaign was implemented or run for this experiment. No saved Ultrafuzz
queries or completed reports were available for a retrospective comparison.

The next comparison should replay saved knowledge requests and use independently
labeled, published finding summaries for classification and review. Keep query inputs
separate from answer labels. Match source coverage before comparing corpus formats:
our 16 selected OWASP sources are not a replacement for the entire OWASP catalog.

| Comparison | Question it isolates |
| --- | --- |
| Existing saved knowledge context versus normalized records from the same sources | Does structured presentation help the reviewer? |
| Full normalized catalog versus budgeted BM25 retrieval | Can context shrink without losing useful evidence? |
| BM25 versus BM25 with graph context, identical text and budgets | Does the graph improve interpretation or generalization? |
| Summary-only versus expanded applicability and exclusions | Does more detail help reject plausible but inapplicable matches? |

Freeze the corpus and labels before comparing methods. Keep related reports and
protocol families together across splits, reserve a temporal holdout, and use an
independent reviewer for labels and disagreements. If a model participates in the
offline classification comparison, hold its version and prompt fixed, repeat paired
runs, and account for all consumed tokens rather than only retrieved context.

Report specific-class recall, unsupported matches, source attribution, duplicate
references, relevant evidence per returned token, review time, and per-family errors.
Distinguish broad explanatory parents from specific failure-mode labels. Retain
negative and inconclusive results. No win is established unless it survives equal
budgets, matched source coverage, and independent judgments.

These experiments evaluate the knowledge layer. They do not configure or launch
Ultrafuzz's autonomous vulnerability-hunting pipeline.
