// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Port of qc-opendrive/tests/test_basic_checks.py.

mod common;

use xodr_qcr::result::{IssueSeverity, StatusType};

const CHECKER_VALID_XML: &str = "check_asam_xodr_xml_valid_xml_document";
const CHECKER_ROOT: &str = "check_asam_xodr_xml_root_tag_is_opendrive";
const CHECKER_FILEHEADER: &str = "check_asam_xodr_xml_fileheader_is_present";
const CHECKER_VERSION: &str = "check_asam_xodr_xml_version_is_defined";

const RULE_VALID_XML: &str = "asam.net:xodr:1.0.0:xml.valid_xml_document";
const RULE_ROOT: &str = "asam.net:xodr:1.0.0:xml.root_tag_is_opendrive";
const RULE_FILEHEADER: &str = "asam.net:xodr:1.0.0:xml.fileheader_is_present";
const RULE_VERSION: &str = "asam.net:xodr:1.0.0:xml.version_is_defined";

#[test]
fn test_valid_xml_document_positive() {
    let r = common::run_on("tests/data/valid_xml_document/xml.valid_xml_document.positive.xodr");
    assert_eq!(r.get_checker_status(CHECKER_VALID_XML), Some(StatusType::Completed));
    assert_eq!(r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE_VALID_XML).count(), 0);
}

#[test]
fn test_valid_xml_document_negative() {
    let r = common::run_on("tests/data/valid_xml_document/xml.valid_xml_document.negative.xodr");
    assert_eq!(r.get_checker_status(CHECKER_VALID_XML), Some(StatusType::Completed));
    let issues: Vec<_> = r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE_VALID_XML).collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].level, IssueSeverity::Error);
}

#[test]
fn test_root_tag_is_opendrive_positive() {
    let r = common::run_on("tests/data/root_tag_is_opendrive/positive.xodr");
    assert_eq!(r.get_checker_status(CHECKER_ROOT), Some(StatusType::Completed));
    assert_eq!(r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE_ROOT).count(), 0);
}

#[test]
fn test_root_tag_is_opendrive_negative() {
    let r = common::run_on("tests/data/root_tag_is_opendrive/negative.xodr");
    assert_eq!(r.get_checker_status(CHECKER_ROOT), Some(StatusType::Completed));
    let issues: Vec<_> = r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE_ROOT).collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].level, IssueSeverity::Error);
}

#[test]
fn test_fileheader_is_present_positive() {
    let r = common::run_on("tests/data/fileheader_is_present/positive.xodr");
    assert_eq!(r.get_checker_status(CHECKER_FILEHEADER), Some(StatusType::Completed));
    assert_eq!(r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE_FILEHEADER).count(), 0);
}

#[test]
fn test_fileheader_is_present_negative() {
    let r = common::run_on("tests/data/fileheader_is_present/negative.xodr");
    assert_eq!(r.get_checker_status(CHECKER_FILEHEADER), Some(StatusType::Completed));
    let issues: Vec<_> = r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE_FILEHEADER).collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].level, IssueSeverity::Error);
}

#[test]
fn test_version_is_defined_positive() {
    let r = common::run_on("tests/data/version_is_defined/positive.xodr");
    assert_eq!(r.get_checker_status(CHECKER_VERSION), Some(StatusType::Completed));
    assert_eq!(r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE_VERSION).count(), 0);
}

#[test]
fn test_version_is_defined_negative_attr() {
    let r = common::run_on("tests/data/version_is_defined/negative_no_attr.xodr");
    assert_eq!(r.get_checker_status(CHECKER_VERSION), Some(StatusType::Completed));
    let issues: Vec<_> = r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE_VERSION).collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].level, IssueSeverity::Error);
}

#[test]
fn test_version_is_defined_negative_type() {
    let r = common::run_on("tests/data/version_is_defined/negative_no_type.xodr");
    assert_eq!(r.get_checker_status(CHECKER_VERSION), Some(StatusType::Completed));
    let issues: Vec<_> = r.bundle.checkers.iter().flat_map(|c| &c.issues).filter(|i| i.rule_uid == RULE_VERSION).collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].level, IssueSeverity::Error);
}
