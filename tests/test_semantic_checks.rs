// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Port of qc-opendrive/tests/test_semantic_checks.py.

mod common;

use xodr_qcr::result::IssueSeverity;

// road.lane.access.no_mix_of_deny_or_allow
const C_ACCESS: &str = "check_asam_xodr_road_lane_access_no_mix_of_deny_or_allow";
const R_ACCESS: &str = "asam.net:xodr:1.7.0:road.lane.access.no_mix_of_deny_or_allow";

#[test]
fn test_road_lane_access_no_mix_of_deny_or_allow_examples() {
    // 17_invalid -> 2 issues, both at access[3]
    let r = common::run_on("tests/data/road_lane_access_no_mix_of_deny_or_allow/road_lane_access_no_mix_of_deny_or_allow_17_invalid.xodr");
    common::check_issues(&r, R_ACCESS, 2, &["/OpenDRIVE/road/lanes/laneSection/left/lane[1]/access[3]"], IssueSeverity::Error, C_ACCESS);

    // 17_valid -> 0
    let r = common::run_on("tests/data/road_lane_access_no_mix_of_deny_or_allow/road_lane_access_no_mix_of_deny_or_allow_17_valid.xodr");
    common::check_issues(&r, R_ACCESS, 0, &[], IssueSeverity::Error, C_ACCESS);

    // 18_invalid -> 1 at access[2]
    let r = common::run_on("tests/data/road_lane_access_no_mix_of_deny_or_allow/road_lane_access_no_mix_of_deny_or_allow_18_invalid.xodr");
    common::check_issues(&r, R_ACCESS, 1, &["/OpenDRIVE/road/lanes/laneSection/left/lane[1]/access[2]"], IssueSeverity::Error, C_ACCESS);

    // 18_valid -> 0
    let r = common::run_on("tests/data/road_lane_access_no_mix_of_deny_or_allow/road_lane_access_no_mix_of_deny_or_allow_18_valid.xodr");
    common::check_issues(&r, R_ACCESS, 0, &[], IssueSeverity::Error, C_ACCESS);
}

#[test]
fn test_road_lane_access_no_mix_of_deny_or_allow_older_schema() {
    let r = common::run_on("tests/data/road_lane_access_no_mix_of_deny_or_allow/road_lane_access_no_mix_of_deny_or_allow_17_invalid_older_schema_version.xodr");
    common::check_skipped(&r, R_ACCESS, C_ACCESS);
}

#[test]
fn test_road_lane_access_no_mix_of_deny_or_allow_close_match() {
    // single_issue -> 1 at access[3]
    let r = common::run_on("tests/data/road_lane_access_no_mix_of_deny_or_allow/close_match_single_issue.xodr");
    common::check_issues(&r, R_ACCESS, 1, &["/OpenDRIVE/road/lanes/laneSection/left/lane[1]/access[3]"], IssueSeverity::Error, C_ACCESS);

    // multiple_issue -> 3
    let r = common::run_on("tests/data/road_lane_access_no_mix_of_deny_or_allow/close_match_multiple_issue.xodr");
    common::check_issues(&r, R_ACCESS, 3, &["/OpenDRIVE/road/lanes/laneSection/left/lane[1]/access[3]", "/OpenDRIVE/road/lanes/laneSection/left/lane[1]/access[4]"], IssueSeverity::Error, C_ACCESS);
}

// road.lane.level_true_one_side
const C_LEVEL: &str = "check_asam_xodr_road_lane_level_true_one_side";
const R_LEVEL: &str = "asam.net:xodr:1.7.0:road.lane.level_true_one_side";

#[test]
fn test_road_lane_true_level_one_side() {
    let r = common::run_on("tests/data/road_lane_level_true_one_side/road_lane_level_true_one_side_valid.xodr");
    common::check_issues(&r, R_LEVEL, 0, &[], IssueSeverity::Error, C_LEVEL);

    let r = common::run_on("tests/data/road_lane_level_true_one_side/road_lane_level_true_one_side_invalid.xodr");
    common::check_issues(&r, R_LEVEL, 2, &["/OpenDRIVE/road/lanes/laneSection/left/lane[1]", "/OpenDRIVE/road/lanes/laneSection/right/lane[3]"], IssueSeverity::Error, C_LEVEL);
}

#[test]
fn test_road_lane_true_level_one_side_older_schema() {
    let r = common::run_on("tests/data/road_lane_level_true_one_side/road_lane_level_true_one_side_invalid_older_schema_version.xodr");
    common::check_skipped(&r, R_LEVEL, C_LEVEL);
}

