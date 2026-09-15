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
from audits or incidents are included in `curated.json`.

`eval.json` contains original AI-authored diagnostic questions and relevance labels.
Topic groups are disjoint across dev/test, but the same author created the corpus
adaptations and questions. This is not independent evidence of retrieval quality on
unseen audit reports.

Files under `data/` use CC-BY-SA-4.0. Retain attribution and identify changes when
redistributing adaptations. Linked sources retain their upstream terms. Code and
documentation outside `data/` are covered by the root MIT license.

## Vault standards extension

`vaults.json` contains six original failure-mode adaptations and two properties
from ERC-4626 (Joey Santoro et al.) and ERC-7540 (Jeroen Offerijns et al.). Each
source registry entry pins the exact Ethereum ERCs commit and credits the authors.
Both specifications waive rights under CC0-1.0; these adaptations are distributed
under the data directory’s CC-BY-SA-4.0 terms. Accessed 2026-09-15.

Changes: concise failure-mode descriptions, applicability and exclusions, facets,
and property relationships. Source checking was performed with OpenAI Codex
assistance, without independent human review. These are standards-derived records,
not published audit findings or claims of novel bug classes absent from OWASP.
The corpus is separately selectable; `owasp.json` remains the pinned OWASP import.

## Protocol audit findings

`protocols.json` contains original factual summaries of sixteen published findings
and sixteen generalized failure modes, five broader categories, and three properties.
Sources are Code4rena's Size (2024-06), INIT Capital (2023-12), and prePO (2022-12)
reports; Morpho Midnight's Blackthorn and Cantina reports (2026); Morpho Vault V2's
Cantina competition report (2025); Cantina's Bitcorn OFT report (2025); and Cantina's
Silo Finance Dynamic Kink review (2025). Each
source entry identifies the finding section, credits the named researchers or
review team, and records the SHA-256 of the observed HTML or complete PDF. The
Morpho PDFs are linked at pinned GitHub commits. The sources supply final severity
and sponsor disposition where provided. Full author lists remain in the cited reports.

The prose and graph relationships were curated with OpenAI Codex assistance and
checked against those sources. `source_checked` does not mean independent human
review or vulnerability reproduction. Fix claims are explicitly attributed to the
report; no present-day deployment status is inferred.

These original summaries and classifications use CC-BY-SA-4.0. Source reports retain
their own rights: source license metadata is `NOASSERTION`, not a claim that report
text is CC-licensed. Report text, code, and reproductions are not redistributed.
This corpus is a starting collection, not an exhaustive audit taxonomy.
