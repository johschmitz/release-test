// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Performance checker (25), ported from qc_opendrive/checks/performance/*.py.

use roxmltree::Node;

use crate::opendrive::checks::{add_location_pair, CheckerData, CheckerSpec};
use crate::opendrive::utils;
use crate::result::IssueSeverity;

const FLOAT_TOLERANCE: f64 = 1e-6;

fn check_road_superelevations(cd: &mut CheckerData, road: Node) {
    let id = "check_asam_xodr_performance_avoid_redundant_info";
    let rule = "asam.net:xodr:1.7.0:performance.avoid_redundant_info";
    let desc = "Redundant superelevation declaration.";
    let list = utils::get_road_superelevations(road);
    for i in 0..list.len().saturating_sub(1) {
        if utils::are_same_equations(&list[i], &list[i + 1]) {
            let cur = list[i].xml_element.unwrap();
            let nxt = list[i + 1].xml_element.unwrap();
            let issue_id = cd
                .result
                .register_issue(id, desc, IssueSeverity::Warning, rule);
            add_location_pair(cd, id, issue_id, desc, cur, cur);
            add_location_pair(cd, id, issue_id, desc, nxt, nxt);
            if let Some(p) = utils::get_point_xyz_from_road_reference_line(road, list[i + 1].s_offset) {
                cd.result
                    .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
            }
        }
    }
}

fn check_road_elevations(cd: &mut CheckerData, road: Node) {
    let id = "check_asam_xodr_performance_avoid_redundant_info";
    let rule = "asam.net:xodr:1.7.0:performance.avoid_redundant_info";
    let desc = "Redundant elevation declaration.";
    let list = utils::get_road_elevations(road);
    for i in 0..list.len().saturating_sub(1) {
        if utils::are_same_equations(&list[i], &list[i + 1]) {
            let cur = list[i].xml_element.unwrap();
            let nxt = list[i + 1].xml_element.unwrap();
            let issue_id = cd
                .result
                .register_issue(id, desc, IssueSeverity::Warning, rule);
            add_location_pair(cd, id, issue_id, desc, cur, cur);
            add_location_pair(cd, id, issue_id, desc, nxt, nxt);
            if let Some(p) = utils::get_point_xyz_from_road_reference_line(road, list[i + 1].s_offset) {
                cd.result
                    .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
            }
        }
    }
}

fn check_lane_offsets(cd: &mut CheckerData, road: Node) {
    let id = "check_asam_xodr_performance_avoid_redundant_info";
    let rule = "asam.net:xodr:1.7.0:performance.avoid_redundant_info";
    let desc = "Redundant lane offset declaration.";
    let list = utils::get_lane_offsets_from_road(road);
    for i in 0..list.len().saturating_sub(1) {
        if utils::are_same_equations(&list[i], &list[i + 1]) {
            let cur = list[i].xml_element.unwrap();
            let nxt = list[i + 1].xml_element.unwrap();
            let issue_id = cd
                .result
                .register_issue(id, desc, IssueSeverity::Warning, rule);
            add_location_pair(cd, id, issue_id, desc, cur, cur);
            add_location_pair(cd, id, issue_id, desc, nxt, nxt);
            let s = list[i + 1].s_offset;
            let t = list[i + 1].poly3.evaluate(0.0);
            if let Some(p) = utils::get_point_xyz_from_road(road, s, t, 0.0) {
                cd.result
                    .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
            }
        }
    }
}

fn check_road_plan_view(cd: &mut CheckerData, road: Node) {
    let id = "check_asam_xodr_performance_avoid_redundant_info";
    let rule = "asam.net:xodr:1.7.0:performance.avoid_redundant_info";
    let desc = "Redundant line geometry declaration.";
    let geometry_list = utils::get_road_plan_view_geometry_list(road);
    for i in 0..geometry_list.len().saturating_sub(1) {
        let cur = geometry_list[i];
        let nxt = geometry_list[i + 1];
        let cur_h = match utils::get_heading_from_geometry(cur) {
            Some(v) => v,
            None => continue,
        };
        let nxt_h = match utils::get_heading_from_geometry(nxt) {
            Some(v) => v,
            None => continue,
        };
        if utils::is_line_geometry(cur)
            && utils::is_line_geometry(nxt)
            && (cur_h - nxt_h).abs() < FLOAT_TOLERANCE
        {
            let issue_id = cd
                .result
                .register_issue(id, desc, IssueSeverity::Warning, rule);
            add_location_pair(cd, id, issue_id, desc, cur, cur);
            add_location_pair(cd, id, issue_id, desc, nxt, nxt);
            if let Some(s) = utils::get_s_from_geometry(nxt) {
                if let Some(p) = utils::get_point_xyz_from_road_reference_line(road, s) {
                    cd.result
                        .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
                }
            }
        }
    }
}