#[test]
fn test_road_lane_true_level_one_side_lane_section() {
    let r = common::run_on("tests/data/road_lane_level_true_one_side_lanesection/road_lane_level_true_one_side_lanesection_valid.xodr");
    common::check_issues(&r, R_LEVEL, 0, &[], IssueSeverity::Warning, C_LEVEL);

    let r = common::run_on("tests/data/road_lane_level_true_one_side_lanesection/road_lane_level_true_one_side_lanesection_invalid.xodr");
    common::check_issues(&r, R_LEVEL, 8, &["/OpenDRIVE/road/lanes/laneSection[1]/left/lane[1]", "/OpenDRIVE/road/lanes/laneSection[1]/left/lane[2]", "/OpenDRIVE/road/lanes/laneSection[2]/left/lane[1]", "/OpenDRIVE/road/lanes/laneSection[2]/left/lane[2]", "/OpenDRIVE/road/lanes/laneSection[1]/right/lane[2]", "/OpenDRIVE/road/lanes/laneSection[1]/right/lane[3]", "/OpenDRIVE/road/lanes/laneSection[2]/right/lane[2]", "/OpenDRIVE/road/lanes/laneSection[2]/right/lane[3]"], IssueSeverity::Warning, C_LEVEL);

    let r = common::run_on("tests/data/road_lane_level_true_one_side_lanesection/road_lane_level_true_one_side_lanesection_valid_wrong_predecessor.xodr");
    common::check_issues(&r, R_LEVEL, 0, &[], IssueSeverity::Warning, C_LEVEL);

    let r = common::run_on("tests/data/road_lane_level_true_one_side_lanesection/road_lane_level_true_one_side_lanesection_invalid_wrong_predecessor.xodr");
    common::check_issues(&r, R_LEVEL, 2, &["/OpenDRIVE/road/lanes/laneSection[1]/left/lane[1]", "/OpenDRIVE/road/lanes/laneSection[1]/left/lane[2]"], IssueSeverity::Warning, C_LEVEL);
}

#[test]
fn test_road_lane_true_level_one_side_road() {
    let r = common::run_on("tests/data/road_lane_level_true_one_side_road/road_lane_level_true_one_side_road_valid.xodr");
    common::check_issues(&r, R_LEVEL, 0, &[], IssueSeverity::Warning, C_LEVEL);

    let r = common::run_on("tests/data/road_lane_level_true_one_side_road/road_lane_level_true_one_side_road_invalid.xodr");
    common::check_issues(&r, R_LEVEL, 4, &["/OpenDRIVE/road[1]/lanes/laneSection[1]/left/lane[1]", "/OpenDRIVE/road[1]/lanes/laneSection[2]/left/lane[1]", "/OpenDRIVE/road[1]/lanes/laneSection[2]/left/lane[1]", "/OpenDRIVE/road[2]/lanes/laneSection/left/lane[1]"], IssueSeverity::Warning, C_LEVEL);
}

#[test]
fn test_road_lane_true_level_one_side_junction() {
    let r = common::run_on("tests/data/road_lane_level_true_one_side_junction/road_lane_level_true_one_side_junction_valid.xodr");
    common::check_issues(&r, R_LEVEL, 0, &[], IssueSeverity::Warning, C_LEVEL);

    let r = common::run_on("tests/data/road_lane_level_true_one_side_junction/road_lane_level_true_one_side_junction_invalid_incoming.xodr");
    common::check_issues(&r, R_LEVEL, 2, &["/OpenDRIVE/road[1]/lanes/laneSection/right/lane", "/OpenDRIVE/road[2]/lanes/laneSection/right/lane"], IssueSeverity::Warning, C_LEVEL);
}

// road.lane.link.lanes_across_lane_sections
const C_ACROSS: &str = "check_asam_xodr_road_lane_link_lanes_across_lane_sections";
const R_ACROSS: &str = "asam.net:xodr:1.4.0:road.lane.link.lanes_across_lane_sections";

