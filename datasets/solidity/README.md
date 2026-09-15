# Solidity compiler advisories

62 distinct published compiler advisories, preserving all 66 affected-version
entries in the pinned Solidity known-bugs registry. Each advisory is a finding
linked to the compiler-correctness category. Summaries, names, severity labels,
version ranges, and compiler conditions come from Solidity contributors.

```sh
bugraph instances datasets/solidity/corpus.json bm25 gpt-4o 4096 full "ABI encoding"
bugraph show datasets/solidity/corpus.json solidity:sol-2019-3
```

These records support retrospective classification and compiler maintenance.
They do not determine whether a particular contract is affected. Version ranges
remain separate alternatives; conditions within each range apply together.
An absent introduction version remains unspecified. Fixed versions are upstream
metadata, not a substitute for checking the release policy of a project.

## Source and reproduction

Source: [Solidity contributors, Known Bugs](https://github.com/argotorg/solidity/blob/72ef01eaaa3e26c5076b18ac0c05eddc7792337c/docs/bugs.json).
Revision: `72ef01eaaa3e26c5076b18ac0c05eddc7792337c`.
Input SHA-256: `bb504a299e08cde52a48fb6a158a9ce61e2a70a8bea676550edbd6e89718ef2b`.
Retrieved 2026-09-15.

Save that revision's raw `docs/bugs.json`, then run from the repository root:

```sh
python3 scripts/import_solidity_advisories.py /path/to/bugs.json datasets/solidity/corpus.json
bugraph validate datasets/solidity/corpus.json
```

The importer rejects different source bytes. It groups entries sharing a `SOL-`
identifier without discarding release-branch variants. The corpus retains summary
metadata; full technical descriptions and advisory links remain in the pinned
upstream registry. It does not bundle detection expressions or reproductions.
Advisories are marked `imported`; the grouping category was source-checked with
OpenAI Codex assistance. No independent audit review is claimed.

## License

This corpus is derived from the Solidity repository, distributed under
[GNU GPL version 3](LICENSE), with attribution to Solidity contributors.
It is a separately selectable data collection; the MIT code license and the
CC-BY-SA-4.0 terms for `data/` do not relicense it. The transformation script is MIT.
