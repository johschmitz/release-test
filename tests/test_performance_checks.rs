// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Port of qc-opendrive/tests/test_performance_checks.py.

mod common;

use xodr_qcr::result::IssueSeverity;

const CHECKER: &str = "check_asam_xodr_performance_avoid_redundant_info";
const RULE: &str = "asam.net:xodr:1.7.0:performance.avoid_redundant_info";

#[test]
fn test_performance_avoid_redundant_info() {
    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_elevation_valid.xodr");
    common::check_issues(&r, RULE, 0, &[], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_elevation_invalid_1.xodr");
    common::check_issues(&r, RULE, 1, &["/OpenDRIVE/road/elevationProfile/elevation[1]", "/OpenDRIVE/road/elevationProfile/elevation[2]"], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_elevation_invalid_2.xodr");
    common::check_issues(&r, RULE, 1, &["/OpenDRIVE/road/elevationProfile/elevation[1]", "/OpenDRIVE/road/elevationProfile/elevation[2]"], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_superelevation_valid.xodr");
    common::check_issues(&r, RULE, 0, &[], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_superelevation_invalid.xodr");
    common::check_issues(&r, RULE, 1, &["/OpenDRIVE/road/lateralProfile/superelevation[2]", "/OpenDRIVE/road/lateralProfile/superelevation[3]"], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_lane_offset_valid.xodr");
    common::check_issues(&r, RULE, 0, &[], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_lane_offset_invalid_1.xodr");
    common::check_issues(&r, RULE, 1, &["/OpenDRIVE/road/lanes/laneOffset[2]", "/OpenDRIVE/road/lanes/laneOffset[3]"], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_lane_offset_invalid_2.xodr");
    common::check_issues(&r, RULE, 1, &["/OpenDRIVE/road/lanes/laneOffset[2]", "/OpenDRIVE/road/lanes/laneOffset[3]"], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_lane_width_valid.xodr");
    common::check_issues(&r, RULE, 0, &[], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_lane_width_invalid.xodr");
    common::check_issues(&r, RULE, 1, &["/OpenDRIVE/road/lanes/laneSection[1]/left/lane[7]/width[1]", "/OpenDRIVE/road/lanes/laneSection[1]/left/lane[7]/width[2]"], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_lane_border_valid.xodr");
    common::check_issues(&r, RULE, 0, &[], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_lane_border_invalid.xodr");
    common::check_issues(&r, RULE, 1, &["/OpenDRIVE/road/lanes/laneSection[2]/right/lane[2]/border[1]", "/OpenDRIVE/road/lanes/laneSection[2]/right/lane[2]/border[2]"], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_line_geometry_valid.xodr");
    common::check_issues(&r, RULE, 0, &[], IssueSeverity::Warning, CHECKER);

    let r = common::run_on("tests/data/performance_avoid_redundant_info/performance_avoid_redundant_info_line_geometry_invalid.xodr");
    common::check_issues(&r, RULE, 1, &["/OpenDRIVE/road/planView/geometry[2]", "/OpenDRIVE/road/planView/geometry[3]"], IssueSeverity::Warning, CHECKER);
}