#[test]
fn test_road_lane_link_lanes_across_lane_sections() {
    let r = common::run_on("tests/data/road_lane_link_lanes_across_lane_sections/road_lane_link_lanes_across_lane_sections_valid.xodr");
    common::check_issues(&r, R_ACROSS, 0, &[], IssueSeverity::Error, C_ACROSS);

    let r = common::run_on("tests/data/road_lane_link_lanes_across_lane_sections/road_lane_link_lanes_across_lane_sections_invalid_no_predecessor_road.xodr");
    common::check_issues(&r, R_ACROSS, 3, &["/OpenDRIVE/road/lanes/laneSection[1]/right/lane[1]", "/OpenDRIVE/road/lanes/laneSection[1]/right/lane[2]", "/OpenDRIVE/road/lanes/laneSection[1]/right/lane[3]"], IssueSeverity::Error, C_ACROSS);

    let r = common::run_on("tests/data/road_lane_link_lanes_across_lane_sections/road_lane_link_lanes_across_lane_sections_invalid_non_existing_lanes.xodr");
    common::check_issues(&r, R_ACROSS, 4, &["/OpenDRIVE/road/lanes/laneSection[2]/left/lane[2]", "/OpenDRIVE/road/lanes/laneSection[2]/right/lane[2]", "/OpenDRIVE/road/lanes/laneSection[2]/left/lane[1]", "/OpenDRIVE/road/lanes/laneSection[2]/right/lane[3]"], IssueSeverity::Error, C_ACROSS);

    let r = common::run_on("tests/data/road_lane_link_lanes_across_lane_sections/road_lane_link_lanes_across_lane_sections_invalid_wrong_id.xodr");
    common::check_issues(&r, R_ACROSS, 6, &["/OpenDRIVE/road/lanes/laneSection[1]/left/lane[2]", "/OpenDRIVE/road/lanes/laneSection[1]/right/lane[2]", "/OpenDRIVE/road/lanes/laneSection[1]/right/lane[3]", "/OpenDRIVE/road/lanes/laneSection[2]/left/lane[1]", "/OpenDRIVE/road/lanes/laneSection[2]/left/lane[2]", "/OpenDRIVE/road/lanes/laneSection[2]/right/lane[2]"], IssueSeverity::Error, C_ACROSS);
}

// road.linkage.is_junction_needed
const C_JUNCTION_NEEDED: &str = "check_asam_xodr_road_linkage_is_junction_needed";
const R_JUNCTION_NEEDED: &str = "asam.net:xodr:1.4.0:road.linkage.is_junction_needed";

#[test]
fn test_road_linkage_is_junction_needed() {
    let r = common::run_on("tests/data/road_linkage_is_junction_needed/road_linkage_is_junction_needed_valid.xodr");
    common::check_issues(&r, R_JUNCTION_NEEDED, 0, &[], IssueSeverity::Error, C_JUNCTION_NEEDED);

    let r = common::run_on("tests/data/road_linkage_is_junction_needed/road_linkage_is_junction_needed_invalid.xodr");
    common::check_issues(&r, R_JUNCTION_NEEDED, 1, &["/OpenDRIVE/road[2]/link/predecessor", "/OpenDRIVE/road[3]/link/predecessor"], IssueSeverity::Error, C_JUNCTION_NEEDED);

    let r = common::run_on("tests/data/road_linkage_is_junction_needed/road_linkage_is_junction_needed_invalid_converge.xodr");
    common::check_issues(&r, R_JUNCTION_NEEDED, 1, &["/OpenDRIVE/road[2]/link/successor", "/OpenDRIVE/road[3]/link/successor"], IssueSeverity::Error, C_JUNCTION_NEEDED);
}

// junctions.connection.connect_road_no_incoming_road
const C_NO_INCOMING: &str = "check_asam_xodr_junctions_connection_connect_road_no_incoming_road";
const R_NO_INCOMING: &str = "asam.net:xodr:1.4.0:junctions.connection.connect_road_no_incoming_road";

#[test]
fn test_junctions_connection_road_no_incoming_road() {
    let r = common::run_on("tests/data/junctions_connection_connect_road_no_incoming_road/junctions_connection_connect_road_no_incoming_road_valid.xodr");
    common::check_issues(&r, R_NO_INCOMING, 0, &[], IssueSeverity::Error, C_NO_INCOMING);

    let r = common::run_on("tests/data/junctions_connection_connect_road_no_incoming_road/junctions_connection_connect_road_no_incoming_road_invalid.xodr");
    common::check_issues(&r, R_NO_INCOMING, 2, &["/OpenDRIVE/junction[2]/connection[1]", "/OpenDRIVE/junction[2]/connection[2]"], IssueSeverity::Error, C_NO_INCOMING);
}

