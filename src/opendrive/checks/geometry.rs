// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Geometry checkers (18-24), ported from qc_opendrive/checks/geometry/*.py.

use roxmltree::Node;

use crate::opendrive::checks::{add_issue, add_location_pair, CheckerData, CheckerSpec};
use crate::opendrive::models::{ContactPoint, Point3D};
use crate::opendrive::utils;
use crate::result::IssueSeverity;

const FLOAT_COMPARISON_THRESHOLD: f64 = 1e-6;
const TOLERANCE_THRESHOLD: f64 = 0.001;

// ---------------------------------------------------------------------------
// road.geometry.contact_point
// ---------------------------------------------------------------------------

fn xcessor_contact_point_has_issue(
    xcessor: Node,
    road_contact_point_xyz: Point3D,
    road_id_map: &std::collections::HashMap<i64, Node>,
) -> Option<Point3D> {
    if xcessor.attribute("elementType") == Some("junction") {
        return None;
    }
    let xcessor_road = match utils::to_int(xcessor.attribute("elementId").unwrap_or("")) {
        Some(id) => road_id_map.get(&id),
        None => return None,
    };
    let xcessor_road = match xcessor_road {
        Some(v) => *v,
        None => return None,
    };
    let contact_point = match xcessor.attribute("contactPoint") {
        Some(v) => v,
        None => return None,
    };
    let xcessor_contact_point_xyz = match utils::get_point_xyz_from_contact_point(xcessor_road, ContactPoint::from_str(contact_point)?) {
        Some(v) => v,
        None => return None,
    };
    if (road_contact_point_xyz.x - xcessor_contact_point_xyz.x).abs() >= FLOAT_COMPARISON_THRESHOLD
        || (road_contact_point_xyz.y - xcessor_contact_point_xyz.y).abs() >= FLOAT_COMPARISON_THRESHOLD
        || (road_contact_point_xyz.z - xcessor_contact_point_xyz.z).abs() >= FLOAT_COMPARISON_THRESHOLD
    {
        Some(xcessor_contact_point_xyz)
    } else {
        None
    }
}

