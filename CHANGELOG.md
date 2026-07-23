# Changelog

## [0.1.0] - 2026-07-23

### Added

- Initial release of the Rust rewrite of the ASAM OpenDRIVE Quality Checker (`qc-opendrive`).
- CLI binary `xodr-qcr` that checks `.xodr` files for quality issues.
- Full checker suite (26 checkers) ported from the Python reference:
  - 4 basic checks (valid XML document, root tag, file header, version defined).
  - Schema validation against bundled OpenDRIVE XSDs (1.4–1.7 XSD 1.0, 1.8 XSD 1.1).
  - 12 semantic checks, 7 geometry checks, 1 performance check, 1 smoothness check.
- Text report to stdout and optional `.xqar` XML report (`-o/--output`).
- Precondition and version-gating logic matching the Python `main.py`.
- Diff-test harness (`tools/diff_test.py` / `tools/diff_all.py`) verifying identical
  issue sets (checker id, rule uid, level, xpath) to the Python reference across all
  bundled fixtures (121 passed, 0 failed).

[0.1.0]: https://github.com/wo9xr4d/xodr-qcr/releases/tag/v0.1.0