// junctions.connection.one_connection_element
const C_ONE_CONN: &str = "check_asam_xodr_junctions_connection_one_connection_element";
const R_ONE_CONN: &str = "asam.net:xodr:1.7.0:junctions.connection.one_connection_element";

#[test]
fn test_junctions_connection_one_connection_element() {
    let r = common::run_on("tests/data/junctions_connection_one_connection_element/junctions_connection_one_connection_element_valid.xodr");
    common::check_issues(&r, R_ONE_CONN, 0, &[], IssueSeverity::Error, C_ONE_CONN);

    let r = common::run_on("tests/data/junctions_connection_one_connection_element/junctions_connection_one_connection_element_invalid.xodr");
    common::check_issues(&r, R_ONE_CONN, 1, &["/OpenDRIVE/junction/connection[1]", "/OpenDRIVE/junction/connection[2]"], IssueSeverity::Error, C_ONE_CONN);
}

#[test]
fn test_junctions_connection_one_connection_element_applicable_version() {
    // v1_8_0_valid -> skipped (applicable <=1.7.0)
    let r = common::run_on("tests/data/junctions_connection_one_connection_element/junctions_connection_one_connection_element_v1_8_0_valid.xodr");
    common::check_skipped(&r, R_ONE_CONN, C_ONE_CONN);

    // v1_6_0_skipped -> skipped
    let r = common::run_on("tests/data/junctions_connection_one_connection_element/junctions_connection_one_connection_element_v1_6_0_skipped.xodr");
    common::check_skipped(&r, R_ONE_CONN, C_ONE_CONN);
}

// junctions.connection.one_link_to_incoming
const C_ONE_LINK: &str = "check_asam_xodr_junctions_connection_one_link_to_incoming";
const R_ONE_LINK: &str = "asam.net:xodr:1.8.0:junctions.connection.one_link_to_incoming";

#[test]
fn test_junctions_connection_one_link_to_incoming() {
    let r = common::run_on("tests/data/junctions_connection_one_link_to_incoming/junctions_connection_one_link_to_incoming_valid.xodr");
    common::check_issues(&r, R_ONE_LINK, 0, &[], IssueSeverity::Error, C_ONE_LINK);

    let r = common::run_on("tests/data/junctions_connection_one_link_to_incoming/junctions_connection_one_link_to_incoming_invalid.xodr");
    common::check_issues(&r, R_ONE_LINK, 2, &["/OpenDRIVE/junction/connection[1]", "/OpenDRIVE/junction/connection[2]", "/OpenDRIVE/junction/connection[2]/laneLink"], IssueSeverity::Error, C_ONE_LINK);

    let r = common::run_on("tests/data/junctions_connection_one_link_to_incoming/junctions_connection_one_link_to_incoming_valid_LHT.xodr");
    common::check_issues(&r, R_ONE_LINK, 0, &[], IssueSeverity::Error, C_ONE_LINK);

    let r = common::run_on("tests/data/junctions_connection_one_link_to_incoming/junctions_connection_one_link_to_incoming_invalid_LHT.xodr");
    common::check_issues(&r, R_ONE_LINK, 2, &["/OpenDRIVE/junction/connection[1]", "/OpenDRIVE/junction/connection[2]", "/OpenDRIVE/junction/connection[2]/laneLink"], IssueSeverity::Error, C_ONE_LINK);
}

#[test]
fn test_junctions_connection_one_link_to_incoming_bidirectional() {
    let r = common::run_on("tests/data/junctions_connection_one_link_to_incoming/Ex_Bidirectional_Junction_valid.xodr");
    common::check_issues(&r, R_ONE_LINK, 0, &[], IssueSeverity::Error, C_ONE_LINK);

    let r = common::run_on("tests/data/junctions_connection_one_link_to_incoming/Ex_Bidirectional_Junction_invalid.xodr");
    common::check_issues(&r, R_ONE_LINK, 1, &["/OpenDRIVE/junction/connection[5]/laneLink"], IssueSeverity::Error, C_ONE_LINK);
}

#[test]
fn test_junctions_connection_one_link_to_incoming_direct() {
    let r = common::run_on("tests/data/examples/Ex_Entry_Exit.xodr");
    common::check_issues(&r, R_ONE_LINK, 0, &[], IssueSeverity::Error, C_ONE_LINK);
}

// road.lane.link.zero_width_at_start
const C_ZW_START: &str = "check_asam_xodr_road_lane_link_zero_width_at_start";
const R_ZW_START: &str = "asam.net:xodr:1.7.0:road.lane.link.zero_width_at_start";

