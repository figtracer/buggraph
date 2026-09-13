# Data and graph semantics

Both `data/owasp.json` and `data/curated.json` contain a corpus revision, source
registry, nodes, and typed edges.
The versioned JSON files are the editable source of truth. Corpus revision labels
must change when definitions or relationships change; Git identifies exact content.

Nodes have stable IDs, a kind (`failure_mode`, `property`, or `finding`), a summary,
definition, explicit facets, applicability conditions, exclusions, source IDs,
external mappings, and a review status. Draft is the default. Both `imported` and
`source_checked` require resolvable source IDs. `imported` means a source record was transferred without
independent content adjudication. `source_checked` means the adaptation was checked
against the cited source, not that a human auditor endorsed it.

Sources identify a title, HTTPS URL, revision, and license. Starter sources link to
a fixed OWASP Git commit. The engine validates references and required metadata;
it does not authenticate source claims, fetch URLs, or adjudicate license compatibility.

| Edge | Allowed endpoints | Meaning |
| --- | --- | --- |
| `specializes` | failure mode → failure mode | The child is a narrower case of the parent. |
| `violates` | failure mode → property | The failure breaks this expected property. |
| `instance_of` | finding → failure mode | A documented finding illustrates a class. |
| `related_to` | any → any | A directional cross-reference without inheritance. |

Specialization allows multiple parents and must be acyclic. Duplicate edges, duplicate
IDs, and unknown endpoints are rejected. `related_to` is directional; add both
directions when needed. It is not followed by ancestor retrieval.

Facets use `dimension:value` strings and AND semantics. They are explicit and not
inherited. Missing facets are not proof of non-applicability. Ancestors must also pass
the requested facet filters.

The OWASP reference corpus uses canonical `scwe:NNN` IDs and copies upstream
Description sections. Category facets come from source directories, not inferred
applicability. It has no inferred specialization edges. Its inventory covers a pinned
156-entry snapshot, not every smart contract bug class. The separate curated corpus
retains its existing `fm:` IDs and evaluation labels; the two files are not silently
merged or deduplicated. Source references link to complete upstream documents,
including examples that are not embedded in the catalog.

## Assessment ledger

The separate ledger binds a unique scope to a corpus revision and a design/scope
revision. Each explicit state requires evidence or an explanation. Missing records
remain unassessed; unresolved applicability and exclusions retain separate counts.
See `data/ledger.json`. Evidence references are recorded, not verified. No retrieval
operation changes assessment states. Counts are not a percentage of protocol security.
