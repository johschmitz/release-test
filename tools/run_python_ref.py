#!/usr/bin/env python3
"""Run the Python qc_opendrive reference checker on a single .xodr file and
write the resulting .xqar report.

Usage:
    run_python_ref.py <INPUT.xodr> <OUTPUT.xqar>

This is the reference side of the diff-test harness (IMPLEMENTATION_PLAN §9.3).
It drives qc_opendrive.main.run_checks directly with an in-memory Configuration
so we don't need a config XML file.
"""
import sys

from qc_baselib import Configuration, Result

from qc_opendrive import constants
from qc_opendrive.main import run_checks


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: run_python_ref.py <INPUT.xodr> <OUTPUT.xqar>", file=sys.stderr)
        return 2

    input_path = sys.argv[1]
    output_path = sys.argv[2]

    config = Configuration()
    config.set_config_param("InputFile", input_path)

    result = Result()
    result.register_checker_bundle(
        name=constants.BUNDLE_NAME,
        description="OpenDrive checker bundle",
        version=constants.BUNDLE_VERSION,
        summary="",
    )
    result.set_result_version(version=constants.BUNDLE_VERSION)

    run_checks(config, result)

    result.write_to_file(output_path, generate_summary=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
