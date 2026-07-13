// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Port of qc-opendrive/tests/test_geometry_checks.py.

mod common;

use xodr_qcr::result::IssueSeverity;

// road.geometry.parampoly3.length_match
const C_LEN_MATCH: &str = "check_asam_xodr_road_geometry_parampoly3_length_match";
const R_LEN_MATCH: &str = "asam.net:xodr:1.7.0:road.geometry.parampoly3.length_match";

#[test]
fn test_road_geometry_param_poly3_length_match() {
    let r = common::run_on("tests/data/road_geometry_param_poly3_length_match/road_geometry_param_poly3_length_match_valid.xodr");
    common::check_issues(&r, R_LEN_MATCH, 0, &[], IssueSeverity::Warning, C_LEN_MATCH);

    let r = common::run_on("tests/data/road_geometry_param_poly3_length_match/road_geometry_param_poly3_length_match_invalid.xodr");
    common::check_issues(&r, R_LEN_MATCH, 1, &["/OpenDRIVE/road/planView/geometry[2]"], IssueSeverity::Warning, C_LEN_MATCH);

    let r = common::run_on("tests/data/road_geometry_param_poly3_length_match/road_geometry_param_poly3_length_match_invalid_multiple_cases.xodr");
    common::check_issues(&r, R_LEN_MATCH, 3, &["/OpenDRIVE/road/planView/geometry[2]", "/OpenDRIVE/road/planView/geometry[3]", "/OpenDRIVE/road/planView/geometry[4]"], IssueSeverity::Warning, C_LEN_MATCH);
}

// road.lane.border.overlap_with_inner_lanes
const C_BORDER: &str = "check_asam_xodr_road_lane_border_overlap_with_inner_lanes";
const R_BORDER: &str = "asam.net:xodr:1.4.0:road.lane.border.overlap_with_inner_lanes";

#[test]
fn test_road_lane_border_overlap_with_inner_lanes() {
    let r = common::run_on("tests/data/road_lane_border_overlap_with_inner_lanes/road_lane_border_overlap_with_inner_lanes_valid.xodr");
    common::check_issues(&r, R_BORDER, 0, &[], IssueSeverity::Error, C_BORDER);

    let r = common::run_on("tests/data/road_lane_border_overlap_with_inner_lanes/road_lane_border_overlap_with_inner_lanes_valid_1.xodr");
    common::check_issues(&r, R_BORDER, 0, &[], IssueSeverity::Error, C_BORDER);

    let r = common::run_on("tests/data/road_lane_border_overlap_with_inner_lanes/road_lane_border_overlap_with_inner_lanes_invalid.xodr");
    common::check_issues(&r, R_BORDER, 2, &["/OpenDRIVE/road/lanes/laneSection/left/lane[1]", "/OpenDRIVE/road/lanes/laneSection/left/lane[2]", "/OpenDRIVE/road/lanes/laneSection/right/lane[1]", "/OpenDRIVE/road/lanes/laneSection/right/lane[2]"], IssueSeverity::Error, C_BORDER);

    let r = common::run_on("tests/data/road_lane_border_overlap_with_inner_lanes/road_lane_border_overlap_with_inner_lanes_invalid_1.xodr");
    common::check_issues(&r, R_BORDER, 1, &["/OpenDRIVE/road/lanes/laneSection/left/lane[1]", "/OpenDRIVE/road/lanes/laneSection/left/lane[2]"], IssueSeverity::Error, C_BORDER);
}

// road.geometry.parampoly3.arclength_range
const C_ARC_RANGE: &str = "check_asam_xodr_road_geometry_parampoly3_arclength_range";
const R_ARC_RANGE: &str = "asam.net:xodr:1.7.0:road.geometry.parampoly3.arclength_range";

#[test]
fn test_road_geometry_parampoly3_arclength_range() {
    let r = common::run_on("tests/data/road_geometry_parampoly3_arclength_range/road_geometry_parampoly3_arclength_range_valid.xodr");
    common::check_issues(&r, R_ARC_RANGE, 0, &[], IssueSeverity::Error, C_ARC_RANGE);

    let r = common::run_on("tests/data/road_geometry_parampoly3_arclength_range/road_geometry_parampoly3_arclength_range_invalid.xodr");
    common::check_issues(&r, R_ARC_RANGE, 3, &["/OpenDRIVE/road/planView/geometry[3]", "/OpenDRIVE/road/planView/geometry[4]", "/OpenDRIVE/road/planView/geometry[6]"], IssueSeverity::Error, C_ARC_RANGE);
}

// road.geometry.parampoly3.normalized_range
const C_NORM_RANGE: &str = "check_asam_xodr_road_geometry_parampoly3_normalized_range";
const R_NORM_RANGE: &str = "asam.net:xodr:1.7.0:road.geometry.parampoly3.normalized_range";

