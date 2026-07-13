// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Port of qc-opendrive/tests/test_schema_checks.py.

mod common;

use xodr_qcr::result::{IssueSeverity, StatusType};

const CHECKER: &str = "check_asam_xodr_xml_valid_schema";
const RULE: &str = "asam.net:xodr:1.0.0:xml.valid_schema";

#[test]
fn test_valid_schema_positive17() {
    let r = common::run_on("tests/data/valid_schema/positive17.xodr");
    assert_eq!(r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE).count(), 0);
}

#[test]
fn test_valid_schema_positive18() {
    let r = common::run_on("tests/data/valid_schema/positive18.xodr");
    assert_eq!(r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE).count(), 0);
}

#[test]
fn test_valid_schema_negative18() {
    let r = common::run_on("tests/data/valid_schema/negative18.xodr");
    let issues: Vec<_> = r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE).collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].level, IssueSeverity::Error);
}

#[test]
fn test_valid_schema_negative16() {
    let r = common::run_on("tests/data/valid_schema/negative16.xodr");
    let issues: Vec<_> = r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE).collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].level, IssueSeverity::Error);
}

#[test]
fn test_valid_schema_negative17() {
    let r = common::run_on("tests/data/valid_schema/negative17.xodr");
    let issues: Vec<_> = r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE).collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].level, IssueSeverity::Error);
}

#[test]
fn test_unsupported_schema_version() {
    let r = common::run_on("tests/data/valid_schema/unsupported_schema.xodr");
    assert_eq!(r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE).count(), 0);
    assert_eq!(r.get_checker_status(CHECKER), Some(StatusType::Skipped));
}
