#!/usr/bin/env python3
"""Pair offline classification outcomes; never infer deployment or detection benefit."""

import argparse
import hashlib
import json
import math
import pathlib


def paired_outcomes(baseline, candidate):
    left = {case["id"]: case for case in baseline}
    right = {case["id"]: case for case in candidate}
    if len(left) != len(baseline) or len(right) != len(candidate) or left.keys() != right.keys():
        raise ValueError("paired reports require identical unique case IDs")
    wins, losses, ties = [], [], []
    for case_id, before in left.items():
        after = right[case_id]
        if before["relevant"] != after["relevant"]:
            raise ValueError("paired reports disagree on relevance counts")
        if before["relevant"]:
            difference = after["found"] - before["found"]
            (wins if difference > 0 else losses if difference < 0 else ties).append(case_id)
    n = len(wins) + len(losses)
    tail = sum(math.comb(n, i) for i in range(min(len(wins), len(losses)) + 1))
    return {
        "wins": wins, "losses": losses, "ties": ties,
        "two_sided_sign_p": min(1.0, 2 * tail / 2**n) if n else 1.0,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-report", required=True, type=pathlib.Path)
    parser.add_argument("--candidate-report", required=True, type=pathlib.Path)
    parser.add_argument("--source-suite", required=True, type=pathlib.Path)
    parser.add_argument("--candidate-suite", required=True, type=pathlib.Path)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    args = parser.parse_args()
    source, candidate = [json.loads(path.read_text()) for path in [args.source_report, args.candidate_report]]
    source_suite, candidate_suite = [json.loads(path.read_text()) for path in [args.source_suite, args.candidate_suite]]
    if source_suite["cases"] != candidate_suite["cases"]:
        raise ValueError("queries, groups, splits, filters, and labels must match exactly")
    for report, path in [(source, args.source_suite), (candidate, args.candidate_suite)]:
        if report["suite_sha256"] != hashlib.sha256(path.read_bytes()).hexdigest():
            raise ValueError("report does not match its suite")
    def key(run):
        return run["variant"], run["model"], run["max_tokens"], run["k"]
    lookup = {key(run): run for run in candidate["runs"]}
    if len(lookup) != len(candidate["runs"]) or len({key(r) for r in source["runs"]}) != len(source["runs"]):
        raise ValueError("duplicate experimental configurations")
    if {key(run) for run in source["runs"]} != lookup.keys():
        raise ValueError("experimental configurations must match")
    rows = []
    for before in source["runs"]:
        after = lookup[key(before)]
        before_modes = {mode["mode"]: mode for mode in before["results"]}
        after_modes = {mode["mode"]: mode for mode in after["results"]}
        if before_modes.keys() != after_modes.keys():
            raise ValueError("retrieval modes must match")
        for mode, baseline in before_modes.items():
            rows.append({
                "variant": before["variant"], "model": before["model"],
                "max_tokens": before["max_tokens"], "k": before["k"], "mode": mode,
                **paired_outcomes(baseline["cases"], after_modes[mode]["cases"]),
                "source_metrics": baseline["metrics"], "candidate_metrics": after_modes[mode]["metrics"],
            })
    args.output.write_text(json.dumps({
        "interpretation": "Exploratory paired sign tests on positive query outcomes; configurations reuse cases, are dependent, and are not independent replications. No population or detection claim follows.",
        "source_report_sha256": hashlib.sha256(args.source_report.read_bytes()).hexdigest(),
        "candidate_report_sha256": hashlib.sha256(args.candidate_report.read_bytes()).hexdigest(),
        "comparisons": rows,
    }, indent=2) + "\n")


if __name__ == "__main__":
    main()