#[test]
fn test_road_lane_link_zero_width_at_start() {
    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_valid.xodr");
    common::check_issues(&r, R_ZW_START, 0, &[], IssueSeverity::Error, C_ZW_START);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_invalid.xodr");
    common::check_issues(&r, R_ZW_START, 1, &["/OpenDRIVE/road/lanes/laneSection[2]/right/lane[2]"], IssueSeverity::Error, C_ZW_START);
}

#[test]
fn test_road_lane_link_zero_width_at_start_junction() {
    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_junction_valid.xodr");
    common::check_issues(&r, R_ZW_START, 0, &[], IssueSeverity::Error, C_ZW_START);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_junction_valid_1.xodr");
    common::check_issues(&r, R_ZW_START, 0, &[], IssueSeverity::Error, C_ZW_START);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_junction_invalid.xodr");
    common::check_issues(&r, R_ZW_START, 1, &["/OpenDRIVE/road[3]/lanes/laneSection/right/lane[2]"], IssueSeverity::Error, C_ZW_START);
}

#[test]
fn test_road_lane_link_zero_width_at_start_inside_junction() {
    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_inside_junction_valid.xodr");
    common::check_issues(&r, R_ZW_START, 0, &[], IssueSeverity::Error, C_ZW_START);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_inside_junction_invalid.xodr");
    common::check_issues(&r, R_ZW_START, 1, &["/OpenDRIVE/road[2]/lanes/laneSection/right/lane[2]"], IssueSeverity::Error, C_ZW_START);
}

// road.lane.link.zero_width_at_end
const C_ZW_END: &str = "check_asam_xodr_road_lane_link_zero_width_at_end";
const R_ZW_END: &str = "asam.net:xodr:1.7.0:road.lane.link.zero_width_at_end";

#[test]
fn test_road_lane_link_zero_width_at_end() {
    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_valid.xodr");
    common::check_issues(&r, R_ZW_END, 0, &[], IssueSeverity::Error, C_ZW_END);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_invalid.xodr");
    common::check_issues(&r, R_ZW_END, 1, &["/OpenDRIVE/road/lanes/laneSection[1]/left/lane[1]"], IssueSeverity::Error, C_ZW_END);
}

#[test]
fn test_road_lane_link_zero_width_at_end_junction() {
    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_junction_valid.xodr");
    common::check_issues(&r, R_ZW_END, 0, &[], IssueSeverity::Error, C_ZW_END);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_junction_valid_1.xodr");
    common::check_issues(&r, R_ZW_END, 0, &[], IssueSeverity::Error, C_ZW_END);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_junction_invalid.xodr");
    common::check_issues(&r, R_ZW_END, 1, &["/OpenDRIVE/road[1]/lanes/laneSection/left/lane[1]"], IssueSeverity::Error, C_ZW_END);
}

#[test]
fn test_road_lane_link_zero_width_at_end_inside_junction() {
    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_inside_junction_valid.xodr");
    common::check_issues(&r, R_ZW_END, 0, &[], IssueSeverity::Error, C_ZW_END);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_inside_junction_invalid.xodr");
    common::check_issues(&r, R_ZW_END, 1, &["/OpenDRIVE/road[2]/lanes/laneSection/left/lane[1]"], IssueSeverity::Error, C_ZW_END);
}

// road.lane.link.new_lane_appear
const C_NEW_LANE: &str = "check_asam_xodr_road_lane_link_new_lane_appear";
const R_NEW_LANE: &str = "asam.net:xodr:1.4.0:road.lane.link.new_lane_appear";

#[test]
fn test_road_lane_link_new_lane_appear() {
    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_valid.xodr");
    common::check_issues(&r, R_NEW_LANE, 0, &[], IssueSeverity::Error, C_NEW_LANE);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_invalid.xodr");
    common::check_issues(&r, R_NEW_LANE, 1, &["/OpenDRIVE/road/lanes/laneSection[1]/right/lane", "/OpenDRIVE/road/lanes/laneSection[2]/right/lane[2]"], IssueSeverity::Error, C_NEW_LANE);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_both_start_contact_point_invalid.xodr");
    common::check_issues(&r, R_NEW_LANE, 1, &["/OpenDRIVE/road[1]/lanes/laneSection/left/lane", "/OpenDRIVE/road[2]/lanes/laneSection/right/lane[2]"], IssueSeverity::Error, C_NEW_LANE);
}