fn check_lane_widths(cd: &mut CheckerData, road: Node, lane_section: Node, lane: Node) {
    let id = "check_asam_xodr_performance_avoid_redundant_info";
    let rule = "asam.net:xodr:1.7.0:performance.avoid_redundant_info";
    let desc = "Redundant lane width declaration.";
    let widths = utils::get_lane_width_poly3_list(lane);
    for i in 0..widths.len().saturating_sub(1) {
        if utils::are_same_equations(&widths[i], &widths[i + 1]) {
            let cur = widths[i].xml_element.unwrap();
            let nxt = widths[i + 1].xml_element.unwrap();
            let issue_id = cd
                .result
                .register_issue(id, desc, IssueSeverity::Warning, rule);
            add_location_pair(cd, id, issue_id, desc, cur, cur);
            add_location_pair(cd, id, issue_id, desc, nxt, nxt);
            let s_section = match utils::get_s_from_lane_section(lane_section) {
                Some(v) => v,
                None => continue,
            };
            let s = s_section + widths[i + 1].s_offset;
            if let Some(p) = utils::get_middle_point_xyz_at_height_zero_from_lane_by_s(road, lane_section, lane, s) {
                cd.result
                    .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
            }
        }
    }
}

fn check_lane_borders(cd: &mut CheckerData, road: Node, lane_section: Node, lane: Node) {
    let id = "check_asam_xodr_performance_avoid_redundant_info";
    let rule = "asam.net:xodr:1.7.0:performance.avoid_redundant_info";
    let desc = "Redundant lane border declaration.";
    let borders = utils::get_borders_from_lane(lane);
    for i in 0..borders.len().saturating_sub(1) {
        if utils::are_same_equations(&borders[i], &borders[i + 1]) {
            let cur = borders[i].xml_element.unwrap();
            let nxt = borders[i + 1].xml_element.unwrap();
            let issue_id = cd
                .result
                .register_issue(id, desc, IssueSeverity::Warning, rule);
            add_location_pair(cd, id, issue_id, desc, cur, cur);
            add_location_pair(cd, id, issue_id, desc, nxt, nxt);
            let s_section = match utils::get_s_from_lane_section(lane_section) {
                Some(v) => v,
                None => continue,
            };
            let s = s_section + borders[i + 1].s_offset;
            if let Some(p) = utils::get_middle_point_xyz_at_height_zero_from_lane_by_s(road, lane_section, lane, s) {
                cd.result
                    .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
            }
        }
    }
}

fn check_performance_avoid_redundant_info(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    for road in utils::get_roads(doc) {
        check_road_elevations(cd, road);
        check_road_superelevations(cd, road);
        check_lane_offsets(cd, road);
        check_road_plan_view(cd, road);
        for lane_section in utils::get_lane_sections(road) {
            for lane in utils::get_left_and_right_lanes_from_lane_section(lane_section) {
                check_lane_widths(cd, road, lane_section, lane);
                check_lane_borders(cd, road, lane_section, lane);
            }
        }
    }
}

pub fn specs() -> Vec<CheckerSpec> {
    vec![CheckerSpec {
        id: "check_asam_xodr_performance_avoid_redundant_info",
        description: "Avoid redundant info.",
        rule_uid: "asam.net:xodr:1.7.0:performance.avoid_redundant_info",
        preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
        applicable_versions: None,
        version_required: true,
        run: check_performance_avoid_redundant_info,
    }]
}