fn check_geometry_contact_point(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_geometry_contact_point";
    let rule = "asam.net:xodr:1.7.0:road.geometry.contact_point";
    let desc = "The road reference line does not begin at the contact point of its predecessor or successor.";
    let road_id_map = utils::get_road_id_map(doc);
    for road in utils::get_roads(doc) {
        if utils::to_int(road.attribute("junction").unwrap_or("")) == Some(1) {
            continue;
        }
        let start = utils::get_start_point_xyz_from_road_reference_line(road);
        let end = utils::get_end_point_xyz_from_road_reference_line(road);
        let link = road.children().find(|n| n.has_tag_name("link"));
        let pred = link
            .and_then(|l| l.children().find(|n| n.has_tag_name("predecessor")));
        let succ = link
            .and_then(|l| l.children().find(|n| n.has_tag_name("successor")));
        for (xcessor, contact_point_xyz) in [(pred, start), (succ, end)] {
            let xcessor = match xcessor {
                Some(v) => v,
                None => continue,
            };
            let contact_point_xyz = match contact_point_xyz {
                Some(v) => v,
                None => continue,
            };
            if let Some(issue_point) = xcessor_contact_point_has_issue(xcessor, contact_point_xyz, &road_id_map) {
                let issue_id = cd
                    .result
                    .register_issue(id, desc, IssueSeverity::Error, rule);
                add_location_pair(cd, id, issue_id, desc, road, road);
                cd.result
                    .add_inertial_location(id, issue_id, issue_point.x, issue_point.y, issue_point.z, desc);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// road.geometry.elem_asc_order
// ---------------------------------------------------------------------------

fn check_geometry_elem_asc_order(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_geometry_elem_asc_order";
    let rule = "asam.net:xodr:1.4.0:road.geometry.elem_asc_order";
    let desc = "<geometry> elements shall be defined in ascending order along the road reference line according to the s-coordinate.";
    for road in utils::get_roads(doc) {
        let geometry_list = utils::get_road_plan_view_geometry_list(road);
        let mut last_s = 0.0;
        for geometry in geometry_list {
            let current_s = match utils::get_s_from_geometry(geometry) {
                Some(v) => v,
                None => continue,
            };
            if current_s >= last_s {
                last_s = current_s;
            } else {
                add_issue(cd, id, rule, desc, IssueSeverity::Error, Some(geometry), Some(geometry), None);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// road.lane.border.overlap_with_inner_lanes
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct BorderPair {
    left_lane_poly3: crate::opendrive::models::Poly3,
    right_lane_poly3: crate::opendrive::models::Poly3,
    ds_left_start: f64,
    ds_right_start: f64,
    ds_length: f64,
}

fn create_border_pairs(
    left_lane_borders: &[crate::opendrive::models::OffsetPoly3],
    right_lane_borders: &[crate::opendrive::models::OffsetPoly3],
    lane_section_length: f64,
) -> Vec<BorderPair> {
    if left_lane_borders.is_empty() || right_lane_borders.is_empty() {
        return Vec::new();
    }
    let mut sorted_left: Vec<&crate::opendrive::models::OffsetPoly3> = left_lane_borders.iter().collect();
    let mut sorted_right: Vec<&crate::opendrive::models::OffsetPoly3> = right_lane_borders.iter().collect();
    sorted_left.sort_by(|a, b| a.s_offset.partial_cmp(&b.s_offset).unwrap());
    sorted_right.sort_by(|a, b| a.s_offset.partial_cmp(&b.s_offset).unwrap());
    let mut left_iter = sorted_left.iter();
    let mut right_iter = sorted_right.iter();
    let mut current_left = *left_iter.next().unwrap();
    let mut current_right = *right_iter.next().unwrap();
    let mut s_offset_start = 0.0;
    let mut next_left = left_iter.next();
    let mut next_right = right_iter.next();
    let mut border_pairs = Vec::new();
    loop {
        let next_left_s_offset = match next_left {
            None => lane_section_length,
            Some(b) => b.s_offset,
        };
        let next_right_s_offset = match next_right {
            None => lane_section_length,
            Some(b) => b.s_offset,
        };
        let s_offset_end = next_left_s_offset.min(next_right_s_offset);
        border_pairs.push(BorderPair {
            left_lane_poly3: current_left.poly3,
            right_lane_poly3: current_right.poly3,
            ds_left_start: s_offset_start - current_left.s_offset,
            ds_right_start: s_offset_start - current_right.s_offset,
            ds_length: s_offset_end - s_offset_start,
        });
        if next_left.is_none() && next_right.is_none() {
            break;
        }
        if next_left.is_some() && (next_left_s_offset - s_offset_end).abs() <= FLOAT_COMPARISON_THRESHOLD {
            current_left = *next_left.unwrap();
            next_left = left_iter.next();
        }
        if next_right.is_some() && (next_right_s_offset - s_offset_end).abs() <= FLOAT_COMPARISON_THRESHOLD {
            current_right = *next_right.unwrap();
            next_right = right_iter.next();
        }
        s_offset_start = s_offset_end;
    }
    border_pairs
}

fn intersect_or_stay_within(border_pair: &BorderPair) -> bool {
    let a = border_pair.left_lane_poly3.a - border_pair.right_lane_poly3.a
        + border_pair.left_lane_poly3.b * border_pair.ds_left_start
        - border_pair.right_lane_poly3.b * border_pair.ds_right_start
        + border_pair.left_lane_poly3.c * border_pair.ds_left_start.powi(2)
        - border_pair.right_lane_poly3.c * border_pair.ds_right_start.powi(2)
        + border_pair.left_lane_poly3.d * border_pair.ds_left_start.powi(3)
        - border_pair.right_lane_poly3.d * border_pair.ds_right_start.powi(3);
    let b = border_pair.left_lane_poly3.b - border_pair.right_lane_poly3.b
        + 2.0 * border_pair.left_lane_poly3.c * border_pair.ds_left_start
        - 2.0 * border_pair.right_lane_poly3.c * border_pair.ds_right_start
        + 3.0 * border_pair.left_lane_poly3.d * border_pair.ds_left_start.powi(2)
        - 3.0 * border_pair.right_lane_poly3.d * border_pair.ds_right_start.powi(2);
    let c = border_pair.left_lane_poly3.c - border_pair.right_lane_poly3.c
        + 3.0 * border_pair.left_lane_poly3.d * border_pair.ds_left_start
        - 3.0 * border_pair.right_lane_poly3.d * border_pair.ds_right_start;
    let d = border_pair.left_lane_poly3.d - border_pair.right_lane_poly3.d;
    let f = |x: f64| a + b * x + c * x * x + d * x * x * x;
    if f(0.0) < -FLOAT_COMPARISON_THRESHOLD || f(border_pair.ds_length) < -FLOAT_COMPARISON_THRESHOLD {
        return true;
    }
    // find real roots of f in [0, length]
    let mut roots = Vec::new();
    if d.abs() > 1e-12 {
        let disc = (c).powi(2) - 3.0 * b * d;
        if disc >= 0.0 {
            let sq = disc.sqrt();
            let r1 = (-c + sq) / (3.0 * d);
            let r2 = (-c - sq) / (3.0 * d);
            roots.push(r1);
            roots.push(r2);
        }
    } else if c.abs() > 1e-12 {
        roots.push(-b / (2.0 * c));
    }
    for r in roots {
        if r >= 0.0 && r <= border_pair.ds_length && f(r) < -FLOAT_COMPARISON_THRESHOLD {
            return true;
        }
    }
    false
}

fn check_overlap(
    cd: &mut CheckerData,
    left_lane: Node,
    right_lane: Node,
    lswl: crate::opendrive::models::LaneSectionWithLength,
    road: Node,
) {
    let id = "check_asam_xodr_road_lane_border_overlap_with_inner_lanes";
    let rule = "asam.net:xodr:1.4.0:road.lane.border.overlap_with_inner_lanes";
    let desc = "Outer lane border intersects or stays within inner lane border.";
    let left_lane_borders = utils::get_borders_from_lane(left_lane);
    let right_lane_borders = utils::get_borders_from_lane(right_lane);
    let border_pairs = create_border_pairs(&left_lane_borders, &right_lane_borders, lswl.length);
    for border_pair in &border_pairs {
        if intersect_or_stay_within(border_pair) {
            let issue_id = cd
                .result
                .register_issue(id, desc, IssueSeverity::Error, rule);
            add_location_pair(cd, id, issue_id, desc, left_lane, left_lane);
            add_location_pair(cd, id, issue_id, desc, right_lane, right_lane);
            let s_section = match utils::get_s_from_lane_section(lswl.lane_section) {
                Some(v) => v,
                None => return,
            };
            let s = s_section + lswl.length / 2.0;
            if let Some(p) = utils::get_middle_point_xyz_at_height_zero_from_lane_by_s(road, lswl.lane_section, left_lane, s) {
                cd.result
                    .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
            }
            if let Some(p) = utils::get_middle_point_xyz_at_height_zero_from_lane_by_s(road, lswl.lane_section, right_lane, s) {
                cd.result
                    .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
            }
            return;
        }
    }
}

fn check_overlap_among_lane_list(
    cd: &mut CheckerData,
    lanes: &[Node],
    lswl: crate::opendrive::models::LaneSectionWithLength,
    road: Node,
) {
    let mut sorted: Vec<Node> = lanes
        .iter()
        .cloned()
        .filter(|l| utils::get_lane_id(*l).is_some())
        .collect();
    sorted.sort_by_key(|l| utils::get_lane_id(*l).unwrap());
    for right_index in 0..sorted.len() {
        for left_index in (right_index + 1)..sorted.len() {
            check_overlap(cd, sorted[left_index], sorted[right_index], lswl, road);
        }
    }
}

fn check_road_border_overlap(cd: &mut CheckerData, road: Node) {
    for lswl in utils::get_sorted_lane_sections_with_length_from_road(road) {
        let left_lanes = utils::get_left_lanes_from_lane_section(lswl.lane_section);
        check_overlap_among_lane_list(cd, &left_lanes, lswl, road);
        let right_lanes = utils::get_right_lanes_from_lane_section(lswl.lane_section);
        check_overlap_among_lane_list(cd, &right_lanes, lswl, road);
    }
}

fn check_lane_border_overlap_with_inner_lanes(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    for road in utils::get_roads(doc) {
        check_road_border_overlap(cd, road);
    }
}

// ---------------------------------------------------------------------------
// road.geometry.parampoly3.* (length_match, arclength_range, normalized_range)
// ---------------------------------------------------------------------------

fn poly3_integral_length(u: &crate::opendrive::models::Poly3, v: &crate::opendrive::models::Poly3, a: f64, b: f64) -> f64 {
    let du = |x: f64| u.deriv(x);
    let dv = |x: f64| v.deriv(x);
    let integrand = |x: f64| (du(x).powi(2) + dv(x).powi(2)).sqrt();
    utils::adaptive_simpson(&integrand, a, b, 1e-10).0
}

fn check_parampoly3_length_match(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_geometry_parampoly3_length_match";
    let rule = "asam.net:xodr:1.7.0:road.geometry.parampoly3.length_match";
    let desc = "The actual curve length, as determined by numerical integration over the parameter range, should match '@Length'.";
    for road in utils::get_roads(doc) {
        for geometry in utils::get_road_plan_view_geometry_list(road) {
            let length = match utils::get_length_from_geometry(geometry) {
                Some(v) => v,
                None => continue,
            };
            let pp = match utils::get_normalized_param_poly3_from_geometry(geometry) {
                Some(v) => v,
                None => continue,
            };
            let integral = poly3_integral_length(&pp.u, &pp.v, 0.0, 1.0);
            if (integral - length).abs() > TOLERANCE_THRESHOLD {
                let issue_id = cd
                    .result
                    .register_issue(id, desc, IssueSeverity::Warning, rule);
                add_location_pair(cd, id, issue_id, desc, geometry, geometry);
                let s = match utils::get_s_from_geometry(geometry) {
                    Some(v) => v + length / 2.0,
                    None => continue,
                };
                if let Some(p) = utils::get_point_xyz_from_road_reference_line(road, s) {
                    cd.result
                        .add_inertial_location(id, issue_id, p.x, p.y, p.z, "Geometry where length doesn't match.");
                }
            }
        }
    }
}

fn check_parampoly3_arclength_range(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_geometry_parampoly3_arclength_range";
    let rule = "asam.net:xodr:1.7.0:road.geometry.parampoly3.arclength_range";
    let desc = "If @prange='arcLength', p shall be chosen in [0, @Length from geometry].";
    for road in utils::get_roads(doc) {
        for geometry in utils::get_road_plan_view_geometry_list(road) {
            let length = match utils::get_length_from_geometry(geometry) {
                Some(v) => v,
                None => continue,
            };
            let pp = match utils::get_arclen_param_poly3_from_geometry(geometry) {
                Some(v) => v,
                None => continue,
            };
            let integral = poly3_integral_length(&pp.u, &pp.v, 0.0, length);
            if (integral - length).abs() > TOLERANCE_THRESHOLD {
                let issue_id = cd
                    .result
                    .register_issue(id, desc, IssueSeverity::Error, rule);
                add_location_pair(cd, id, issue_id, desc, geometry, geometry);
                let s = match utils::get_s_from_geometry(geometry) {
                    Some(v) => v + length / 2.0,
                    None => continue,
                };
                if let Some(p) = utils::get_point_xyz_from_road_reference_line(road, s) {
                    cd.result
                        .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
                }
            }
        }
    }
}

fn check_parampoly3_normalized_range(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_geometry_parampoly3_normalized_range";
    let rule = "asam.net:xodr:1.7.0:road.geometry.parampoly3.normalized_range";
    let desc = "If @prange='normalized', p shall be chosen in [0, 1].";
    for road in utils::get_roads(doc) {
        for geometry in utils::get_road_plan_view_geometry_list(road) {
            let length = match utils::get_length_from_geometry(geometry) {
                Some(v) => v,
                None => continue,
            };
            let pp = match utils::get_normalized_param_poly3_from_geometry(geometry) {
                Some(v) => v,
                None => continue,
            };
            let integral = poly3_integral_length(&pp.u, &pp.v, 0.0, 1.0);
            if (integral - length).abs() > TOLERANCE_THRESHOLD {
                let issue_id = cd
                    .result
                    .register_issue(id, desc, IssueSeverity::Error, rule);
                add_location_pair(cd, id, issue_id, desc, geometry, geometry);
                let s = match utils::get_s_from_geometry(geometry) {
                    Some(v) => v + length / 2.0,
                    None => continue,
                };
                if let Some(p) = utils::get_point_xyz_from_road_reference_line(road, s) {
                    cd.result
                        .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// road.geometry.paramPoly3.valid_parameters
// ---------------------------------------------------------------------------

fn check_parampoly3_valid_parameters(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_geometry_parampoly3_valid_parameters";
    let rule = "asam.net:xodr:1.7.0:road.geometry.paramPoly3.valid_parameters";
    let desc = "The local u/v coordinate system should be aligned with the s/t coordinate system of the start point (meaning that the curve starts in the direction given by @hdg, and at the position given by @x and @y). To achieve this, the polynomial parameter coefficients have to be @aU=@aV=@bV=0, @bU>0.";
    for road in utils::get_roads(doc) {
        for geometry in utils::get_road_plan_view_geometry_list(road) {
            let length = match utils::get_length_from_geometry(geometry) {
                Some(_) => {}
                None => continue,
            };
            let _ = length;
            let mut pp: Option<Node> = None;
            for el in geometry.descendants().filter(|n| n.has_tag_name("paramPoly3")) {
                pp = Some(el);
            }
            let pp = match pp {
                Some(v) => v,
                None => continue,
            };
            let a_u = utils::to_float(pp.attribute("aU").unwrap_or("")).unwrap_or(0.0);
            let a_v = utils::to_float(pp.attribute("aV").unwrap_or("")).unwrap_or(0.0);
            let b_v = utils::to_float(pp.attribute("bV").unwrap_or("")).unwrap_or(0.0);
            let b_u = utils::to_float(pp.attribute("bU").unwrap_or("")).unwrap_or(0.0);
            if a_u.abs() > FLOAT_COMPARISON_THRESHOLD
                || a_v.abs() > FLOAT_COMPARISON_THRESHOLD
                || b_v.abs() > FLOAT_COMPARISON_THRESHOLD
                || !(b_u > FLOAT_COMPARISON_THRESHOLD)
            {
                add_issue(cd, id, rule, desc, IssueSeverity::Error, Some(pp), Some(pp), None);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Specs
// ---------------------------------------------------------------------------

pub fn specs() -> Vec<CheckerSpec> {
    vec![
        CheckerSpec {
            id: "check_asam_xodr_road_geometry_parampoly3_length_match",
            description: "ParamPoly3 length match.",
            rule_uid: "asam.net:xodr:1.7.0:road.geometry.parampoly3.length_match",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_parampoly3_length_match,
        },
        CheckerSpec {
            id: "check_asam_xodr_road_lane_border_overlap_with_inner_lanes",
            description: "Lane border overlap with inner lanes.",
            rule_uid: "asam.net:xodr:1.4.0:road.lane.border.overlap_with_inner_lanes",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_lane_border_overlap_with_inner_lanes,
        },
        CheckerSpec {
            id: "check_asam_xodr_road_geometry_parampoly3_arclength_range",
            description: "ParamPoly3 arclength range.",
            rule_uid: "asam.net:xodr:1.7.0:road.geometry.parampoly3.arclength_range",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_parampoly3_arclength_range,
        },
        CheckerSpec {
            id: "check_asam_xodr_road_geometry_parampoly3_normalized_range",
            description: "ParamPoly3 normalized range.",
            rule_uid: "asam.net:xodr:1.7.0:road.geometry.parampoly3.normalized_range",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_parampoly3_normalized_range,
        },
        CheckerSpec {
            id: "check_asam_xodr_road_geometry_contact_point",
            description: "Geometry contact point.",
            rule_uid: "asam.net:xodr:1.7.0:road.geometry.contact_point",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_geometry_contact_point,
        },
        CheckerSpec {
            id: "check_asam_xodr_road_geometry_elem_asc_order",
            description: "Geometry elements in ascending order.",
            rule_uid: "asam.net:xodr:1.4.0:road.geometry.elem_asc_order",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_geometry_elem_asc_order,
        },
        CheckerSpec {
            id: "check_asam_xodr_road_geometry_parampoly3_valid_parameters",
            description: "ParamPoly3 valid parameters.",
            rule_uid: "asam.net:xodr:1.7.0:road.geometry.paramPoly3.valid_parameters",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_parampoly3_valid_parameters,
        },
    ]
}
