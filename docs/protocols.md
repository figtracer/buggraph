# Protocol audit knowledge

`data/protocols.json` separates reusable failure modes from historical findings.
Five findings from three published Code4rena reports support five narrow modes:

| Failure mode | Published example | Report status |
| --- | --- | --- |
| Inconsistent liquidation reward units | Size H-04 | Confirmed; fix reported |
| Delisting blocks existing collateral exits | INIT M-09 | Acknowledged |
| Residual debt lacks loss recognition | INIT M-10 | Confirmed |
| User withdrawal counters share a reset marker | prePO H-01 | Confirmed |
| Deposit credit exceeds net receipts | prePO M-02 | Confirmed; final severity Medium |

The graph contains 17 nodes and 18 edges. `specializes` connects narrower modes
to broader categories, `violates` links modes to properties, and `instance_of`
connects historical findings to modes. The liquidation reward mode has both an
accounting parent and a liquidation parent. These relationships are Bugraph's
curation, not labels supplied by Code4rena or OWASP.

```sh
bugraph explore data/protocols.json gpt-4o 4096 full 5 2 "liquidation reward" --compact
bugraph instances data/protocols.json bm25 gpt-4o 4096 full "withdrawal" --compact
bugraph resolve data/protocols.json gpt-4o 4096 full mode:delisted-collateral-exit finding:c4:init-m09 --compact
```

Mode records describe root cause, applicability, exclusions, and corrective design
principles. Finding records retain report severity and sponsor disposition separately
from Bugraph's `source_checked` status. A reported fix is not independent verification
of that fix. Findings describe historical audit scopes, not current deployments.

INIT M-09 specifically preserves the report's qualification: under the sponsor's
stated delisting sequence, the remaining issue is blocked withdrawal. It is not
classified here as demonstrated liquidation denial of service.

Each citation points to the exact report section and includes a SHA-256 digest of
the complete HTML response observed on 2026-09-15. URLs are mutable; the digest
identifies the observed source, not an immutable hosting guarantee. Raw report
snapshots are retained locally during curation, not redistributed in the repository.
Original issue links are retained as mappings. No report code or reproductions are
bundled. See [attribution](../data/ATTRIBUTION.md).

This corpus is separately selectable. It does not modify OWASP source records or
automatically add new classes to UltraFuzz's SCWE planner catalog. Existing authored
retrieval diagnostics are not independent evaluation of these new records.
