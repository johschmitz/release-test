#!/usr/bin/env python3
"""Diff-test harness (IMPLEMENTATION_PLAN §9.3).

Runs the Python qc_opendrive reference and the Rust xodr-qcr binary on the
same .xodr fixture and compares the normalized issue sets. The comparison key
is (checker_id, rule_uid, level, xpath) — matching the success criteria that
"All 26 checkers produce identical issue counts + xpaths to the Python
reference on the bundled tests/data fixtures".

Usage:
    diff_test.py <INPUT.xodr> [--rust-binary PATH]

Exit code 0 if the normalized issue sets match, 1 otherwise.
"""
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
PYTHON_REF = REPO / "tools" / "run_python_ref.py"
DEFAULT_RUST = REPO / "target" / "release" / "xodr-qcr"


def run_python(input_path: Path, out_xqar: Path) -> None:
    env = dict()
    # Ensure qc_opendrive and qc_baselib are importable.
    import os

    pypath = str(REPO / "qc-opendrive")
    existing = os.environ.get("PYTHONPATH", "")
    env["PYTHONPATH"] = pypath + (":" + existing if existing else "")
    # Use the venv python if present.
    python = REPO / ".venv" / "bin" / "python"
    if not python.exists():
        python = Path(sys.executable)
    subprocess.run(
        [str(python), str(PYTHON_REF), str(input_path), str(out_xqar)],
        env={**os.environ, **env},
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=True,
    )


def run_rust(input_path: Path, out_xqar: Path, rust_bin: Path) -> None:
    # Exit code 1 means ERROR-level issues were found (intended CI behavior);
    # only 2 (usage/file error) is a real failure.
    proc = subprocess.run(
        [str(rust_bin), str(input_path), "-o", str(out_xqar), "-q"],
    )
    if proc.returncode not in (0, 1):
        raise subprocess.CalledProcessError(proc.returncode, proc.args)


def normalize_xpath(xpath: str) -> str:
    """Canonicalize an xpath so lxml-style (/OpenDRIVE/road/.../lane[1], no
    index on root or on single-occurrence containers) and roxmltree-style
    (/OpenDRIVE[1]/road[1]/lanes[1]/.../lane[1], index everywhere) compare
    equal. We strip all [n] predicates, leaving the bare tag path."""
    import re

    if not xpath:
        return ""
    # Remove every [digits] predicate.
    stripped = re.sub(r"\[\d+\]", "", xpath)
    return stripped


LEVEL_MAP = {"1": "error", "2": "warning", "3": "information",
             "error": "error", "warning": "warning", "information": "information"}


def normalize_level(level: str) -> str:
    return LEVEL_MAP.get(level, level)


def collect_issues(xqar_path: Path):
    """Return a dict mapping checker_id -> list of (rule_uid, level, xpath)."""
    tree = ET.parse(xqar_path)
    root = tree.getroot()
    issues = {}
    for checker in root.iter("Checker"):
        cid = checker.get("checkerId")
        for issue in checker.iter("Issue"):
            rule = issue.get("ruleUID")
            level = normalize_level(issue.get("level") or "")
            xpaths = [normalize_xpath(xl.get("xpath") or "") for xl in issue.iter("XMLLocation")]
            # If multiple xml locations, join for a stable key.
            xpath = " | ".join(xpaths) if xpaths else ""
            issues.setdefault(cid, []).append((rule, level, xpath))
    return issues


def normalize(issues):
    """Sort and count issues per (checker_id, rule_uid, level, xpath)."""
    counts = {}
    for cid, lst in issues.items():
        for key in lst:
            counts[(cid, *key)] = counts.get((cid, *key), 0) + 1
    return counts


def main() -> int:
    args = sys.argv[1:]
    rust_bin = DEFAULT_RUST
    if "--rust-binary" in args:
        i = args.index("--rust-binary")
        rust_bin = Path(args[i + 1])
        args = args[:i] + args[i + 2 :]
    if not args:
        print("usage: diff_test.py <INPUT.xodr> [--rust-binary PATH]", file=sys.stderr)
        return 2
    input_path = Path(args[0])

    ref_xqar = Path("/tmp/diff_ref.xqar")
    rust_xqar = Path("/tmp/diff_rust.xqar")

    run_python(input_path, ref_xqar)
    run_rust(input_path, rust_xqar, rust_bin)

    ref_counts = normalize(collect_issues(ref_xqar))
    rust_counts = normalize(collect_issues(rust_xqar))

    all_keys = sorted(set(ref_counts) | set(rust_counts))
    mismatches = []
    for key in all_keys:
        rc = ref_counts.get(key, 0)
        mc = rust_counts.get(key, 0)
        if rc != mc:
            mismatches.append((key, rc, mc))

    if not mismatches:
        print(f"MATCH: {input_path.name} ({sum(ref_counts.values())} issues)")
        return 0

    print(f"MISMATCH: {input_path.name}")
    for key, rc, mc in mismatches:
        print(f"  {key[0]} rule={key[1]} level={key[2]} xpath='{key[3]}' "
              f"ref={rc} rust={mc}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
