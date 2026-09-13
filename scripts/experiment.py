#!/usr/bin/env python3
"""Run local taxonomy retrieval ablations; never launch agents or analyze contracts."""

import argparse
import copy
import hashlib
import json
import pathlib
import subprocess
import tempfile


def most_specific_suite(suite, corpus):
    """Remove a labeled ancestor only when its descendant is also labeled."""
    parents = {node["id"]: [] for node in corpus["nodes"]}
    for edge in corpus["edges"]:
        if edge["relation"] == "specializes":
            parents[edge["from"]].append(edge["to"])
    result = copy.deepcopy(suite)
    result["revision"] += "-most-specific"
    result["description"] += " Ancestor labels redundant with labeled descendants removed."
    removed = 0
    for case in result["cases"]:
        broader = set()
        for label in case["relevant_ids"]:
            pending = list(parents[label])
            visited = set()
            while pending:
                parent = pending.pop()
                if parent not in visited:
                    visited.add(parent)
                    broader.add(parent)
                    pending.extend(parents[parent])
        before = len(case["relevant_ids"])
        case["relevant_ids"] = [label for label in case["relevant_ids"] if label not in broader]
        removed += before - len(case["relevant_ids"])
    return result, removed


def positive_integer(value):
    number = int(value)
    if number <= 0:
        raise argparse.ArgumentTypeError("expected a positive integer")
    return number


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=pathlib.Path)
    parser.add_argument("--engine-revision", required=True)
    parser.add_argument("--corpus", required=True, type=pathlib.Path)
    parser.add_argument("--suite", required=True, type=pathlib.Path)
    parser.add_argument("--models", required=True, nargs="+")
    parser.add_argument("--budgets", required=True, nargs="+", type=positive_integer)
    parser.add_argument("--k", required=True, nargs="+", type=positive_integer)
    parser.add_argument(
        "--variants", nargs="+", choices=["original", "most_specific"],
        default=["original", "most_specific"],
        help="Label variants to score; both preserves the original ablation workflow.",
    )
    parser.add_argument("--output", required=True, type=pathlib.Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    corpus_path = args.corpus.resolve()
    corpus = json.loads(corpus_path.read_text())
    suite = json.loads(args.suite.read_text())
    # The Rust engine validates references and DAG semantics before transformation.
    subprocess.run([str(binary), "validate", str(corpus_path)], check=True, capture_output=True)
    # Validate original suite labels before looking them up in the graph.
    subprocess.run([
        str(binary), "eval", str(corpus_path), str(args.suite.resolve()),
        args.models[0], str(args.budgets[0]), str(args.k[0]),
    ], check=True, capture_output=True)
    specific, removed = most_specific_suite(suite, corpus)
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    result = {
        "engine_revision": args.engine_revision,
        "binary_sha256": digest(binary),
        "corpus_sha256": digest(corpus_path),
        "suite_sha256": digest(args.suite),
        "runner_sha256": digest(pathlib.Path(__file__)),
        "removed_ancestor_labels": removed,
        "specific_labels": {case["id"]: case["relevant_ids"] for case in specific["cases"]},
        "runs": [],
    }
    with tempfile.TemporaryDirectory(prefix="buggraph-eval-") as temporary:
        for variant, data in [("original", suite), ("most_specific", specific)]:
            if variant not in args.variants:
                continue
            path = pathlib.Path(temporary) / f"{variant}.json"
            path.write_text(json.dumps(data))
            for model in dict.fromkeys(args.models):
                for budget in dict.fromkeys(args.budgets):
                    for k in dict.fromkeys(args.k):
                        run = subprocess.run([
                            str(binary), "eval", str(corpus_path), str(path),
                            model, str(budget), str(k),
                        ], check=True, capture_output=True, text=True)
                        report = json.loads(run.stdout)
                        # Retain all per-case outputs, not only winning configurations.
                        result["runs"].append({"variant": variant, **report})
    args.output.write_text(json.dumps(result, separators=(",", ":")) + "\n")
    print(f"Wrote {len(result['runs'])} evaluations; removed {removed} redundant ancestor labels.")


if __name__ == "__main__":
    main()
