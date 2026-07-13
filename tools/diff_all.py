#!/usr/bin/env python3
"""Run diff_test.py across every .xodr fixture under tests/data.

Usage:
    diff_all.py [--rust-binary PATH]
"""
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DIFF = REPO / "tools" / "diff_test.py"
DATA = REPO / "tests" / "data"

# Fixtures in these subdirs are large/example files or cover rules that are
# out of scope for the 26-checker port (the Python repo's not_implemented_yet
# dir holds extra rules we deliberately don't implement). Skip by default.
SKIP_DIRS = {"examples", "not_implemented_yet"}


def run_with_timeout(cmd, timeout_s=120):
    """Run cmd, returning (returncode, stdout, stderr). Kills if it exceeds timeout."""
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        out, err = proc.communicate(timeout=timeout_s)
    except subprocess.TimeoutExpired:
        proc.kill()
        out, err = proc.communicate()
        return 124, out, err + f"\n[TIMEOUT after {timeout_s}s]"
    return proc.returncode, out, err


def main() -> int:
    rust_args = []
    use_all = "--all" in sys.argv
    if "--rust-binary" in sys.argv:
        i = sys.argv.index("--rust-binary")
        rust_args = ["--rust-binary", sys.argv[i + 1]]

    xodr_files = sorted(DATA.rglob("*.xodr"))
    if not xodr_files:
        print(f"No .xodr files found under {DATA}", file=sys.stderr)
        return 2

    passed = 0
    failed = 0
    for xf in xodr_files:
        if not use_all and any(part in SKIP_DIRS for part in xf.parts):
            continue
        cmd = [sys.executable, str(DIFF), str(xf), *rust_args]
        rc, out, err = run_with_timeout(cmd, timeout_s=120)
        summary = ""
        for line in out.splitlines():
            if line.startswith(("MATCH", "MISMATCH")):
                summary = line
        if rc == 0:
            passed += 1
            print(f"[PASS] {summary}")
        else:
            failed += 1
            print(f"[FAIL] {xf}")
            print(out)
            print(err)
    print(f"\n{passed} passed, {failed} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
