// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Basic checkers, ported from qc_opendrive/checks/basic/*.py.

use crate::opendrive::checks::CheckerData;
use crate::opendrive::utils;
use crate::result::StatusType;

pub static VALID_XML_DOCUMENT: crate::opendrive::checks::CheckerSpec = crate::opendrive::checks::CheckerSpec {
    id: "check_asam_xodr_xml_valid_xml_document",
    description: "The input file must be a valid XML document.",
    rule_uid: "asam.net:xodr:1.0.0:xml.valid_xml_document",
    preconditions: &[],
    applicable_versions: None,
    version_required: false,
    run: valid_xml_document_run,
};

pub static ROOT_TAG_IS_OPENDRIVE: crate::opendrive::checks::CheckerSpec = crate::opendrive::checks::CheckerSpec {
    id: "check_asam_xodr_xml_root_tag_is_opendrive",
    description: "The root element of a valid XML document must be OpenDRIVE.",
    rule_uid: "asam.net:xodr:1.0.0:xml.root_tag_is_opendrive",
    preconditions: &["check_asam_xodr_xml_valid_xml_document"],
    applicable_versions: None,
    version_required: false,
    run: root_tag_is_opendrive_run,
};

pub static FILEHEADER_IS_PRESENT: crate::opendrive::checks::CheckerSpec = crate::opendrive::checks::CheckerSpec {
    id: "check_asam_xodr_xml_fileheader_is_present",
    description: "Below the root element a tag with FileHeader must be defined.",
    rule_uid: "asam.net:xodr:1.0.0:xml.fileheader_is_present",
    preconditions: &[
        "check_asam_xodr_xml_valid_xml_document",
        "check_asam_xodr_xml_root_tag_is_opendrive",
    ],
    applicable_versions: None,
    version_required: false,
    run: fileheader_is_present_run,
};

pub static VERSION_IS_DEFINED: crate::opendrive::checks::CheckerSpec = crate::opendrive::checks::CheckerSpec {
    id: "check_asam_xodr_xml_version_is_defined",
    description: "The FileHeader tag must have the attributes revMajor and revMinor and of type unsignedShort.",
    rule_uid: "asam.net:xodr:1.0.0:xml.version_is_defined",
    preconditions: &[
        "check_asam_xodr_xml_valid_xml_document",
        "check_asam_xodr_xml_root_tag_is_opendrive",
        "check_asam_xodr_xml_fileheader_is_present",
    ],
    applicable_versions: None,
    version_required: false,
    run: version_is_defined_run,
};

fn is_unsigned_short(value: &str) -> bool {
    match utils::to_int(value) {
        Some(n) => (0..=65535).contains(&n),
        None => false,
    }
}

fn valid_xml_document_run(cd: &mut CheckerData) {
    // XML well-formedness is checked at parse time in main.rs; if we reached
    // here the doc parsed. But to mirror the Python behavior, re-validate the
    // raw bytes and report a file location on failure.
    let text = match std::fs::read(cd.xml_file_path) {
        Ok(t) => t,
        Err(e) => {
            cd.result.set_checker_status(
                "check_asam_xodr_xml_valid_xml_document",
                StatusType::Error,
            );
            cd.result.add_checker_summary(
                "check_asam_xodr_xml_valid_xml_document",
                &format!("Cannot read file: {e}"),
            );
            return;
        }
    };
    match roxmltree::Document::parse(std::str::from_utf8(&text).unwrap_or("")) {
        Ok(_) => {}
        Err(e) => {
            let id = cd.result.register_issue(
                "check_asam_xodr_xml_valid_xml_document",
                "The input file is not a valid xml document.",
                crate::result::IssueSeverity::Error,
                "asam.net:xodr:1.0.0:xml.valid_xml_document",
            );
            cd.result.add_file_location(
                "check_asam_xodr_xml_valid_xml_document",
                id,
                Some(e.pos().row as u32),
                Some(e.pos().col as u32),
                None,
                "Invalid xml file.",
            );
        }
    }
}

fn root_tag_is_opendrive_run(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let root = doc.root_element();
    if root.tag_name().name() != "OpenDRIVE" {
        let xpath = utils::node_xpath(root);
        let row = utils::node_row(doc, root);
        let id = cd.result.register_issue(
            "check_asam_xodr_xml_root_tag_is_opendrive",
            "Issue flagging when root tag is not OpenDRIVE",
            crate::result::IssueSeverity::Error,
            "asam.net:xodr:1.0.0:xml.root_tag_is_opendrive",
        );
        cd.result
            .add_xml_location("check_asam_xodr_xml_root_tag_is_opendrive", id, &xpath, "Root is not OpenDRIVE");
        cd.result.add_file_location(
            "check_asam_xodr_xml_root_tag_is_opendrive",
            id,
            row,
            Some(0),
            None,
            "Root is not OpenDRIVE",
        );
    }
}

fn fileheader_is_present_run(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let root = doc.root_element();
    let header = root.children().find(|n| n.has_tag_name("header"));
    if header.is_none() {
        let xpath = utils::node_xpath(root);
        let row = utils::node_row(doc, root);
        let id = cd.result.register_issue(
            "check_asam_xodr_xml_fileheader_is_present",
            "Issue flagging when no header is found under root element",
            crate::result::IssueSeverity::Error,
            "asam.net:xodr:1.0.0:xml.fileheader_is_present",
        );
        cd.result
            .add_xml_location("check_asam_xodr_xml_fileheader_is_present", id, &xpath, "No child element header");
        cd.result.add_file_location(
            "check_asam_xodr_xml_fileheader_is_present",
            id,
            row,
            Some(0),
            None,
            "No child element header",
        );
    }
}

fn version_is_defined_run(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let root = doc.root_element();
    let header = match root.children().find(|n| n.has_tag_name("header")) {
        Some(h) => h,
        None => {
            // No header -> skip (mirrors Python behavior)
            cd.result
                .set_checker_status("check_asam_xodr_xml_version_is_defined", StatusType::Skipped);
            cd.result.add_checker_summary(
                "check_asam_xodr_xml_version_is_defined",
                "The xml file does not contains the 'header' tag.",
            );
            return;
        }
    };

    let mut is_valid = true;
    if header.attribute("revMajor").is_none() || header.attribute("revMinor").is_none() {
        is_valid = false;
    }
    if is_valid {
        let rm = header.attribute("revMajor").unwrap();
        let rn = header.attribute("revMinor").unwrap();
        if !is_unsigned_short(rm) || !is_unsigned_short(rn) {
            is_valid = false;
        }
    }

    if !is_valid {
        let xpath = utils::node_xpath(header);
        let row = utils::node_row(doc, header);
        let id = cd.result.register_issue(
            "check_asam_xodr_xml_version_is_defined",
            "Issue flagging when revMajor revMinor attribute of header are missing or invalid",
            crate::result::IssueSeverity::Error,
            "asam.net:xodr:1.0.0:xml.version_is_defined",
        );
        cd.result.add_xml_location(
            "check_asam_xodr_xml_version_is_defined",
            id,
            &xpath,
            "Header tag has invalid or missing version info",
        );
        cd.result.add_file_location(
            "check_asam_xodr_xml_version_is_defined",
            id,
            row,
            Some(0),
            None,
            "Header tag has invalid or missing version info",
        );
    }
}
