"""Regression checks for advisory identity and release-branch preservation."""

import hashlib
import json
import unittest
from unittest.mock import patch

import import_solidity_advisories as importer


class AdvisoryImportTests(unittest.TestCase):
    def import_rows(self, rows):
        raw = json.dumps(rows).encode()
        with patch.object(importer, "SHA256", hashlib.sha256(raw).hexdigest()):
            return importer.build(raw)

    def test_release_branches_keep_their_own_conditions(self):
        common = dict(uid="SOL-2019-3", summary="Encoding changes stored values.", severity="low")
        rows = [
            dict(common, name="Encoder", introduced="0.5.0", fixed="0.5.7",
                 conditions={"ABIEncoderV2": True}),
            dict(common, name="Encoder_0.4.x", introduced="0.4.19", fixed="0.4.26",
                 conditions={"optimizer": False}),
        ]
        corpus = self.import_rows(rows)
        self.assertEqual(len(corpus["nodes"]), 2)
        finding = corpus["nodes"][1]
        ranges = [json.loads(item) for item in finding["applicability"]]
        self.assertEqual([(r["introduced"], r["fixed"]) for r in ranges],
                         [("0.5.0", "0.5.7"), ("0.4.19", "0.4.26")])
        self.assertEqual(ranges[1]["conditions"], {"optimizer": False})
        self.assertEqual(finding["mappings"], ["SOL-2019-3", "Encoder", "Encoder_0.4.x"])
        self.assertEqual(len(corpus["edges"]), 1)

    def test_missing_introduction_stays_unspecified(self):
        corpus = self.import_rows([dict(uid="SOL-2019-1", name="CompilerDefect",
                                       summary="Unexpected result.", fixed="0.5.0", severity="low")])
        self.assertIsNone(json.loads(corpus["nodes"][1]["applicability"][0])["introduced"])

    def test_unpinned_source_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
            importer.build(b"[]")

    def test_conflicting_summaries_are_rejected(self):
        row = dict(uid="SOL-2019-1", name="CompilerDefect", summary="First.",
                   fixed="0.5.0", severity="low")
        with self.assertRaisesRegex(ValueError, "Conflicting advisory identity"):
            self.import_rows([row, dict(row, summary="Different.")])


if __name__ == "__main__":
    unittest.main()
