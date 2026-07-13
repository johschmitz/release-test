// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Checker framework: CheckerData, CheckerSpec, precondition + version gating,
// and the run_checks orchestrator. Ports qc_opendrive/main.py.

use std::path::Path;

use roxmltree::{Document, Node};

use crate::opendrive::models::Point3D;
use crate::opendrive::utils::{node_row, node_xpath};
use crate::opendrive::utils;
use crate::result::{IssueSeverity, Result, StatusType};
use crate::version::{self, RuleUid};

pub mod basic;
pub mod schema;
pub mod semantic;
pub mod geometry;
pub mod performance;
pub mod smoothness;

/// Register an issue on `checker_id` with optional xml (xpath), file (row),
/// and inertial locations. Mirrors the Python pattern of
/// `register_issue` + `add_xml_location` + `add_file_location` +
/// `add_inertial_location`.
pub fn add_issue(
    cd: &mut CheckerData,
    checker_id: &str,
    rule_uid: &str,
    description: &str,
    level: IssueSeverity,
    xml_node: Option<Node>,
    file_node: Option<Node>,
    inertial: Option<Point3D>,
) {
    let issue_id = cd
        .result
        .register_issue(checker_id, description, level, rule_uid);
    if let Some(node) = xml_node {
        let xpath = node_xpath(node);
        cd.result
            .add_xml_location(checker_id, issue_id, &xpath, description);
    }
    if let Some(node) = file_node {
        let row = cd.doc.and_then(|d| node_row(d, node));
        cd.result
            .add_file_location(checker_id, issue_id, row, Some(0), None, description);
    }
    if let Some(p) = inertial {
        cd.result.add_inertial_location(
            checker_id, issue_id, p.x, p.y, p.z, description,
        );
    }
}

/// Append an xml+file location pair to an already-registered issue.
pub fn add_location_pair(
    cd: &mut CheckerData,
    checker_id: &str,
    issue_id: u32,
    description: &str,
    xml_node: Node,
    file_node: Node,
) {
    let xpath = node_xpath(xml_node);
    cd.result
        .add_xml_location(checker_id, issue_id, &xpath, description);
    let row = cd.doc.and_then(|d| node_row(d, file_node));
    cd.result
        .add_file_location(checker_id, issue_id, row, Some(0), None, description);
}

/// Data passed to every checker.
pub struct CheckerData<'d> {
    pub xml_file_path: &'d Path,
    pub doc: Option<&'d Document<'d>>,
    pub schema_version: Option<String>,
    pub result: &'d mut Result,
}

/// A checker specification (mirrors a Python checker module).
pub struct CheckerSpec {
    pub id: &'static str,
    pub description: &'static str,
    pub rule_uid: &'static str,
    pub preconditions: &'static [&'static str],
    pub applicable_versions: Option<&'static str>,
    pub version_required: bool,
    pub run: fn(&mut CheckerData),
}

/// Common precondition set (mirrors basic_preconditions.CHECKER_PRECONDITIONS).
pub const BASIC_PRECONDITIONS: &[&str] = &[
    "check_asam_xodr_xml_valid_xml_document",
    "check_asam_xodr_xml_root_tag_is_opendrive",
    "check_asam_xodr_xml_fileheader_is_present",
    "check_asam_xodr_xml_version_is_defined",
    "check_asam_xodr_xml_valid_schema",
];

fn check_preconditions(cd: &mut CheckerData, spec: &CheckerSpec) -> bool {
    let satisfied = cd
        .result
        .all_checkers_completed_without_issue(spec.preconditions);
    if !satisfied {
        cd.result.set_checker_status(spec.id, StatusType::Skipped);
        cd.result.add_checker_summary(
            spec.id,
            "Preconditions are not satisfied. Skip the check.",
        );
        return false;
    }
    true
}

