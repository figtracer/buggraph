import unittest

from compare_results import paired_outcomes


class PairTests(unittest.TestCase):
    def test_two_wins_do_not_establish_strong_evidence(self):
        baseline = [{"id": str(i), "relevant": 1, "found": 0} for i in range(8)]
        candidate = [{**case, "found": int(i < 2)} for i, case in enumerate(baseline)]
        result = paired_outcomes(baseline, candidate)
        self.assertEqual(result["wins"], ["0", "1"])
        self.assertEqual(result["losses"], [])
        self.assertEqual(len(result["ties"]), 6)
        self.assertEqual(result["two_sided_sign_p"], 0.5)

    def test_pairs_reject_missing_or_duplicate_cases(self):
        case = {"id": "one", "relevant": 1, "found": 0}
        with self.assertRaises(ValueError):
            paired_outcomes([case], [])
        with self.assertRaises(ValueError):
            paired_outcomes([case, case], [case])

    def test_all_ties_and_negative_queries(self):
        cases = [{"id": "positive", "relevant": 1, "found": 1}, {"id": "negative", "relevant": 0, "found": 0}]
        result = paired_outcomes(cases, cases)
        self.assertEqual(result["ties"], ["positive"])
        self.assertEqual(result["two_sided_sign_p"], 1.0)


if __name__ == "__main__":
    unittest.main()
