// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Schema validation checker, ported from qc_opendrive/checks/schema/valid_schema.py.
//
// Uses the `xsd-schema` crate for both XSD 1.0 (OpenDRIVE 1.4-1.7) and XSD 1.1
// (OpenDRIVE 1.8).
use std::path::PathBuf;

use xsd_schema::validation::{CollectingValidationSink, SchemaValidator, ValidationFlags, drive_navigator};
use xsd_schema::{RoXmlNavigator, SchemaSetBuilder};

use crate::opendrive::checks::CheckerData;
use crate::opendrive::schema_files;
use crate::result::StatusType;

pub static VALID_SCHEMA: crate::opendrive::checks::CheckerSpec = crate::opendrive::checks::CheckerSpec {
    id: "check_asam_xodr_xml_valid_schema",
    description: "Input xml file must be valid according to the schema.",
    rule_uid: "asam.net:xodr:1.0.0:xml.valid_schema",
    preconditions: &[
        "check_asam_xodr_xml_valid_xml_document",
        "check_asam_xodr_xml_root_tag_is_opendrive",
        "check_asam_xodr_xml_fileheader_is_present",
        "check_asam_xodr_xml_version_is_defined",
    ],
    applicable_versions: None,
    version_required: false,
    run: valid_schema_run,
};

fn schema_dir() -> PathBuf {
    // schemas/ lives next to the binary's crate root at runtime; resolve relative
    // to the current working directory (the tool is run from the project root).
    PathBuf::from("schemas")
}

fn valid_schema_run(cd: &mut CheckerData) {
    let schema_version = match &cd.schema_version {
        Some(v) => v.clone(),
        None => {
            cd.result.set_checker_status("check_asam_xodr_xml_valid_schema", StatusType::Skipped);
            cd.result.add_checker_summary(
                "check_asam_xodr_xml_valid_schema",
                "Cannot determine schema version.",
            );
            return;
        }
    };

    let rel = match schema_files().get(schema_version.as_str()) {
        Some(p) => *p,
        None => {
            cd.result.set_checker_status("check_asam_xodr_xml_valid_schema", StatusType::Skipped);
            cd.result.add_checker_summary(
                "check_asam_xodr_xml_valid_schema",
                &format!(
                    "Cannot find the schema file for ASAM OpenDrive version {}.",
                    schema_version
                ),
            );
            return;
        }
    };

    let xsd_path = schema_dir().join(rel);
    let xsd_text = match std::fs::read_to_string(&xsd_path) {
        Ok(t) => t,
        Err(e) => {
            cd.result.set_checker_status("check_asam_xodr_xml_valid_schema", StatusType::Error);
            cd.result.add_checker_summary(
                "check_asam_xodr_xml_valid_schema",
                &format!("Cannot read schema file {:?}: {}", xsd_path, e),
            );
            return;
        }
    };

    // Decide XSD 1.1 vs 1.0 based on version (>=1.8 => XSD 1.1)
    let parts: Vec<&str> = schema_version.split('.').collect();
    let major: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let is_xsd11 = major >= 1 && minor >= 8;

    let compiled = if is_xsd11 {
        SchemaSetBuilder::xsd11()
    } else {
        SchemaSetBuilder::new()
    }
    .add_source(&xsd_text, xsd_path.to_str().unwrap())
    .and_then(|b| b.compile());

    let compiled = match compiled {
        Ok(c) => c,
        Err(e) => {
            cd.result.set_checker_status("check_asam_xodr_xml_valid_schema", StatusType::Error);
            cd.result.add_checker_summary(
                "check_asam_xodr_xml_valid_schema",
                &format!("Failed to compile schema: {}", e),
            );
            return;
        }
    };

    let validator = SchemaValidator::new(&compiled.schema_set, ValidationFlags::default());
    let mut errors: Vec<xsd_schema::validation::ValidationError> = Vec::new();
    let mut warnings: Vec<xsd_schema::validation::ValidationWarning> = Vec::new();
    let sink = CollectingValidationSink {
        errors: &mut errors,
        warnings: &mut warnings,
    };
    let mut runtime = validator.start_run(sink);

    let xml_text = match std::fs::read(cd.xml_file_path) {
        Ok(t) => t,
        Err(e) => {
            cd.result.set_checker_status("check_asam_xodr_xml_valid_schema", StatusType::Error);
            cd.result.add_checker_summary(
                "check_asam_xodr_xml_valid_schema",
                &format!("Cannot read input file: {}", e),
            );
            return;
        }
    };

    let xml_str = match std::str::from_utf8(&xml_text) {
        Ok(s) => s,
        Err(e) => {
            cd.result.set_checker_status("check_asam_xodr_xml_valid_schema", StatusType::Error);
            cd.result.add_checker_summary(
                "check_asam_xodr_xml_valid_schema",
                &format!("Input is not valid UTF-8: {}", e),
            );
            return;
        }
    };

    let doc = match roxmltree::Document::parse(xml_str) {
        Ok(d) => d,
        Err(e) => {
            cd.result.set_checker_status("check_asam_xodr_xml_valid_schema", StatusType::Error);
            cd.result.add_checker_summary(
                "check_asam_xodr_xml_valid_schema",
                &format!("Cannot parse input XML: {}", e),
            );
            return;
        }
    };

    let nav = RoXmlNavigator::new(&doc);
    if let Err(e) = drive_navigator(&nav, &mut runtime, &compiled.schema_set) {
        cd.result.set_checker_status("check_asam_xodr_xml_valid_schema", StatusType::Error);
        cd.result.add_checker_summary(
            "check_asam_xodr_xml_valid_schema",
            &format!("Validation driver error: {:?}", e),
        );
        return;
    }

    // Resolve element path -> line/col via roxmltree (engine doesn't give coords).
    for err in &errors {
        let (row, col) = match &err.element_path {
            Some(path) => resolve_path(&doc, path),
            None => (None, None),
        };
        let id = cd.result.register_issue(
            "check_asam_xodr_xml_valid_schema",
            "Issue flagging when input file does not follow its version schema",
            crate::result::IssueSeverity::Error,
            "asam.net:xodr:1.0.0:xml.valid_schema",
        );
        cd.result.add_file_location(
            "check_asam_xodr_xml_valid_schema",
            id,
            row,
            col,
            None,
            &err.message,
        );
    }
}

/// Resolve an xsd-schema element path (e.g. "/OpenDRIVE/road[1]") to line/col.
fn resolve_path(doc: &roxmltree::Document, path: &str) -> (Option<u32>, Option<u32>) {
    let mut node = doc.root_element();
    let mut segments = path.split('/').filter(|s| !s.is_empty());
    if let Some(first) = segments.next() {
        let tag = first.split('[').next().unwrap_or(first);
        if node.tag_name().name() != tag {
            return (None, None);
        }
    }
    for seg in segments {
        let (tag, idx) = match seg.find('[') {
            Some(i) => (
                &seg[..i],
                seg[i + 1..seg.len() - 1].parse::<usize>().unwrap_or(1),
            ),
            None => (seg, 1),
        };
        let mut count = 0;
        let mut found = None;
        let mut child = node.first_element_child();
        while let Some(c) = child {
            if c.tag_name().name() == tag {
                count += 1;
                if count == idx {
                    found = Some(c);
                    break;
                }
            }
            child = c.next_sibling_element();
        }
        node = match found {
            Some(n) => n,
            None => return (None, None),
        };
    }
    let tp = doc.text_pos_at(node.range().start);
    (Some(tp.row as u32), Some(tp.col as u32))
}