#[test]
fn test_road_geometry_param_poly3_normalized_range() {
    let r = common::run_on("tests/data/road_geometry_param_poly3_length_match/road_geometry_param_poly3_length_match_valid.xodr");
    common::check_issues(&r, R_NORM_RANGE, 0, &[], IssueSeverity::Error, C_NORM_RANGE);

    let r = common::run_on("tests/data/road_geometry_param_poly3_length_match/road_geometry_param_poly3_length_match_invalid.xodr");
    common::check_issues(&r, R_NORM_RANGE, 1, &["/OpenDRIVE/road/planView/geometry[2]"], IssueSeverity::Error, C_NORM_RANGE);
}

// road.geometry.contact_point
const C_CONTACT: &str = "check_asam_xodr_road_geometry_contact_point";
const R_CONTACT: &str = "asam.net:xodr:1.7.0:road.geometry.contact_point";

#[test]
fn test_road_geometry_contact_point() {
    let r = common::run_on("tests/data/road_geometry_contact_point/valid_1.xodr");
    common::check_issues(&r, R_CONTACT, 0, &[], IssueSeverity::Error, C_CONTACT);

    let r = common::run_on("tests/data/road_geometry_contact_point/valid_2.xodr");
    common::check_issues(&r, R_CONTACT, 0, &[], IssueSeverity::Error, C_CONTACT);

    let r = common::run_on("tests/data/road_geometry_contact_point/valid_3.xodr");
    common::check_issues(&r, R_CONTACT, 0, &[], IssueSeverity::Error, C_CONTACT);

    let r = common::run_on("tests/data/road_geometry_contact_point/invalid.xodr");
    common::check_issues(&r, R_CONTACT, 2, &["/OpenDRIVE/road[2]", "/OpenDRIVE/road[1]"], IssueSeverity::Error, C_CONTACT);

    let r = common::run_on("tests/data/road_geometry_contact_point/invalid_2.xodr");
    common::check_issues(&r, R_CONTACT, 1, &["/OpenDRIVE/road[1]"], IssueSeverity::Error, C_CONTACT);
}

// road.geometry.elem_asc_order
const C_ASC: &str = "check_asam_xodr_road_geometry_elem_asc_order";
const R_ASC: &str = "asam.net:xodr:1.4.0:road.geometry.elem_asc_order";

#[test]
fn test_road_geometry_elem_asc_order() {
    let r = common::run_on("tests/data/road_geometry_elem_asc_order/valid.xodr");
    common::check_issues(&r, R_ASC, 0, &[], IssueSeverity::Error, C_ASC);

    let r = common::run_on("tests/data/road_geometry_elem_asc_order/invalid.xodr");
    common::check_issues(&r, R_ASC, 1, &["/OpenDRIVE/road/planView/geometry[3]"], IssueSeverity::Error, C_ASC);
}

// road.geometry.paramPoly3.valid_parameters
const C_VALID_PARAM: &str = "check_asam_xodr_road_geometry_parampoly3_valid_parameters";
const R_VALID_PARAM: &str = "asam.net:xodr:1.7.0:road.geometry.paramPoly3.valid_parameters";

#[test]
fn test_road_geometry_parampoly3_valid_parameters() {
    let r = common::run_on("tests/data/road_geometry_parampoly3_valid_parameters/valid.xodr");
    common::check_issues(&r, R_VALID_PARAM, 0, &[], IssueSeverity::Error, C_VALID_PARAM);

    let r = common::run_on("tests/data/road_geometry_parampoly3_valid_parameters/aU_invalid.xodr");
    common::check_issues(&r, R_VALID_PARAM, 1, &["/OpenDRIVE/road/planView/geometry[1]/paramPoly3"], IssueSeverity::Error, C_VALID_PARAM);

    let r = common::run_on("tests/data/road_geometry_parampoly3_valid_parameters/aV_invalid.xodr");
    common::check_issues(&r, R_VALID_PARAM, 1, &["/OpenDRIVE/road/planView/geometry[1]/paramPoly3"], IssueSeverity::Error, C_VALID_PARAM);

    let r = common::run_on("tests/data/road_geometry_parampoly3_valid_parameters/bU_invalid.xodr");
    common::check_issues(&r, R_VALID_PARAM, 1, &["/OpenDRIVE/road/planView/geometry[2]/paramPoly3"], IssueSeverity::Error, C_VALID_PARAM);

    let r = common::run_on("tests/data/road_geometry_parampoly3_valid_parameters/bV_invalid.xodr");
    common::check_issues(&r, R_VALID_PARAM, 1, &["/OpenDRIVE/road/planView/geometry[2]/paramPoly3"], IssueSeverity::Error, C_VALID_PARAM);
}
