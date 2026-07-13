// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Shared helpers for the integration tests, mirroring qc-opendrive/tests/test_setup.py.
//
// The Python harness drives the engine via `main.main()` and then loads the
// produced `.xqar`. Here we call the engine directly (mirroring `create_test_config`
// + `launch_main`) and return the in-memory `Result`.

// This module is compiled into each integration-test binary separately, so
// helpers that are unused by a particular test binary would otherwise trigger
// dead_code warnings. They are shared infrastructure, so silence that here.
#![allow(dead_code)]

use std::path::Path;

use roxmltree::Document;
use xodr_qcr::opendrive;
use xodr_qcr::opendrive::checks::{CheckerData, run_checks};
use xodr_qcr::result::{IssueSeverity, Result, StatusType};

/// Run the full checker suite on the file at `path`, returning the `Result`.
///
/// Mirrors `create_test_config` + `launch_main`: parse once, register the
/// bundle, run all checks, generate summaries.
pub fn run_on(path: &str) -> Result {
    let p = Path::new(path);
    let text = std::fs::read_to_string(p).expect("failed to read fixture");
    let doc = Document::parse(&text).ok();

    let mut result = Result::new();
    result.register_checker_bundle(
        opendrive::BUNDLE_NAME,
        "OpenDrive checker bundle",
        opendrive::BUNDLE_VERSION,
        "",
    );
    result.set_result_version(opendrive::BUNDLE_VERSION);

    let mut cd = CheckerData {
        xml_file_path: p,
        doc: doc.as_ref(),
        schema_version: None,
        result: &mut result,
    };

    run_checks(&mut cd);
    result.generate_summaries();
    result
}

/// Collect the set of xpath locations for all issues of a given rule UID.
fn collect_xpaths(result: &Result, rule_uid: &str) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for checker in &result.bundle.checkers {
        for issue in &checker.issues {
            if issue.rule_uid == rule_uid {
                for loc in &issue.locations {
                    for xml in &loc.xml {
                        set.insert(xml.xpath.clone());
                    }
                }
            }
        }
    }
    set
}

/// Count issues for a rule UID (across all checkers).
fn count_issues(result: &Result, rule_uid: &str) -> usize {
    result
        .bundle
        .checkers
        .iter()
        .flat_map(|c| &c.issues)
        .filter(|i| i.rule_uid == rule_uid)
        .count()
}

/// Assert the checker completed, that the issue count is exact, that every
/// expected xpath is present (subset check), and that every issue has the
/// expected severity. Mirrors `check_issues` in test_setup.py.
pub fn check_issues(
    result: &Result,
    rule_uid: &str,
    issue_count: usize,
    expected_xpaths: &[&str],
    severity: IssueSeverity,
    checker_id: &str,
) {
    let status = result.get_checker_status(checker_id);
    assert_eq!(
        status,
        Some(StatusType::Completed),
        "checker {checker_id} should be COMPLETED, got {status:?}"
    );

    let actual = count_issues(result, rule_uid);
    assert_eq!(
        actual, issue_count,
        "rule {rule_uid}: expected {issue_count} issues, got {actual}"
    );

    let xpaths = collect_xpaths(result, rule_uid);
    for xpath in expected_xpaths {
        assert!(
            xpaths.contains(*xpath),
            "rule {rule_uid}: expected xpath not found: {xpath}\nfound: {xpaths:#?}"
        );
    }

    for checker in &result.bundle.checkers {
        for issue in &checker.issues {
            if issue.rule_uid == rule_uid {
                assert_eq!(
                    issue.level, severity,
                    "rule {rule_uid}: expected severity {severity:?}, got {:?}",
                    issue.level
                );
            }
        }
    }
}

/// Assert the checker was skipped and produced no issues. Mirrors `check_skipped`.
pub fn check_skipped(result: &Result, rule_uid: &str, checker_id: &str) {
    let status = result.get_checker_status(checker_id);
    assert_eq!(
        status,
        Some(StatusType::Skipped),
        "checker {checker_id} should be SKIPPED, got {status:?}"
    );
    assert_eq!(
        count_issues(result, rule_uid),
        0,
        "rule {rule_uid}: skipped checker should have 0 issues"
    );
}
