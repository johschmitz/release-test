// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Smoothness checker (26), ported from qc_opendrive/checks/smoothness/*.py.

use std::collections::{HashMap, HashSet};

use roxmltree::Node;

use crate::opendrive::checks::{add_location_pair, CheckerData, CheckerSpec};
use crate::opendrive::models::{ContactPoint, LaneSectionWithLength, LinkageTag, Point3D, RoadLinkage};
use crate::opendrive::utils;
use crate::result::IssueSeverity;

const TOLERANCE_THRESHOLD: f64 = 0.01;

const DRIVABLE_LANE_TYPES: &[&str] = &[
    "driving", "entry", "exit", "onRamp", "offRamp", "connectingRamp", "slipLane",
    "parking", "biking", "border", "stop", "restricted",
];

fn is_drivable(lane: Node) -> bool {
    match utils::get_type_from_lane(lane) {
        Some(t) => DRIVABLE_LANE_TYPES.contains(&t),
        None => false,
    }
}

fn euclidean(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

fn raise_geometry_gap_issue(
    cd: &mut CheckerData,
    road: Node,
    previous_geometry: Node,
    geometry: Node,
    distance: f64,
) {
    let id = "check_asam_xodr_lane_smoothness_contact_point_no_horizontal_gaps";
    let rule = "asam.net:xodr:1.7.0:lane_smoothness.contact_point_no_horizontal_gaps";
    let desc = format!(
        "The transition between geometry elements should be defined with no gaps. A gap of {} meters has been found.",
        distance
    );
    let issue_id = cd.result.register_issue(id, &desc, IssueSeverity::Error, rule);
    add_location_pair(cd, id, issue_id, "First geometry element", previous_geometry, previous_geometry);
    add_location_pair(cd, id, issue_id, "Second geometry element", geometry, geometry);
    let x0 = utils::get_x_from_geometry(geometry);
    let y0 = utils::get_y_from_geometry(geometry);
    let current_s = utils::get_s_from_geometry(geometry);
    if let (Some(x0), Some(y0), Some(current_s)) = (x0, y0, current_s) {
        if let Some(elevation) = utils::get_elevation_from_road_by_s(road, current_s) {
            let z0 = utils::calculate_elevation_value(&elevation, current_s);
            cd.result
                .add_inertial_location(id, issue_id, x0, y0, z0, &format!("Point where the transition between geometry elements has a gap of {} meters.", distance));
        }
    }
}

fn raise_lane_linkage_gap_issue(
    cd: &mut CheckerData,
    previous_lane: Node,
    current_lane: Node,
    raised_issue_xpaths: &mut HashSet<String>,
) {
    let id = "check_asam_xodr_lane_smoothness_contact_point_no_horizontal_gaps";
    let rule = "asam.net:xodr:1.7.0:lane_smoothness.contact_point_no_horizontal_gaps";
    let desc = "The transition between lane elements should be defined with no gaps.";
    let issue_xpaths = vec![
        utils::node_xpath(previous_lane),
        utils::node_xpath(current_lane),
    ];
    let prev_id = utils::get_lane_id(previous_lane);
    let cur_id = utils::get_lane_id(current_lane);
    let mut sorted = issue_xpaths.clone();
    sorted.sort();
    let key = sorted.join("-");
    if raised_issue_xpaths.contains(&key) {
        return;
    }
    raised_issue_xpaths.insert(key);
    eprintln!("RAISED prev={:?} cur={:?}", prev_id, cur_id);
    let issue_id = cd.result.register_issue(id, desc, IssueSeverity::Error, rule);
    add_location_pair(cd, id, issue_id, "First lane element", previous_lane, previous_lane);
    add_location_pair(cd, id, issue_id, "Next lane element", current_lane, current_lane);
}

fn check_geometries_gap(cd: &mut CheckerData, road: Node, previous_geometry: Node, current_geometry: Node) {
    let x0 = match utils::get_x_from_geometry(current_geometry) {
        Some(v) => v,
        None => return,
    };
    let y0 = match utils::get_y_from_geometry(current_geometry) {
        Some(v) => v,
        None => return,
    };
    let previous_s0 = match utils::get_s_from_geometry(previous_geometry) {
        Some(v) => v,
        None => return,
    };
    let previous_length = match utils::get_length_from_geometry(previous_geometry) {
        Some(v) => v,
        None => return,
    };
    let previous_end = utils::get_point_xy_from_geometry(previous_geometry, previous_s0 + previous_length);
    if let Some(prev_end) = previous_end {
        let gap = euclidean((prev_end.x, prev_end.y), (x0, y0));
        if gap > TOLERANCE_THRESHOLD {
            raise_geometry_gap_issue(cd, road, previous_geometry, current_geometry, gap);
        }
    }
}

fn check_plan_view_gaps(cd: &mut CheckerData, road: Node, geometries: &[Node]) {
    let mut previous: Option<Node> = None;
    for &geometry in geometries {
        if let Some(prev) = previous {
            check_geometries_gap(cd, road, prev, geometry);
        }
        previous = Some(geometry);
    }
}

fn compute_inner_point(
    lanes_outer_points: &HashMap<i64, Option<f64>>,
    lane_id: i64,
    road: Node,
    road_s: f64,
) -> Option<Point3D> {
    let sign = if lane_id < 0 { -1 } else { 1 };
    let current_lane_t = lanes_outer_points.get(&(lane_id - 1 * sign))?;
    let current_lane_t = (*current_lane_t)?;
    utils::get_point_xyz_from_road(road, road_s, current_lane_t, 0.0)
}

fn compute_outer_point(
    lanes_outer_points: &HashMap<i64, Option<f64>>,
    lane_id: i64,
    road: Node,
    road_s: f64,
) -> Option<Point3D> {
    let current_lane_t = lanes_outer_points.get(&lane_id)?;
    let current_lane_t = (*current_lane_t)?;
    utils::get_point_xyz_from_road(road, road_s, current_lane_t, 0.0)
}

fn compute_middle_point(
    lanes_outer_points: &HashMap<i64, Option<f64>>,
    lane_id: i64,
    road: Node,
    road_s: f64,
) -> Option<Point3D> {
    let outer = compute_outer_point(lanes_outer_points, lane_id, road, road_s)?;
    let inner = compute_inner_point(lanes_outer_points, lane_id, road, road_s)?;
    Some(Point3D {
        x: (inner.x + outer.x) / 2.0,
        y: (inner.y + outer.y) / 2.0,
        z: (inner.z + outer.z) / 2.0,
    })
}

fn equal_outer_border_points(
    road: Node,
    lane_id: i64,
    current_outer_points: &HashMap<i64, Option<f64>>,
    current_s: f64,
    next_lane_id: i64,
    successor_outer_points: &HashMap<i64, Option<f64>>,
    successor_s: f64,
) -> bool {
    let current_xy = compute_outer_point(current_outer_points, lane_id, road, current_s);
    let next_xy = compute_outer_point(successor_outer_points, next_lane_id, road, successor_s);
    match (current_xy, next_xy) {
        (Some(a), Some(b)) => euclidean((a.x, a.y), (b.x, b.y)) <= TOLERANCE_THRESHOLD,
        _ => false,
    }
}

fn equal_inner_border_points(
    road: Node,
    lane_id: i64,
    current_outer_points: &HashMap<i64, Option<f64>>,
    current_s: f64,
    next_lane_id: i64,
    successor_outer_points: &HashMap<i64, Option<f64>>,
    successor_s: f64,
) -> bool {
    let current_xy = compute_inner_point(current_outer_points, lane_id, road, current_s);
    let next_xy = compute_inner_point(successor_outer_points, next_lane_id, road, successor_s);
    match (current_xy, next_xy) {
        (Some(a), Some(b)) => euclidean((a.x, a.y), (b.x, b.y)) <= TOLERANCE_THRESHOLD,
        _ => false,
    }
}

fn validate_same_road_lane_successors(
    cd: &mut CheckerData,
    road: Node,
    lane: Node,
    current_outer_points: &HashMap<i64, Option<f64>>,
    current_road_s: f64,
    successor_outer_points: &HashMap<i64, Option<f64>>,
    successor_road_s: f64,
    successor_lanes: &[Node],
    raised_issue_xpaths: &mut HashSet<String>,
) {
    let successors = utils::get_successor_lane_ids(lane);
    let lane_id = match utils::get_lane_id(lane) {
        Some(v) => v,
        None => return,
    };
    if successors.len() == 1 {
        let next_lane_id = successors[0];
        if !equal_outer_border_points(road, lane_id, current_outer_points, current_road_s, next_lane_id, successor_outer_points, successor_road_s)
            || !equal_inner_border_points(road, lane_id, current_outer_points, current_road_s, next_lane_id, successor_outer_points, successor_road_s)
        {
            let next_lane = successor_lanes.iter().find(|l| utils::get_lane_id(**l) == Some(next_lane_id));
            if let Some(next_lane) = next_lane {
                let p = compute_middle_point(successor_outer_points, next_lane_id, road, successor_road_s);
                raise_lane_linkage_gap_issue(cd, lane, *next_lane, raised_issue_xpaths);
                let _ = p;
            }
        }
    } else if successors.len() == 2 {
        let reverse = lane_id < 0;
        let mut sorted = successors.clone();
        sorted.sort();
        if reverse {
            sorted.reverse();
        }
        let upper_successor_id = sorted[0];
        let bottom_successor_id = sorted[sorted.len() - 1];
        if !equal_outer_border_points(road, lane_id, current_outer_points, current_road_s, bottom_successor_id, successor_outer_points, successor_road_s) {
            let next_lane = successor_lanes.iter().find(|l| utils::get_lane_id(**l) == Some(bottom_successor_id));
            if let Some(next_lane) = next_lane {
                raise_lane_linkage_gap_issue(cd, lane, *next_lane, raised_issue_xpaths);
            }
        }
        if !equal_inner_border_points(road, lane_id, current_outer_points, current_road_s, upper_successor_id, successor_outer_points, successor_road_s) {
            let next_lane = successor_lanes.iter().find(|l| utils::get_lane_id(**l) == Some(upper_successor_id));
            if let Some(next_lane) = next_lane {
                raise_lane_linkage_gap_issue(cd, lane, *next_lane, raised_issue_xpaths);
            }
        }
    } else if successors.len() > 2 {
        for &extra_lane_id in &successors[1..successors.len() - 1] {
            let next_lane = successor_lanes.iter().find(|l| utils::get_lane_id(**l) == Some(extra_lane_id));
            if let Some(next_lane) = next_lane {
                raise_lane_linkage_gap_issue(cd, lane, *next_lane, raised_issue_xpaths);
            }
        }
    }
}

fn validate_same_road_lane_predecessors(
    cd: &mut CheckerData,
    road: Node,
    lane: Node,
    prev_outer_points: &HashMap<i64, Option<f64>>,
    prev_road_s: f64,
    current_outer_points: &HashMap<i64, Option<f64>>,
    current_road_s: f64,
    prev_lanes: &[Node],
    raised_issue_xpaths: &mut HashSet<String>,
) {
    let lane_id = match utils::get_lane_id(lane) {
        Some(v) => v,
        None => return,
    };
    let predecessors = utils::get_predecessor_lane_ids(lane);
    if predecessors.len() == 1 {
        let prev_lane_id = predecessors[0];
        if !equal_outer_border_points(road, prev_lane_id, prev_outer_points, prev_road_s, lane_id, current_outer_points, current_road_s)
            || !equal_inner_border_points(road, prev_lane_id, prev_outer_points, prev_road_s, lane_id, current_outer_points, current_road_s)
        {
            let prev_lane = prev_lanes.iter().find(|l| utils::get_lane_id(**l) == Some(prev_lane_id));
            if let Some(prev_lane) = prev_lane {
                raise_lane_linkage_gap_issue(cd, *prev_lane, lane, raised_issue_xpaths);
            }
        }
    } else if predecessors.len() == 2 {
        let reverse = lane_id < 0;
        let mut sorted = predecessors.clone();
        sorted.sort();
        if reverse {
            sorted.reverse();
        }
        let upper_prev_id = sorted[0];
        let bottom_prev_id = sorted[sorted.len() - 1];
        if !equal_outer_border_points(road, upper_prev_id, prev_outer_points, prev_road_s, lane_id, current_outer_points, current_road_s) {
            let prev_lane = prev_lanes.iter().find(|l| utils::get_lane_id(**l) == Some(upper_prev_id));
            if let Some(prev_lane) = prev_lane {
                raise_lane_linkage_gap_issue(cd, *prev_lane, lane, raised_issue_xpaths);
            }
        }
        if !equal_inner_border_points(road, bottom_prev_id, prev_outer_points, prev_road_s, lane_id, current_outer_points, current_road_s) {
            let prev_lane = prev_lanes.iter().find(|l| utils::get_lane_id(**l) == Some(bottom_prev_id));
            if let Some(prev_lane) = prev_lane {
                raise_lane_linkage_gap_issue(cd, *prev_lane, lane, raised_issue_xpaths);
            }
        }
    } else if predecessors.len() > 2 {
        for &extra_lane_id in &predecessors[1..predecessors.len() - 1] {
            let prev_lane = prev_lanes.iter().find(|l| utils::get_lane_id(**l) == Some(extra_lane_id));
            if let Some(prev_lane) = prev_lane {
                raise_lane_linkage_gap_issue(cd, *prev_lane, lane, raised_issue_xpaths);
            }
        }
    }
}

fn check_road_lane_sections_gaps(cd: &mut CheckerData, road: Node, raised_issue_xpaths: &mut HashSet<String>) {
    let lane_sections = utils::get_sorted_lane_sections_with_length_from_road(road);
    if lane_sections.len() < 2 {
        return;
    }
    for index in 0..lane_sections.len() - 1 {
        let current_lane_section = lane_sections[index];
        let next_lane_section = lane_sections[index + 1];
        let current_lanes = utils::get_left_and_right_lanes_from_lane_section(current_lane_section.lane_section);
        let next_lanes = utils::get_left_and_right_lanes_from_lane_section(next_lane_section.lane_section);
        let current_lane_section_s = match utils::get_s_from_lane_section(current_lane_section.lane_section) {
            Some(v) => v,
            None => continue,
        };
        let next_lane_section_s = match utils::get_s_from_lane_section(next_lane_section.lane_section) {
            Some(v) => v,
            None => continue,
        };
        let lane_offset = utils::get_lane_offset_value_from_road_by_s(road, current_lane_section_s + current_lane_section.length).unwrap_or(0.0);
        let successor_lane_offset = utils::get_lane_offset_value_from_road_by_s(road, next_lane_section_s).unwrap_or(0.0);
        let mut current_outer_points = utils::get_outer_border_points_from_lane_group_by_s(
            &current_lanes, lane_offset, current_lane_section_s, current_lane_section_s + current_lane_section.length,
        );
        current_outer_points.insert(0, Some(lane_offset));
        let mut successor_outer_points = utils::get_outer_border_points_from_lane_group_by_s(
            &next_lanes, successor_lane_offset, next_lane_section_s, next_lane_section_s,
        );
        successor_outer_points.insert(0, Some(successor_lane_offset));
        for lane in &current_lanes {
            if !is_drivable(*lane) {
                continue;
            }
            validate_same_road_lane_successors(
                cd, road, *lane, &current_outer_points, current_lane_section_s + current_lane_section.length,
                &successor_outer_points, next_lane_section_s, &next_lanes, raised_issue_xpaths,
            );
        }
        for lane in &next_lanes {
            if !is_drivable(*lane) {
                continue;
            }
            validate_same_road_lane_predecessors(
                cd, road, *lane, &current_outer_points, current_lane_section_s + current_lane_section.length,
                &successor_outer_points, next_lane_section_s, &current_lanes, raised_issue_xpaths,
            );
        }
    }
}

fn validate_inter_road_smoothness(
    cd: &mut CheckerData,
    road: Node,
    linkage: RoadLinkage,
    road_relation: LinkageTag,
    road_lane_section: LaneSectionWithLength,
    road_s: f64,
    road_id_map: &HashMap<i64, Node>,
    raised_issue_xpaths: &mut HashSet<String>,
) {
    let target_road = match road_id_map.get(&linkage.id) {
        Some(v) => *v,
        None => return,
    };
    let target_road_length = match utils::get_road_length(target_road) {
        Some(v) => v,
        None => return,
    };
    let target_lane_sections = utils::get_sorted_lane_sections_with_length_from_road(target_road);
    if target_lane_sections.is_empty() {
        return;
    }
    let (target_lane_section, target_s) = match linkage.contact_point {
        ContactPoint::End => (target_lane_sections[target_lane_sections.len() - 1], target_road_length),
        ContactPoint::Start => (target_lane_sections[0], 0.0),
    };
    let lanes = utils::get_left_and_right_lanes_from_lane_section(road_lane_section.lane_section);
    let target_lanes = utils::get_left_and_right_lanes_from_lane_section(target_lane_section.lane_section);
    let lane_offset = utils::get_lane_offset_value_from_road_by_s(road, road_s).unwrap_or(0.0);
    let target_lane_offset = utils::get_lane_offset_value_from_road_by_s(target_road, target_s).unwrap_or(0.0);
    let mut lanes_outer_points = utils::get_outer_border_points_from_lane_group_by_s(
        &lanes, lane_offset, utils::get_s_from_lane_section(road_lane_section.lane_section).unwrap_or(0.0), road_s,
    );
    lanes_outer_points.insert(0, Some(lane_offset));
    let mut target_lanes_outer_points = utils::get_outer_border_points_from_lane_group_by_s(
        &target_lanes, target_lane_offset, utils::get_s_from_lane_section(target_lane_section.lane_section).unwrap_or(0.0), target_s,
    );
    target_lanes_outer_points.insert(0, Some(target_lane_offset));
    for lane in &lanes {
        if !is_drivable(*lane) {
            continue;
        }
        let connections = match road_relation {
            LinkageTag::Predecessor => utils::get_predecessor_lane_ids(*lane),
            LinkageTag::Successor => utils::get_successor_lane_ids(*lane),
        };
        let lane_id = match utils::get_lane_id(*lane) {
            Some(v) => v,
            None => continue,
        };
        let current_c0 = compute_inner_point(&lanes_outer_points, lane_id, road, road_s);
        let current_c1 = compute_outer_point(&lanes_outer_points, lane_id, road, road_s);
        let (current_c0, current_c1) = match (current_c0, current_c1) {
            (Some(a), Some(b)) => (a, b),
            _ => continue,
        };
        let matches_threshold = if connections.len() > 1 { 1 } else { 2 };
        for conn_lane_id in &connections {
            let target_c0 = compute_inner_point(&target_lanes_outer_points, *conn_lane_id, target_road, target_s);
            let target_c1 = compute_outer_point(&target_lanes_outer_points, *conn_lane_id, target_road, target_s);
            let (target_c0, target_c1) = match (target_c0, target_c1) {
                (Some(a), Some(b)) => (a, b),
                _ => continue,
            };
            let mut matches = 0;
            if euclidean((current_c0.x, current_c0.y), (target_c0.x, target_c0.y)) < TOLERANCE_THRESHOLD {
                matches += 1;
            }
            if euclidean((current_c0.x, current_c0.y), (target_c1.x, target_c1.y)) < TOLERANCE_THRESHOLD {
                matches += 1;
            }
            if euclidean((current_c1.x, current_c1.y), (target_c0.x, target_c0.y)) < TOLERANCE_THRESHOLD {
                matches += 1;
            }
            if euclidean((current_c1.x, current_c1.y), (target_c1.x, target_c1.y)) < TOLERANCE_THRESHOLD {
                matches += 1;
            }
            if matches < matches_threshold {
                let target_lane = target_lanes.iter().find(|l| utils::get_lane_id(**l) == Some(*conn_lane_id));
                if let Some(target_lane) = target_lane {
                    match road_relation {
                        LinkageTag::Predecessor => {
                            let p = compute_middle_point(&lanes_outer_points, lane_id, road, road_s);
                            raise_lane_linkage_gap_issue(cd, *target_lane, *lane, raised_issue_xpaths);
                            let _ = p;
                        }
                        LinkageTag::Successor => {
                            let p = compute_middle_point(&target_lanes_outer_points, *conn_lane_id, target_road, target_s);
                            raise_lane_linkage_gap_issue(cd, *lane, *target_lane, raised_issue_xpaths);
                            let _ = p;
                        }
                    }
                }
            }
        }
    }
}

fn validate_junction_connection_gaps(
    cd: &mut CheckerData,
    incoming_road: Node,
    incoming_lane_section: LaneSectionWithLength,
    incoming_road_s: f64,
    connection: Node,
    road_id_map: &HashMap<i64, Node>,
    raised_issue_xpaths: &mut HashSet<String>,
) {
    let connection_contact_point = match utils::get_contact_point_from_connection(connection) {
        Some(v) => v,
        None => return,
    };
    let connection_road_id = match utils::get_connecting_road_id_from_connection(connection) {
        Some(v) => v,
        None => return,
    };
    let connection_road = match road_id_map.get(&connection_road_id) {
        Some(v) => *v,
        None => return,
    };
    let lane_links = utils::get_lane_links_from_connection(connection);
    if lane_links.is_empty() {
        return;
    }
    let target_road = connection_road;
    let target_road_length = match utils::get_road_length(target_road) {
        Some(v) => v,
        None => return,
    };
    let target_lane_sections = utils::get_sorted_lane_sections_with_length_from_road(target_road);
    if target_lane_sections.is_empty() {
        return;
    }
    let (target_lane_section, target_s) = match connection_contact_point {
        ContactPoint::End => (target_lane_sections[target_lane_sections.len() - 1], target_road_length),
        ContactPoint::Start => (target_lane_sections[0], 0.0),
    };
    let lanes = utils::get_left_and_right_lanes_from_lane_section(incoming_lane_section.lane_section);
    let target_lanes = utils::get_left_and_right_lanes_from_lane_section(target_lane_section.lane_section);
    let lane_offset = utils::get_lane_offset_value_from_road_by_s(incoming_road, incoming_road_s).unwrap_or(0.0);
    let target_lane_offset = utils::get_lane_offset_value_from_road_by_s(target_road, target_s).unwrap_or(0.0);
    let mut lanes_outer_points = utils::get_outer_border_points_from_lane_group_by_s(
        &lanes, lane_offset, utils::get_s_from_lane_section(incoming_lane_section.lane_section).unwrap_or(0.0), incoming_road_s,
    );
    lanes_outer_points.insert(0, Some(lane_offset));
    let mut target_lanes_outer_points = utils::get_outer_border_points_from_lane_group_by_s(
        &target_lanes, target_lane_offset, utils::get_s_from_lane_section(target_lane_section.lane_section).unwrap_or(0.0), target_s,
    );
    target_lanes_outer_points.insert(0, Some(target_lane_offset));
    for link in &lane_links {
        let from_id = match utils::get_from_attribute_from_lane_link(*link) {
            Some(v) => v,
            None => continue,
        };
        let to_id = match utils::get_to_attribute_from_lane_link(*link) {
            Some(v) => v,
            None => continue,
        };
        let from_lane = utils::get_lane_from_lane_section(incoming_lane_section.lane_section, from_id);
        let from_lane = match from_lane {
            Some(v) => v,
            None => continue,
        };
        if !is_drivable(from_lane) {
            continue;
        }
        let current_c0 = compute_inner_point(&lanes_outer_points, from_id, incoming_road, incoming_road_s);
        let current_c1 = compute_outer_point(&lanes_outer_points, from_id, incoming_road, incoming_road_s);
        let (current_c0, current_c1) = match (current_c0, current_c1) {
            (Some(a), Some(b)) => (a, b),
            _ => continue,
        };
        let target_c0 = compute_inner_point(&target_lanes_outer_points, to_id, target_road, target_s);
        let target_c1 = compute_outer_point(&target_lanes_outer_points, to_id, target_road, target_s);
        let (target_c0, target_c1) = match (target_c0, target_c1) {
            (Some(a), Some(b)) => (a, b),
            _ => continue,
        };
        let mut matches = 0;
        if euclidean((current_c0.x, current_c0.y), (target_c0.x, target_c0.y)) < TOLERANCE_THRESHOLD {
            matches += 1;
        }
        if euclidean((current_c0.x, current_c0.y), (target_c1.x, target_c1.y)) < TOLERANCE_THRESHOLD {
            matches += 1;
        }
        if euclidean((current_c1.x, current_c1.y), (target_c0.x, target_c0.y)) < TOLERANCE_THRESHOLD {
            matches += 1;
        }
        if euclidean((current_c1.x, current_c1.y), (target_c1.x, target_c1.y)) < TOLERANCE_THRESHOLD {
            matches += 1;
        }
        if matches < 2 {
            let target_lane_id = target_lanes.iter().find_map(|l| {
                if utils::get_lane_id(*l) == Some(to_id) {
                    Some(utils::get_lane_id(*l))
                } else {
                    None
                }
            });
            eprintln!("[DBG junc] from_id={} to_id={} from_lane_id={:?} target_lane_id={:?} matches={} cur0=({:.4},{:.4}) cur1=({:.4},{:.4}) tgt0=({:.4},{:.4}) tgt1=({:.4},{:.4})",
                from_id, to_id, utils::get_lane_id(from_lane), target_lane_id, matches,
                current_c0.x, current_c0.y, current_c1.x, current_c1.y, target_c0.x, target_c0.y, target_c1.x, target_c1.y);
            let target_lane = target_lanes.iter().find(|l| utils::get_lane_id(**l) == Some(to_id));
            if let Some(target_lane) = target_lane {
                raise_lane_linkage_gap_issue(cd, from_lane, *target_lane, raised_issue_xpaths);
            }
        }
    }
}

fn check_roads_internal_smoothness(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let mut raised_issue_xpaths: HashSet<String> = HashSet::new();
    let road_id_map = utils::get_road_id_map(doc);
    for (_rid, road) in &road_id_map {
        let geometries = utils::get_road_plan_view_geometry_list(*road);
        if geometries.len() > 2 {
            check_plan_view_gaps(cd, *road, &geometries);
        }
        check_road_lane_sections_gaps(cd, *road, &mut raised_issue_xpaths);
    }
}

fn check_inter_roads_smoothness(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let mut raised_issue_xpaths: HashSet<String> = HashSet::new();
    let road_id_map = utils::get_road_id_map(doc);
    let junction_id_map = utils::get_junction_id_map(doc);
    for (road_id, road) in &road_id_map {
        let successor = utils::get_road_linkage(*road, LinkageTag::Successor);
        let predecessor = utils::get_road_linkage(*road, LinkageTag::Predecessor);
        let road_lane_sections = utils::get_sorted_lane_sections_with_length_from_road(*road);
        if road_lane_sections.is_empty() {
            continue;
        }
        let road_length = match utils::get_road_length(*road) {
            Some(v) => v,
            None => continue,
        };
        if let Some(succ) = successor {
            validate_inter_road_smoothness(
                cd, *road, succ, LinkageTag::Successor, road_lane_sections[road_lane_sections.len() - 1], road_length,
                &road_id_map, &mut raised_issue_xpaths,
            );
        }
        if let Some(pred) = predecessor {
            validate_inter_road_smoothness(
                cd, *road, pred, LinkageTag::Predecessor, road_lane_sections[0], 0.0,
                &road_id_map, &mut raised_issue_xpaths,
            );
        }
        if let Some(successor_junction_id) = utils::get_linked_junction_id(*road, LinkageTag::Successor) {
            let connections = utils::get_connections_between_road_and_junction(
                *road_id, successor_junction_id, &road_id_map, &junction_id_map, ContactPoint::End,
            );
            let incoming_lane_section = road_lane_sections[road_lane_sections.len() - 1];
            for connection in connections {
                validate_junction_connection_gaps(
                    cd, *road, incoming_lane_section, road_length, connection, &road_id_map, &mut raised_issue_xpaths,
                );
            }
        }
        if let Some(predecessor_junction_id) = utils::get_linked_junction_id(*road, LinkageTag::Predecessor) {
            let connections = utils::get_connections_between_road_and_junction(
                *road_id, predecessor_junction_id, &road_id_map, &junction_id_map, ContactPoint::Start,
            );
            let incoming_lane_section = road_lane_sections[0];
            for connection in connections {
                validate_junction_connection_gaps(
                    cd, *road, incoming_lane_section, 0.0, connection, &road_id_map, &mut raised_issue_xpaths,
                );
            }
        }
    }
}

fn check_lane_smoothness_contact_point_no_horizontal_gaps(cd: &mut CheckerData) {
    check_roads_internal_smoothness(cd);
    check_inter_roads_smoothness(cd);
}

pub fn specs() -> Vec<CheckerSpec> {
    vec![CheckerSpec {
        id: "check_asam_xodr_lane_smoothness_contact_point_no_horizontal_gaps",
        description: "Contact point no horizontal gaps.",
        rule_uid: "asam.net:xodr:1.7.0:lane_smoothness.contact_point_no_horizontal_gaps",
        preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
        applicable_versions: None,
        version_required: true,
        run: check_lane_smoothness_contact_point_no_horizontal_gaps,
    }]
}