fn check_version(cd: &mut CheckerData, spec: &CheckerSpec) -> bool {
    let schema_version = match &cd.schema_version {
        Some(v) => v.clone(),
        None => String::new(),
    };

    let rule = match RuleUid::parse(spec.rule_uid) {
        Ok(r) => r,
        Err(e) => {
            cd.result.set_checker_status(spec.id, StatusType::Error);
            cd.result.add_checker_summary(
                spec.id,
                &format!("Invalid rule uid {}: {}.", spec.rule_uid, e),
            );
            return false;
        }
    };
    let definition_setting_expr = format!(">={}", rule.definition_setting);

    let applicable_version = spec.applicable_versions.unwrap_or("");

    // applicable version validity
    if !version::is_valid_version_expression(applicable_version) {
        cd.result.set_checker_status(spec.id, StatusType::Error);
        cd.result.add_checker_summary(
            spec.id,
            &format!(
                "The applicable version {} is not valid. Skip the check.",
                applicable_version
            ),
        );
        return false;
    }

    // definition setting validity
    if !version::is_valid_version_expression(&definition_setting_expr) {
        cd.result.set_checker_status(spec.id, StatusType::Error);
        cd.result.add_checker_summary(
            spec.id,
            &format!(
                "The definition setting {} is not valid. Skip the check.",
                rule.definition_setting
            ),
        );
        return false;
    }

    // First, check applicable version
    if !version::matches(&schema_version, applicable_version) {
        cd.result.set_checker_status(spec.id, StatusType::Skipped);
        cd.result.add_checker_summary(
            spec.id,
            &format!(
                "Version {} is not valid according to the applicable version {}. Skip the check.",
                schema_version, applicable_version
            ),
        );
        return false;
    }

    // definition setting if no lower bound in applicable version
    if !version::has_lower_bound(applicable_version) {
        if !version::matches(&schema_version, &definition_setting_expr) {
            cd.result.set_checker_status(spec.id, StatusType::Skipped);
            cd.result.add_checker_summary(
                spec.id,
                &format!(
                    "Version {} is not valid according to definition setting {}. Skip the check.",
                    schema_version, definition_setting_expr
                ),
            );
            return false;
        }
    }

    true
}

fn execute_checker(cd: &mut CheckerData, spec: &CheckerSpec) {
    cd.result.register_checker(spec.id, spec.description);
    cd.result.register_rule_by_uid(spec.id, spec.rule_uid);

    if !check_preconditions(cd, spec) {
        return;
    }

    if spec.version_required {
        if !check_version(cd, spec) {
            return;
        }
    }

    (spec.run)(cd);

    if cd.result.get_checker_status(spec.id) != Some(StatusType::Skipped) {
        cd.result.set_checker_status(spec.id, StatusType::Completed);
    }
}

/// Run the full checker suite in the order defined by qc_opendrive/main.py.
pub fn run_checks(cd: &mut CheckerData) {
    // 1. Basic checks
    execute_checker(cd, &basic::VALID_XML_DOCUMENT);
    execute_checker(cd, &basic::ROOT_TAG_IS_OPENDRIVE);
    execute_checker(cd, &basic::FILEHEADER_IS_PRESENT);
    execute_checker(cd, &basic::VERSION_IS_DEFINED);

    // Read schema version if the 4 basic checks passed
    if cd.result.all_checkers_completed_without_issue(&[
        "check_asam_xodr_xml_valid_xml_document",
        "check_asam_xodr_xml_root_tag_is_opendrive",
        "check_asam_xodr_xml_fileheader_is_present",
        "check_asam_xodr_xml_version_is_defined",
    ]) {
        if let Some(doc) = cd.doc {
            cd.schema_version = utils::get_standard_schema_version(doc);
        }
    }

    // 2. Schema check
    execute_checker(cd, &schema::VALID_SCHEMA);

    // 3. Semantic / geometry / performance / smoothness checks.
    for spec in semantic::specs() {
        execute_checker(cd, &spec);
    }
    for spec in geometry::specs() {
        execute_checker(cd, &spec);
    }
    for spec in performance::specs() {
        execute_checker(cd, &spec);
    }
    for spec in smoothness::specs() {
        execute_checker(cd, &spec);
    }
}
