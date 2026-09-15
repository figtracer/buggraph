#!/usr/bin/env python3
"""Build retrospective advisory records from the pinned Solidity registry."""

import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path

REVISION = "72ef01eaaa3e26c5076b18ac0c05eddc7792337c"
SHA256 = "bb504a299e08cde52a48fb6a158a9ce61e2a70a8bea676550edbd6e89718ef2b"
SOURCE_URL = f"https://github.com/argotorg/solidity/blob/{REVISION}/docs/bugs.json"
SOURCE_ID = "solidity:known-bugs"


def build(raw):
    if hashlib.sha256(raw).hexdigest() != SHA256:
        raise ValueError("Source SHA-256 mismatch")
    groups = defaultdict(list)
    for row in json.loads(raw):
        for key in ("uid", "name", "summary", "fixed", "severity"):
            if not isinstance(row.get(key), str) or not row[key].strip():
                raise ValueError(f"Missing advisory field: {key}")
        groups[row["uid"]].append(row)

    def node(record_id, kind, summary, definition, facets):
        return dict(
            id=record_id,
            kind=kind,
            summary=summary,
            definition=definition,
            applicability=[],
            exclusions=[],
            facets=facets,
            sources=[SOURCE_ID],
            mappings=[],
            review="imported",
        )

    root = node(
        "solidity:compiler-correctness", "failure_mode",
        "Compiler defects change program behavior.",
        "The Solidity known-bugs registry documents compiler defects and their fixes.",
        ["component:compiler", "dataset:solidity-advisories"],
    )
    root["review"] = "source_checked"
    nodes = [root]
    edges = []
    for uid, variants in sorted(groups.items()):
        if len({row["summary"] for row in variants}) != 1:
            raise ValueError(f"Conflicting advisory identity: {uid}")
        record = node(
            f"solidity:{uid.lower()}", "finding", variants[0]["summary"],
            variants[0]["summary"],
            ["component:compiler", "dataset:solidity-advisories"]
            + sorted({f"severity:{row['severity']}" for row in variants}),
        )
        record["mappings"] = [uid] + sorted({row["name"] for row in variants})
        for row in variants:
            condition = {
                "name": row["name"],
                "introduced": row.get("introduced"),
                "fixed": row["fixed"],
                "conditions": row.get("conditions", {}),
            }
            record["applicability"].append(json.dumps(condition, sort_keys=True))
        record["exclusions"] = [
            "Version and setting matches alone do not establish that a contract is affected.",
            "An absent introduced version is unspecified, not version zero.",
            "Conditions within one range apply together; separate ranges are alternatives.",
        ]
        nodes.append(record)
        edges.append({"from": record["id"], "relation": "instance_of", "to": root["id"]})
    return dict(
        revision=f"solidity-advisories-{REVISION[:12]}-v1",
        sources=[dict(id=SOURCE_ID, title="Solidity contributors: Known Bugs",
                      url=SOURCE_URL, revision=REVISION, license="GPL-3.0-only")],
        nodes=nodes, edges=edges,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    corpus = build(args.source.read_bytes())
    args.output.write_text(json.dumps(corpus, indent=2, ensure_ascii=False) + "\n")
