"""Tests for evaluation-label transformation, independent of retrieval scores."""

import unittest

from experiment import most_specific_suite


class LabelTests(unittest.TestCase):
    def test_removes_only_redundant_ancestors_and_preserves_original(self):
        corpus = {
            "nodes": [{"id": node} for node in ["root", "left", "right", "leaf", "other"]],
            "edges": [
                {"from": child, "relation": "specializes", "to": parent}
                for child, parent in [
                    ("left", "root"), ("right", "root"), ("leaf", "left"), ("leaf", "right")
                ]
            ],
        }
        suite = {
            "revision": "test",
            "description": "diamond",
            "cases": [
                {"relevant_ids": ["root", "left", "right", "leaf", "other"]},
                {"relevant_ids": ["root"]},
                {"relevant_ids": []},
            ],
        }
        result, removed = most_specific_suite(suite, corpus)
        self.assertEqual(removed, 3)
        self.assertEqual(result["cases"][0]["relevant_ids"], ["leaf", "other"])
        self.assertEqual(result["cases"][1]["relevant_ids"], ["root"])
        self.assertEqual(result["cases"][2]["relevant_ids"], [])
        self.assertEqual(len(suite["cases"][0]["relevant_ids"]), 5)


if __name__ == "__main__":
    unittest.main()
