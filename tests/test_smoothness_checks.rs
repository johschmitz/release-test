// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Port of qc-opendrive/tests/test_smoothness_checks.py.

mod common;

use xodr_qcr::result::IssueSeverity;

const CHECKER: &str = "check_asam_xodr_lane_smoothness_contact_point_no_horizontal_gaps";
const RULE: &str = "asam.net:xodr:1.7.0:lane_smoothness.contact_point_no_horizontal_gaps";

#[test]
fn test_lane_smoothness_contact_point_no_horizontal_gaps() {
    let r = common::run_on("tests/data/smoothness_example/many_invalid.xodr");
    common::check_issues(&r, RULE, 24, &[
        "/OpenDRIVE/road[1]/lanes/laneSection/left/lane[1]",
        "/OpenDRIVE/road[2]/lanes/laneSection/right/lane[3]",
        "/OpenDRIVE/road[1]/lanes/laneSection/left/lane[2]",
        "/OpenDRIVE/road[2]/lanes/laneSection/right/lane[2]",
        "/OpenDRIVE/road[1]/lanes/laneSection/left/lane[3]",
        "/OpenDRIVE/road[2]/lanes/laneSection/right/lane[1]",
        "/OpenDRIVE/road[11]/lanes/laneSection[1]/right/lane[3]",
        "/OpenDRIVE/road[11]/lanes/laneSection[2]/right/lane[4]",
        "/OpenDRIVE/road[11]/lanes/laneSection[1]/right/lane[4]",
        "/OpenDRIVE/road[11]/lanes/laneSection[2]/right/lane[5]",
        "/OpenDRIVE/road[12]/lanes/laneSection[1]/left/lane[3]",
        "/OpenDRIVE/road[12]/lanes/laneSection[2]/left/lane[4]",
        "/OpenDRIVE/road[12]/lanes/laneSection[1]/left/lane[4]",
        "/OpenDRIVE/road[12]/lanes/laneSection[2]/left/lane[5]",
        "/OpenDRIVE/road[12]/lanes/laneSection[1]/right/lane[5]",
        "/OpenDRIVE/road[12]/lanes/laneSection[2]/right/lane[4]",
        "/OpenDRIVE/road[15]/planView/geometry[2]",
        "/OpenDRIVE/road[15]/planView/geometry[3]",
        "/OpenDRIVE/road[3]/lanes/laneSection/left/lane[1]",
        "/OpenDRIVE/road[6]/lanes/laneSection/left/lane[1]",
        "/OpenDRIVE/road[3]/lanes/laneSection/left/lane[2]",
        "/OpenDRIVE/road[6]/lanes/laneSection/left/lane[2]",
        "/OpenDRIVE/road[3]/lanes/laneSection/left/lane[3]",
        "/OpenDRIVE/road[6]/lanes/laneSection/left/lane[3]",
        "/OpenDRIVE/road[9]/lanes/laneSection/left/lane[1]",
        "/OpenDRIVE/road[10]/lanes/laneSection/left/lane[1]",
        "/OpenDRIVE/road[9]/lanes/laneSection/left/lane[2]",
        "/OpenDRIVE/road[10]/lanes/laneSection/left/lane[2]",
        "/OpenDRIVE/road[9]/lanes/laneSection/left/lane[3]",
        "/OpenDRIVE/road[10]/lanes/laneSection/left/lane[3]",
    ], IssueSeverity::Error, CHECKER);

    let r = common::run_on("tests/data/smoothness_example/simple_valid.xodr");
    common::check_issues(&r, RULE, 0, &[], IssueSeverity::Error, CHECKER);

    let r = common::run_on("tests/data/smoothness_example/junction_invalid_conn_smoothness.xodr");
    common::check_issues(&r, RULE, 2, &["/OpenDRIVE/road[1]/lanes/laneSection/right/lane", "/OpenDRIVE/road[3]/lanes/laneSection/left/lane", "/OpenDRIVE/road[2]/lanes/laneSection/left/lane", "/OpenDRIVE/road[2]/lanes/laneSection/right/lane"], IssueSeverity::Error, CHECKER);

    let r = common::run_on("tests/data/smoothness_example/junction_valid_conn_smoothness.xodr");
    common::check_issues(&r, RULE, 0, &[], IssueSeverity::Error, CHECKER);

    let r = common::run_on("tests/data/smoothness_example/multiple_successor_invalid.xodr");
    common::check_issues(&r, RULE, 1, &["/OpenDRIVE/road/lanes/laneSection[1]/right/lane", "/OpenDRIVE/road/lanes/laneSection[2]/right/lane[2]"], IssueSeverity::Error, CHECKER);

    let r = common::run_on("tests/data/smoothness_example/multiple_successor_valid.xodr");
    common::check_issues(&r, RULE, 0, &[], IssueSeverity::Error, CHECKER);

    let r = common::run_on("tests/data/smoothness_example/lane_gap_example_issue_119.xodr");
    common::check_issues(&r, RULE, 0, &[], IssueSeverity::Error, CHECKER);
}
