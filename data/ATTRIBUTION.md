# Data attribution and review status

`owasp.json` imports all 156 SCWE records present in the pinned snapshot. Titles and
complete Markdown source bytes are copied from the corresponding files, including
frontmatter, relationships, descriptions, remediation, examples, and fenced code;
155 records contain fenced examples. The snapshot also imports the SCSVS root and
11 groups, then connects each SCWE to its explicit `scsvs-cg` mapping. Group names
replace upstream `TBD` placeholders with concise expansions of the group IDs. Source
IDs are retained as mappings, and SCWE node IDs use `scwe:NNN`. Each record is marked
`imported`; no independent review is claimed. The deterministic `import-owasp`
command produced the snapshot without normalizing SCWE line endings or surrounding
whitespace. This is source inventory coverage, not exhaustive coverage of smart
contract weaknesses.

The separate `curated.json` starter taxonomy adapts 16 entries from the
[OWASP Smart Contract Weakness Enumeration](https://github.com/OWASP/owasp-scs),
by OWASP SCS contributors, at commit
`fefd476b83074666ada2d816f103436a18e1ece4`, accessed 2026-09-13.
Every registry entry links to its definition at that commit.
OWASP publishes this material under [CC-BY-SA-4.0](LICENSE).

Changes: concise rewritten definitions; narrower boundaries where appropriate;
new applicability and exclusion notes; independent facets; property links; three
synthesized broader concepts; and specialization relationships. The signature-identity
record narrows the source to identity/uniqueness assumptions and does not claim
that every malleable signature enables replay.

The 19 failure modes are source-checked; four property records remain draft.
Source comparisons and adaptations were performed with OpenAI Codex assistance.
No independent human review is claimed. This is not an exhaustive catalog, audit,
list of affected deployments, or OWASP-endorsed taxonomy. No finding instances
from audits or incidents are included.

`eval.json` contains original AI-authored diagnostic questions and relevance labels.
Topic groups are disjoint across dev/test, but the same author created the corpus
adaptations and questions. This is not independent evidence of retrieval quality on
unseen audit reports.

Files under `data/` use CC-BY-SA-4.0. Retain attribution and identify changes when
redistributing adaptations. Linked sources retain their upstream terms. Code and
documentation outside `data/` are covered by the root MIT license.