#[test]
fn test_road_lane_link_new_lane_appear_junction() {
    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_junction_valid.xodr");
    common::check_issues(&r, R_NEW_LANE, 0, &[], IssueSeverity::Error, C_NEW_LANE);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_junction_valid_1.xodr");
    common::check_issues(&r, R_NEW_LANE, 0, &[], IssueSeverity::Error, C_NEW_LANE);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_junction_invalid.xodr");
    common::check_issues(&r, R_NEW_LANE, 1, &["/OpenDRIVE/road[2]/lanes/laneSection/right/lane", "/OpenDRIVE/road[3]/lanes/laneSection/right/lane[2]"], IssueSeverity::Error, C_NEW_LANE);
}

#[test]
fn test_road_lane_link_new_lane_appear_inside_junction() {
    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_inside_junction_valid.xodr");
    common::check_issues(&r, R_NEW_LANE, 0, &[], IssueSeverity::Error, C_NEW_LANE);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_inside_junction_valid_2.xodr");
    common::check_issues(&r, R_NEW_LANE, 0, &[], IssueSeverity::Error, C_NEW_LANE);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_inside_junction_invalid.xodr");
    common::check_issues(&r, R_NEW_LANE, 2, &["/OpenDRIVE/road[1]/lanes/laneSection/right/lane", "/OpenDRIVE/road[2]/lanes/laneSection/right/lane[2]", "/OpenDRIVE/road[2]/lanes/laneSection/left/lane[1]", "/OpenDRIVE/road[3]/lanes/laneSection/left/lane"], IssueSeverity::Error, C_NEW_LANE);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_inside_junction_invalid_2.xodr");
    common::check_issues(&r, R_NEW_LANE, 1, &["/OpenDRIVE/road[2]/lanes/laneSection/left/lane[1]", "/OpenDRIVE/road[3]/lanes/laneSection/left/lane"], IssueSeverity::Error, C_NEW_LANE);
}

#[test]
fn test_road_lane_link_new_lane_appear_end_contact_point() {
    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_end_contact_point_valid.xodr");
    common::check_issues(&r, R_NEW_LANE, 0, &[], IssueSeverity::Error, C_NEW_LANE);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_end_contact_point_valid_example.xodr");
    common::check_issues(&r, R_NEW_LANE, 0, &[], IssueSeverity::Error, C_NEW_LANE);

    let r = common::run_on("tests/data/road_lane_link_new_lane_appear/road_lane_link_new_lane_appear_end_contact_point_invalid.xodr");
    common::check_issues(&r, R_NEW_LANE, 1, &["/OpenDRIVE/road[1]/lanes/laneSection/right/lane", "/OpenDRIVE/road[2]/lanes/laneSection/left/lane[2]"], IssueSeverity::Error, C_NEW_LANE);
}

// junctions.connection.start_along_linkage
const C_START_ALONG: &str = "check_asam_xodr_junctions_connection_start_along_linkage";
const R_START_ALONG: &str = "asam.net:xodr:1.7.0:junctions.connection.start_along_linkage";

#[test]
fn test_junction_connection_start_along_linkage() {
    let r = common::run_on("tests/data/junction_connection_linkage/junction_connection_linkage_valid.xodr");
    common::check_issues(&r, R_START_ALONG, 0, &[], IssueSeverity::Error, C_START_ALONG);

    let r = common::run_on("tests/data/junction_connection_linkage/junction_connection_linkage_invalid.xodr");
    common::check_issues(&r, R_START_ALONG, 1, &["/OpenDRIVE/junction/connection[2]"], IssueSeverity::Error, C_START_ALONG);
}

// junctions.connection.end_opposite_linkage
const C_END_OPP: &str = "check_asam_xodr_junctions_connection_end_opposite_linkage";
const R_END_OPP: &str = "asam.net:xodr:1.7.0:junctions.connection.end_opposite_linkage";

#[test]
fn test_junction_connection_end_opposite_linkage() {
    let r = common::run_on("tests/data/junction_connection_linkage/junction_connection_linkage_valid.xodr");
    common::check_issues(&r, R_END_OPP, 0, &[], IssueSeverity::Error, C_END_OPP);

    let r = common::run_on("tests/data/junction_connection_linkage/junction_connection_linkage_invalid.xodr");
    common::check_issues(&r, R_END_OPP, 1, &["/OpenDRIVE/junction/connection[1]"], IssueSeverity::Error, C_END_OPP);
}
