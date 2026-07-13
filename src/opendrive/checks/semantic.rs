// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Semantic checkers (6-17), ported from qc_opendrive/checks/semantic/*.py.

use std::collections::HashMap;

use roxmltree::Node;

use crate::opendrive::checks::{add_location_pair, CheckerData, CheckerSpec};
use crate::opendrive::models::{ContactPoint, LaneDirection, LinkageTag, RoadLinkage, TrafficHandRule};
use crate::opendrive::utils;
use crate::result::IssueSeverity;

const FLOAT_COMPARISON_THRESHOLD: f64 = 1e-6;

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// road.lane.level_true_one_side
// ---------------------------------------------------------------------------

fn check_level_in_lane_section(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    for road in utils::get_roads(doc) {
        let lane_sections = utils::get_sorted_lane_sections_with_length_from_road(road);
        for lswl in &lane_sections {
            let ls = lswl.lane_section;
            let mut left: Vec<Node> = utils::get_left_lanes_from_lane_section(ls)
                .into_iter()
                .filter(|l| utils::get_lane_id(*l).is_some())
                .collect();
            let mut right: Vec<Node> = utils::get_right_lanes_from_lane_section(ls)
                .into_iter()
                .filter(|l| utils::get_lane_id(*l).is_some())
                .collect();
            left.sort_by_key(|l| utils::get_lane_id(*l).unwrap());
            right.sort_by_key(|l| utils::get_lane_id(*l).unwrap().abs());
            check_true_level_on_side(cd, &left, road, *lswl);
            check_true_level_on_side(cd, &right, road, *lswl);
        }
    }
}

fn check_true_level_on_side(
    cd: &mut CheckerData,
    side_lanes: &[Node],
    road: Node,
    lswl: crate::opendrive::models::LaneSectionWithLength,
) {
    let id = "check_asam_xodr_road_lane_level_true_one_side";
    let rule = "asam.net:xodr:1.7.0:road.lane.level_true_one_side";
    let mut found_true = false;
    for (index, lane) in side_lanes.iter().enumerate() {
        let lane_level = utils::get_lane_level_from_lane(*lane);
        if lane_level {
            found_true = true;
        } else if found_true {
            let desc = "Lane level False encountered on same side after True set.";
            let issue_id = cd
                .result
                .register_issue(id, desc, IssueSeverity::Error, rule);
            add_location_pair(cd, id, issue_id, desc, *lane, *lane);
            if let Some(s_section) = utils::get_s_from_lane_section(lswl.lane_section) {
                let s = s_section + lswl.length / 2.0;
                if let Some(p) = utils::get_middle_point_xyz_at_height_zero_from_lane_by_s(
                    road, lswl.lane_section, *lane, s,
                ) {
                    cd.result
                        .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
                }
            }
            // index unused beyond position; keep for parity with Python
            let _ = index;
        }
    }
}

fn get_linkage_level_warnings(
    doc: &roxmltree::Document,
    current_lane_section: Node,
    target_lane_section: Node,
    linkage_tag: LinkageTag,
) -> (Vec<String>, Vec<u32>) {
    let mut warnings: Vec<String> = Vec::new();
    let mut linenr: Vec<u32> = Vec::new();
    for lane in utils::get_left_and_right_lanes_from_lane_section(current_lane_section) {
        let lane_level = utils::get_lane_level_from_lane(lane);
        for link in lane.children().filter(|n| n.has_tag_name("link")) {
            for linkage in link.children().filter(|n| n.has_tag_name(linkage_tag.value())) {
                let linkage_id = match utils::to_int(linkage.attribute("id").unwrap_or("")) {
                    Some(v) => v,
                    None => continue,
                };
                let linkage_lane = utils::get_lane_from_lane_section(target_lane_section, linkage_id);
                if linkage_lane.is_none() {
                    continue;
                }
                let linkage_level = utils::get_lane_level_from_lane(linkage_lane.unwrap());
                if linkage_level != lane_level {
                    warnings.push(utils::node_xpath(lane));
                    linenr.push(cd_row(doc, lane));
                }
            }
        }
    }
    (warnings, linenr)
}

fn cd_row(doc: &roxmltree::Document, node: Node) -> u32 {
    utils::node_row(doc, node).unwrap_or(0)
}

fn check_level_change_between_lane_sections(cd: &mut CheckerData, current: Node, previous: Node) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_lane_level_true_one_side";
    let rule = "asam.net:xodr:1.7.0:road.lane.level_true_one_side";
    let desc = "Lane levels are not the same in two consecutive lane sections";
    let (mut pw, mut pl) = get_linkage_level_warnings(doc, current, previous, LinkageTag::Predecessor);
    let (sw, sl) = get_linkage_level_warnings(doc, previous, current, LinkageTag::Successor);
    pw.extend(sw);
    pl.extend(sl);
    let mut pairs: Vec<(String, u32)> = pw.into_iter().zip(pl).collect();
    pairs.sort_by_key(|(_, l)| *l);
    for (warning, warning_linenr) in pairs {
        let issue_id = cd
            .result
            .register_issue(id, desc, IssueSeverity::Warning, rule);
        cd.result
            .add_xml_location(id, issue_id, &warning, desc);
        cd.result
            .add_file_location(id, issue_id, Some(warning_linenr), Some(0), None, desc);
    }
}

fn check_level_change_linkage_roads(
    cd: &mut CheckerData,
    linkage_tag: LinkageTag,
    road: Node,
    road_id_map: &HashMap<i64, Node>,
) {
    let id = "check_asam_xodr_road_lane_level_true_one_side";
    let rule = "asam.net:xodr:1.7.0:road.lane.level_true_one_side";
    let desc = "Lane levels are not the same between two connected roads.";
    let current_lane_section = match linkage_tag {
        LinkageTag::Predecessor => utils::get_first_lane_section(road),
        LinkageTag::Successor => utils::get_last_lane_section(road),
    };
    let current_lane_section = match current_lane_section {
        Some(v) => v,
        None => return,
    };
    let all_lanes = utils::get_left_and_right_lanes_from_lane_section(current_lane_section);
    let linkage = match utils::get_road_linkage(road, linkage_tag) {
        Some(v) => v,
        None => return,
    };
    let other_lane_section = match utils::get_contact_lane_section_from_linked_road(&linkage, road_id_map) {
        Some(v) => v,
        None => return,
    };
    for lane in all_lanes {
        let lane_level = utils::get_lane_level_from_lane(lane);
        let linkage_lane_ids = match linkage_tag {
            LinkageTag::Predecessor => utils::get_predecessor_lane_ids(lane),
            LinkageTag::Successor => utils::get_successor_lane_ids(lane),
        };
        for lane_id in linkage_lane_ids {
            let other_lane = utils::get_lane_from_lane_section(other_lane_section.lane_section, lane_id);
            let other_lane = match other_lane {
                Some(v) => v,
                None => continue,
            };
            let other_lane_level = utils::get_lane_level_from_lane(other_lane);
            if other_lane_level != lane_level {
                let issue_id = cd
                    .result
                    .register_issue(id, desc, IssueSeverity::Warning, rule);
                add_location_pair(cd, id, issue_id, desc, lane, lane);
                add_location_pair(cd, id, issue_id, desc, other_lane, other_lane);
                let s = match linkage_tag {
                    LinkageTag::Predecessor => 0.0,
                    LinkageTag::Successor => match utils::get_road_length(road) {
                        Some(v) => v,
                        None => continue,
                    },
                };
                if let Some(p) = utils::get_middle_point_xyz_at_height_zero_from_lane_by_s(
                    road, current_lane_section, lane, s,
                ) {
                    cd.result
                        .add_inertial_location(id, issue_id, p.x, p.y, p.z, desc);
                }
            }
        }
    }
}

fn check_level_among_lane_sections(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    for road in utils::get_roads(doc) {
        let lane_sections = utils::get_lane_sections(road);
        if lane_sections.len() < 2 {
            continue;
        }
        for i in 1..lane_sections.len() {
            check_level_change_between_lane_sections(cd, lane_sections[i], lane_sections[i - 1]);
        }
    }
}

fn check_level_among_roads(cd: &mut CheckerData, road_id_map: &HashMap<i64, Node>) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    for road in utils::get_roads(doc) {
        check_level_change_linkage_roads(cd, LinkageTag::Predecessor, road, road_id_map);
        check_level_change_linkage_roads(cd, LinkageTag::Successor, road, road_id_map);
    }
}

fn check_level_among_junctions(cd: &mut CheckerData, road_id_map: &HashMap<i64, Node>) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_lane_level_true_one_side";
    let rule = "asam.net:xodr:1.7.0:road.lane.level_true_one_side";
    let desc = "Lane levels are not the same between incoming road and junction.";
    for junction in utils::get_junctions(doc) {
        for connection in utils::get_connections_from_junction(junction) {
            let cls = utils::get_incoming_and_connection_contacting_lane_sections(connection, road_id_map);
            let cls = match cls {
                Some(v) => v,
                None => continue,
            };
            for lane_link in utils::get_lane_links_from_connection(connection) {
                let incoming_lane_id = utils::get_from_attribute_from_lane_link(lane_link);
                let connection_lane_id = utils::get_to_attribute_from_lane_link(lane_link);
                let (incoming_lane_id, connection_lane_id) = match (incoming_lane_id, connection_lane_id) {
                    (Some(a), Some(b)) => (a, b),
                    _ => continue,
                };
                let incoming_lane = utils::get_lane_from_lane_section(cls.incoming, incoming_lane_id);
                let connection_lane = utils::get_lane_from_lane_section(cls.connection, connection_lane_id);
                let (incoming_lane, connection_lane) = match (incoming_lane, connection_lane) {
                    (Some(a), Some(b)) => (a, b),
                    _ => continue,
                };
                let incoming_level = utils::get_lane_level_from_lane(incoming_lane);
                let connection_level = utils::get_lane_level_from_lane(connection_lane);
                if incoming_level != connection_level {
                    let issue_id = cd
                        .result
                        .register_issue(id, desc, IssueSeverity::Warning, rule);
                    add_location_pair(cd, id, issue_id, desc, incoming_lane, incoming_lane);
                    add_location_pair(cd, id, issue_id, desc, connection_lane, connection_lane);
                }
            }
        }
    }
}

fn check_lane_level_true_one_side(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let road_id_map = utils::get_road_id_map(doc);
    check_level_in_lane_section(cd);
    check_level_among_lane_sections(cd);
    check_level_among_roads(cd, &road_id_map);
    check_level_among_junctions(cd, &road_id_map);
}

// ---------------------------------------------------------------------------
// road.lane.access.no_mix_of_deny_or_allow
// ---------------------------------------------------------------------------

fn check_lane_access_no_mix(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_lane_access_no_mix_of_deny_or_allow";
    let rule = "asam.net:xodr:1.7.0:road.lane.access.no_mix_of_deny_or_allow";
    let desc = "At a given s-position, either only deny or only allow values shall be given, not mixed.";
    for road in utils::get_roads(doc) {
        for lswl in utils::get_sorted_lane_sections_with_length_from_road(road) {
            let ls = lswl.lane_section;
            let length = lswl.length;
            let lanes = utils::get_left_and_right_lanes_from_lane_section(ls);
            let s_section = utils::get_s_from_lane_section(ls);
            for lane in lanes {
                let mut info: Vec<(f64, String)> = Vec::new();
                for access in lane.descendants().filter(|n| n.has_tag_name("access")) {
                    let rule_attr = match access.attribute("rule") {
                        Some(v) => v.to_string(),
                        None => continue,
                    };
                    let s_offset = match utils::get_s_offset_from_access(access) {
                        Some(v) => v,
                        None => continue,
                    };
                    for (prev_offset, prev_rule) in &info {
                        if (prev_offset - s_offset).abs() <= 1e-6 && rule_attr != *prev_rule {
                            let issue_id = cd
                                .result
                                .register_issue(id, desc, IssueSeverity::Error, rule);
                            add_location_pair(cd, id, issue_id, &format!("First encounter of {rule_attr} having {prev_rule} before."), access, access);
                            if let Some(s_section) = s_section {
                                let s = s_section + s_offset + (length - s_offset) / 2.0;
                                let t = utils::get_t_middle_point_from_lane_by_s(road, ls, lane, s);
                                if let Some(t) = t {
                                    if let Some(p) = utils::get_point_xyz_from_road(road, s, t, 0.0) {
                                        cd.result.add_inertial_location(
                                            id, issue_id, p.x, p.y, p.z, "Mixed access point.",
                                        );
                                    }
                                }
                            }
                        }
                    }
                    info.push((s_offset, rule_attr));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// road.lane.link.lanes_across_lane_sections
// ---------------------------------------------------------------------------

fn check_two_lane_sections_one_direction(
    cd: &mut CheckerData,
    first: crate::opendrive::models::ContactingLaneSection,
    second: crate::opendrive::models::ContactingLaneSection,
) {
    let id = "check_asam_xodr_road_lane_link_lanes_across_lane_sections";
    let rule = "asam.net:xodr:1.4.0:road.lane.link.lanes_across_lane_sections";
    let desc = "Missing lane link.";
    for lane in utils::get_left_and_right_lanes_from_lane_section(first.lane_section) {
        let current_lane_id = match utils::get_lane_id(lane) {
            Some(v) => v,
            None => continue,
        };
        let connecting_ids = utils::get_connecting_lane_ids(lane, first.linkage_tag);
        for connecting_id in connecting_ids {
            let connecting_lane = utils::get_lane_from_lane_section(second.lane_section, connecting_id);
            let connecting_lane = match connecting_lane {
                Some(v) => v,
                None => continue,
            };
            let second_ids = utils::get_connecting_lane_ids(connecting_lane, second.linkage_tag);
            if !second_ids.contains(&current_lane_id) {
                let issue_id = cd
                    .result
                    .register_issue(id, desc, IssueSeverity::Error, rule);
                add_location_pair(cd, id, issue_id, desc, connecting_lane, connecting_lane);
            }
        }
    }
}

fn check_two_lane_sections(
    cd: &mut CheckerData,
    first: crate::opendrive::models::ContactingLaneSection,
    second: crate::opendrive::models::ContactingLaneSection,
) {
    check_two_lane_sections_one_direction(cd, first, second);
    check_two_lane_sections_one_direction(cd, second, first);
}

fn check_middle_lane_sections(cd: &mut CheckerData, road: Node) {
    let lane_sections = utils::get_lane_sections(road);
    if lane_sections.len() < 2 {
        return;
    }
    for i in 1..lane_sections.len() {
        check_two_lane_sections(
            cd,
            crate::opendrive::models::ContactingLaneSection {
                lane_section: lane_sections[i - 1],
                linkage_tag: LinkageTag::Successor,
            },
            crate::opendrive::models::ContactingLaneSection {
                lane_section: lane_sections[i],
                linkage_tag: LinkageTag::Predecessor,
            },
        );
    }
}

fn check_first_lane_section(cd: &mut CheckerData, road: Node, road_id_map: &HashMap<i64, Node>) {
    let first = match utils::get_first_lane_section(road) {
        Some(v) => v,
        None => return,
    };
    let first_contacting = crate::opendrive::models::ContactingLaneSection {
        lane_section: first,
        linkage_tag: LinkageTag::Predecessor,
    };
    let predecessor_linkage = match utils::get_road_linkage(road, LinkageTag::Predecessor) {
        Some(v) => v,
        None => return,
    };
    let other = match utils::get_contact_lane_section_from_linked_road(&predecessor_linkage, road_id_map) {
        Some(v) => v,
        None => return,
    };
    check_two_lane_sections(cd, first_contacting, other);
}

fn check_last_lane_section(cd: &mut CheckerData, road: Node, road_id_map: &HashMap<i64, Node>) {
    let last = match utils::get_last_lane_section(road) {
        Some(v) => v,
        None => return,
    };
    let last_contacting = crate::opendrive::models::ContactingLaneSection {
        lane_section: last,
        linkage_tag: LinkageTag::Successor,
    };
    let successor_linkage = match utils::get_road_linkage(road, LinkageTag::Successor) {
        Some(v) => v,
        None => return,
    };
    let other = match utils::get_contact_lane_section_from_linked_road(&successor_linkage, road_id_map) {
        Some(v) => v,
        None => return,
    };
    check_two_lane_sections(cd, last_contacting, other);
}

fn check_lanes_across_lane_sections(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let road_id_map = utils::get_road_id_map(doc);
    for road in utils::get_roads(doc) {
        check_middle_lane_sections(cd, road);
        if !utils::road_belongs_to_junction(road) {
            check_first_lane_section(cd, road, &road_id_map);
            check_last_lane_section(cd, road, &road_id_map);
        }
    }
}

// ---------------------------------------------------------------------------
// road.linkage.is_junction_needed
// ---------------------------------------------------------------------------

fn create_contact_point_id(linkage: &RoadLinkage) -> String {
    format!("{}-{}", linkage.id, linkage.contact_point.value())
}

fn check_road_linkage_is_junction_needed(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_linkage_is_junction_needed";
    let rule = "asam.net:xodr:1.4.0:road.linkage.is_junction_needed";
    let road_id_map = utils::get_road_id_map(doc);
    if road_id_map.len() < 2 {
        return;
    }
    let mut contact_point_map: HashMap<String, Vec<Node>> = HashMap::new();
    for (_rid, road) in &road_id_map {
        if utils::road_belongs_to_junction(*road) {
            continue;
        }
        if let Some(pred) = utils::get_road_linkage(*road, LinkageTag::Predecessor) {
            let cid = create_contact_point_id(&pred);
            let link = utils::get_road_link_element(*road, pred.id, LinkageTag::Predecessor);
            if let Some(link) = link {
                contact_point_map.entry(cid).or_default().push(link);
            }
        }
        if let Some(succ) = utils::get_road_linkage(*road, LinkageTag::Successor) {
            let cid = create_contact_point_id(&succ);
            let link = utils::get_road_link_element(*road, succ.id, LinkageTag::Successor);
            if let Some(link) = link {
                contact_point_map.entry(cid).or_default().push(link);
            }
        }
    }
    for (cid, elements) in &contact_point_map {
        if elements.len() > 1 {
            let parts: Vec<&str> = cid.split('-').collect();
            let linkage = RoadLinkage {
                id: parts[0].parse().unwrap_or(0),
                contact_point: if parts[1] == "end" {
                    ContactPoint::End
                } else {
                    ContactPoint::Start
                },
            };
            let linkage_tag = match linkage.contact_point {
                ContactPoint::End => LinkageTag::Successor,
                ContactPoint::Start => LinkageTag::Predecessor,
            };
            let desc = format!(
                "Road cannot have ambiguous {} a junction is needed.",
                linkage_tag.value()
            );
            let issue_id = cd
                .result
                .register_issue(id, &desc, IssueSeverity::Error, rule);
            for el in elements {
                add_location_pair(cd, id, issue_id, &desc, *el, *el);
            }
            if let Some(problematic_road) = road_id_map.get(&linkage.id) {
                let p = match linkage_tag {
                    LinkageTag::Predecessor => utils::get_start_point_xyz_from_road_reference_line(*problematic_road),
                    LinkageTag::Successor => utils::get_end_point_xyz_from_road_reference_line(*problematic_road),
                };
                if let Some(p) = p {
                    cd.result
                        .add_inertial_location(id, issue_id, p.x, p.y, p.z, "Point where the linkage is not clear.");
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// road.lane.link.zero_width_at_start / _end
// ---------------------------------------------------------------------------

fn raise_zero_width_issue(
    cd: &mut CheckerData,
    id: &str,
    rule: &str,
    road: Node,
    lane_section: Node,
    lane: Node,
    severity: IssueSeverity,
) {
    let desc = " Lanes that have a width of zero at the beginning of the lane section shall have no predecessor element.";
    let issue_id = cd.result.register_issue(id, desc, severity, rule);
    add_location_pair(cd, id, issue_id, "Lane with width zero and predecessors.", lane, lane);
    let s = match utils::get_s_from_lane_section(lane_section) {
        Some(v) => v,
        None => return,
    };
    if let Some(p) = utils::get_middle_point_xyz_at_height_zero_from_lane_by_s(road, lane_section, lane, s) {
        cd.result
            .add_inertial_location(id, issue_id, p.x, p.y, p.z, "Lane with width zero and predecessors.");
    }
}

fn raise_zero_width_end_issue(
    cd: &mut CheckerData,
    id: &str,
    rule: &str,
    road: Node,
    lswl: crate::opendrive::models::LaneSectionWithLength,
    lane: Node,
    severity: IssueSeverity,
) {
    let desc = " Lanes that have a width of zero at the end of the lane section shall have no successor element.";
    let issue_id = cd.result.register_issue(id, desc, severity, rule);
    add_location_pair(cd, id, issue_id, "Lane with width zero and successors.", lane, lane);
    let s_section = match utils::get_s_from_lane_section(lswl.lane_section) {
        Some(v) => v,
        None => return,
    };
    let s = s_section + lswl.length;
    if let Some(p) = utils::get_middle_point_xyz_at_height_zero_from_lane_by_s(road, lswl.lane_section, lane, s) {
        cd.result
            .add_inertial_location(id, issue_id, p.x, p.y, p.z, "Lane with width zero and successors.");
    }
}

fn check_road_lane_link_zero_width_at_start(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_lane_link_zero_width_at_start";
    let rule = "asam.net:xodr:1.7.0:road.lane.link.zero_width_at_start";
    for road in utils::get_roads(doc) {
        for section in utils::get_lane_sections(road) {
            for lane in utils::get_left_and_right_lanes_from_lane_section(section) {
                let start_width = match utils::evaluate_lane_width(lane, 0.0) {
                    Some(v) => v,
                    None => continue,
                };
                if start_width < FLOAT_COMPARISON_THRESHOLD {
                    let predecessor_ids = utils::get_predecessor_lane_ids(lane);
                    if !predecessor_ids.is_empty() {
                        let lane_id = utils::get_lane_id(lane).unwrap_or(0);
                        let sev = if lane_id == 0 {
                            IssueSeverity::Warning
                        } else {
                            IssueSeverity::Error
                        };
                        raise_zero_width_issue(cd, id, rule, road, section, lane, sev);
                    }
                }
            }
        }
    }
}

fn check_incoming_road_junction_predecessor_lane_width_zero(
    cd: &mut CheckerData,
    road: Node,
    road_id: i64,
    road_id_map: &HashMap<i64, Node>,
    junction_id_map: &HashMap<i64, Node>,
) {
    let id = "check_asam_xodr_road_lane_link_zero_width_at_start";
    let rule = "asam.net:xodr:1.7.0:road.lane.link.zero_width_at_start";
    let predecessor_junction_id = match utils::get_linked_junction_id(road, LinkageTag::Predecessor) {
        Some(v) => v,
        None => return,
    };
    let predecessor_connections = utils::get_connections_between_road_and_junction(
        road_id, predecessor_junction_id, road_id_map, junction_id_map, ContactPoint::Start,
    );
    let mut lane_ids_with_predecessor: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for connection in &predecessor_connections {
        for lane_link in utils::get_lane_links_from_connection(*connection) {
            if let Some(from_id) = utils::get_from_attribute_from_lane_link(lane_link) {
                lane_ids_with_predecessor.insert(from_id);
            }
        }
    }
    let first = match utils::get_first_lane_section(road) {
        Some(v) => v,
        None => return,
    };
    for lane in utils::get_left_and_right_lanes_from_lane_section(first) {
        let start_width = match utils::evaluate_lane_width(lane, 0.0) {
            Some(v) => v,
            None => continue,
        };
        if start_width < FLOAT_COMPARISON_THRESHOLD {
            let lane_id = match utils::get_lane_id(lane) {
                Some(v) => v,
                None => continue,
            };
            if lane_ids_with_predecessor.contains(&lane_id) {
                let sev = if lane_id == 0 {
                    IssueSeverity::Warning
                } else {
                    IssueSeverity::Error
                };
                raise_zero_width_issue(cd, id, rule, road, first, lane, sev);
            }
        }
    }
}

fn check_connecting_road_lane_width_zero_with_predecessor(
    cd: &mut CheckerData,
    road: Node,
    road_id: i64,
    junction_id_map: &HashMap<i64, Node>,
) {
    let id = "check_asam_xodr_road_lane_link_zero_width_at_start";
    let rule = "asam.net:xodr:1.7.0:road.lane.link.zero_width_at_start";
    let road_junction_id = match utils::get_road_junction_id(road) {
        Some(v) => v,
        None => return,
    };
    let junction = match junction_id_map.get(&road_junction_id) {
        Some(v) => *v,
        None => return,
    };
    let predecessor_connections = utils::get_connections_of_connecting_road(road_id, junction, ContactPoint::Start);
    let mut lane_ids_with_predecessor: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for connection in &predecessor_connections {
        for lane_link in utils::get_lane_links_from_connection(*connection) {
            if let Some(to_id) = utils::get_to_attribute_from_lane_link(lane_link) {
                lane_ids_with_predecessor.insert(to_id);
            }
        }
    }
    let first = match utils::get_first_lane_section(road) {
        Some(v) => v,
        None => return,
    };
    for lane in utils::get_left_and_right_lanes_from_lane_section(first) {
        let start_width = match utils::evaluate_lane_width(lane, 0.0) {
            Some(v) => v,
            None => continue,
        };
        if start_width < FLOAT_COMPARISON_THRESHOLD {
            let lane_id = match utils::get_lane_id(lane) {
                Some(v) => v,
                None => continue,
            };
            if lane_ids_with_predecessor.contains(&lane_id) {
                let sev = if lane_id == 0 {
                    IssueSeverity::Warning
                } else {
                    IssueSeverity::Error
                };
                raise_zero_width_issue(cd, id, rule, road, first, lane, sev);
            }
        }
    }
}

fn check_junction_road_lane_link_zero_width_at_start(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let road_id_map = utils::get_road_id_map(doc);
    let junction_id_map = utils::get_junction_id_map(doc);
    for (road_id, road) in &road_id_map {
        if utils::road_belongs_to_junction(*road) {
            check_connecting_road_lane_width_zero_with_predecessor(cd, *road, *road_id, &junction_id_map);
        } else {
            check_incoming_road_junction_predecessor_lane_width_zero(
                cd, *road, *road_id, &road_id_map, &junction_id_map,
            );
        }
    }
}

fn check_road_lane_link_zero_width_at_end(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_road_lane_link_zero_width_at_end";
    let rule = "asam.net:xodr:1.7.0:road.lane.link.zero_width_at_end";
    for road in utils::get_roads(doc) {
        for lswl in utils::get_sorted_lane_sections_with_length_from_road(road) {
            for lane in utils::get_left_and_right_lanes_from_lane_section(lswl.lane_section) {
                let end_width = match utils::evaluate_lane_width(lane, lswl.length) {
                    Some(v) => v,
                    None => continue,
                };
                if end_width.abs() < FLOAT_COMPARISON_THRESHOLD {
                    let successor_ids = utils::get_successor_lane_ids(lane);
                    if !successor_ids.is_empty() {
                        let lane_id = utils::get_lane_id(lane).unwrap_or(0);
                        let sev = if lane_id == 0 {
                            IssueSeverity::Warning
                        } else {
                            IssueSeverity::Error
                        };
                        raise_zero_width_end_issue(cd, id, rule, road, lswl, lane, sev);
                    }
                }
            }
        }
    }
}

fn check_incoming_road_junction_successor_lane_width_zero(
    cd: &mut CheckerData,
    road: Node,
    road_id: i64,
    road_id_map: &HashMap<i64, Node>,
    junction_id_map: &HashMap<i64, Node>,
) {
    let id = "check_asam_xodr_road_lane_link_zero_width_at_end";
    let rule = "asam.net:xodr:1.7.0:road.lane.link.zero_width_at_end";
    let successor_junction_id = match utils::get_linked_junction_id(road, LinkageTag::Successor) {
        Some(v) => v,
        None => return,
    };
    let successor_connections = utils::get_connections_between_road_and_junction(
        road_id, successor_junction_id, road_id_map, junction_id_map, ContactPoint::End,
    );
    let mut lane_ids_with_successor: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for connection in &successor_connections {
        for lane_link in utils::get_lane_links_from_connection(*connection) {
            if let Some(from_id) = utils::get_from_attribute_from_lane_link(lane_link) {
                lane_ids_with_successor.insert(from_id);
            }
        }
    }
    let lswl_list = utils::get_sorted_lane_sections_with_length_from_road(road);
    if lswl_list.is_empty() {
        return;
    }
    let last = lswl_list[lswl_list.len() - 1];
    for lane in utils::get_left_and_right_lanes_from_lane_section(last.lane_section) {
        let end_width = match utils::evaluate_lane_width(lane, last.length) {
            Some(v) => v,
            None => continue,
        };
        if end_width.abs() < FLOAT_COMPARISON_THRESHOLD {
            let lane_id = match utils::get_lane_id(lane) {
                Some(v) => v,
                None => continue,
            };
            if lane_ids_with_successor.contains(&lane_id) {
                let sev = if lane_id == 0 {
                    IssueSeverity::Warning
                } else {
                    IssueSeverity::Error
                };
                raise_zero_width_end_issue(cd, id, rule, road, last, lane, sev);
            }
        }
    }
}

fn check_connecting_road_lane_width_zero_with_successor(
    cd: &mut CheckerData,
    road: Node,
    road_id: i64,
    junction_id_map: &HashMap<i64, Node>,
) {
    let id = "check_asam_xodr_road_lane_link_zero_width_at_end";
    let rule = "asam.net:xodr:1.7.0:road.lane.link.zero_width_at_end";
    let road_junction_id = match utils::get_road_junction_id(road) {
        Some(v) => v,
        None => return,
    };
    let junction = match junction_id_map.get(&road_junction_id) {
        Some(v) => *v,
        None => return,
    };
    let successor_connections = utils::get_connections_of_connecting_road(road_id, junction, ContactPoint::End);
    let mut lane_ids_with_successor: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for connection in &successor_connections {
        for lane_link in utils::get_lane_links_from_connection(*connection) {
            if let Some(to_id) = utils::get_to_attribute_from_lane_link(lane_link) {
                lane_ids_with_successor.insert(to_id);
            }
        }
    }
    let lswl_list = utils::get_sorted_lane_sections_with_length_from_road(road);
    if lswl_list.is_empty() {
        return;
    }
    let last = lswl_list[lswl_list.len() - 1];
    for lane in utils::get_left_and_right_lanes_from_lane_section(last.lane_section) {
        let end_width = match utils::evaluate_lane_width(lane, last.length) {
            Some(v) => v,
            None => continue,
        };
        if end_width.abs() < FLOAT_COMPARISON_THRESHOLD {
            let lane_id = match utils::get_lane_id(lane) {
                Some(v) => v,
                None => continue,
            };
            if lane_ids_with_successor.contains(&lane_id) {
                let sev = if lane_id == 0 {
                    IssueSeverity::Warning
                } else {
                    IssueSeverity::Error
                };
                raise_zero_width_end_issue(cd, id, rule, road, last, lane, sev);
            }
        }
    }
}

fn check_junction_road_lane_link_zero_width_at_end(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let road_id_map = utils::get_road_id_map(doc);
    let junction_id_map = utils::get_junction_id_map(doc);
    for (road_id, road) in &road_id_map {
        if utils::road_belongs_to_junction(*road) {
            check_connecting_road_lane_width_zero_with_successor(cd, *road, *road_id, &junction_id_map);
        } else {
            check_incoming_road_junction_successor_lane_width_zero(
                cd, *road, *road_id, &road_id_map, &junction_id_map,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// road.lane.link.new_lane_appear
// ---------------------------------------------------------------------------

fn raise_new_lane_issue(
    cd: &mut CheckerData,
    lane: Node,
    width_zero_lane: Node,
    severity: IssueSeverity,
    linkage_tag: LinkageTag,
) {
    let id = "check_asam_xodr_road_lane_link_new_lane_appear";
    let rule = "asam.net:xodr:1.4.0:road.lane.link.new_lane_appear";
    let desc = "If a new lane appears besides, only the continuing lane shall be connected to the original lane, not the appearing lane.";
    let issue_id = cd.result.register_issue(id, desc, severity, rule);
    add_location_pair(cd, id, issue_id, &format!("Lane with {} with width zero.", linkage_tag.value()), lane, lane);
    add_location_pair(
        cd, id, issue_id,
        &format!("{} lane with width zero.", capitalize_first(linkage_tag.value())),
        width_zero_lane, width_zero_lane,
    );
}

fn check_successor_with_width_zero_between_lane_sections(
    cd: &mut CheckerData,
    current_lane_section: Node,
    next_lane_section: Node,
    contact_point: ContactPoint,
    next_lane_section_length: f64,
) {
    for lane in utils::get_left_and_right_lanes_from_lane_section(current_lane_section) {
        for successor_lane_id in utils::get_successor_lane_ids(lane) {
            let successor_lane = utils::get_lane_from_lane_section(next_lane_section, successor_lane_id);
            let successor_lane = match successor_lane {
                Some(v) => v,
                None => continue,
            };
            let target_width = match contact_point {
                ContactPoint::Start => utils::evaluate_lane_width(successor_lane, 0.0),
                ContactPoint::End => utils::evaluate_lane_width(successor_lane, next_lane_section_length),
            };
            if let Some(w) = target_width {
                if w.abs() < FLOAT_COMPARISON_THRESHOLD {
                    raise_new_lane_issue(cd, lane, successor_lane, IssueSeverity::Error, LinkageTag::Successor);
                }
            }
        }
    }
}

fn check_predecessor_with_width_zero_between_lane_sections(
    cd: &mut CheckerData,
    current_lane_section: Node,
    next_lane_section: Node,
    contact_point: ContactPoint,
    next_lane_section_length: f64,
) {
    for lane in utils::get_left_and_right_lanes_from_lane_section(current_lane_section) {
        let lane_id = match utils::get_lane_id(lane) {
            Some(v) => v,
            None => continue,
        };
        let _ = lane_id;
        for predecessor_lane_id in utils::get_predecessor_lane_ids(lane) {
            let predecessor_lane = utils::get_lane_from_lane_section(next_lane_section, predecessor_lane_id);
            let predecessor_lane = match predecessor_lane {
                Some(v) => v,
                None => continue,
            };
            let target_width = match contact_point {
                ContactPoint::Start => utils::evaluate_lane_width(predecessor_lane, 0.0),
                ContactPoint::End => utils::evaluate_lane_width(predecessor_lane, next_lane_section_length),
            };
            if let Some(w) = target_width {
                if w.abs() < FLOAT_COMPARISON_THRESHOLD {
                    raise_new_lane_issue(cd, lane, predecessor_lane, IssueSeverity::Error, LinkageTag::Predecessor);
                }
            }
        }
    }
}

fn check_appearing_successor_with_width_zero_on_road(cd: &mut CheckerData, road: Node) {
    let lane_sections = utils::get_sorted_lane_sections_with_length_from_road(road);
    if lane_sections.len() < 2 {
        return;
    }
    for i in 0..lane_sections.len() - 1 {
        check_successor_with_width_zero_between_lane_sections(
            cd,
            lane_sections[i].lane_section,
            lane_sections[i + 1].lane_section,
            ContactPoint::Start,
            lane_sections[i + 1].length,
        );
    }
}

fn check_appearing_successor_road(
    cd: &mut CheckerData,
    road_id_map: &HashMap<i64, Node>,
    current_road_id: i64,
    successor_road_id: i64,
) {
    let current_road = match road_id_map.get(&current_road_id) {
        Some(v) => *v,
        None => return,
    };
    let successor_road = match road_id_map.get(&successor_road_id) {
        Some(v) => *v,
        None => return,
    };
    let current_last = match utils::get_last_lane_section(current_road) {
        Some(v) => v,
        None => return,
    };
    let successor_linkage = match utils::get_road_linkage(current_road, LinkageTag::Successor) {
        Some(v) => v,
        None => return,
    };
    let target = match utils::get_contact_lane_section_from_linked_road(&successor_linkage, road_id_map) {
        Some(v) => v,
        None => return,
    };
    let next_length;
    let succ_lswl = utils::get_sorted_lane_sections_with_length_from_road(successor_road);
    if successor_linkage.contact_point == ContactPoint::Start {
        next_length = succ_lswl[0].length;
    } else {
        next_length = succ_lswl[succ_lswl.len() - 1].length;
    }
    check_successor_with_width_zero_between_lane_sections(
        cd, current_last, target.lane_section, successor_linkage.contact_point, next_length,
    );
}

fn check_appearing_predecessor_road(
    cd: &mut CheckerData,
    road_id_map: &HashMap<i64, Node>,
    current_road_id: i64,
    predecessor_road_id: i64,
) {
    let current_road = match road_id_map.get(&current_road_id) {
        Some(v) => *v,
        None => return,
    };
    let predecessor_road = match road_id_map.get(&predecessor_road_id) {
        Some(v) => *v,
        None => return,
    };
    let current_first = match utils::get_first_lane_section(current_road) {
        Some(v) => v,
        None => return,
    };
    let predecessor_linkage = match utils::get_road_linkage(current_road, LinkageTag::Predecessor) {
        Some(v) => v,
        None => return,
    };
    let target = match utils::get_contact_lane_section_from_linked_road(&predecessor_linkage, road_id_map) {
        Some(v) => v,
        None => return,
    };
    let next_length;
    let pred_lswl = utils::get_sorted_lane_sections_with_length_from_road(predecessor_road);
    if predecessor_linkage.contact_point == ContactPoint::Start {
        next_length = pred_lswl[0].length;
    } else {
        next_length = pred_lswl[pred_lswl.len() - 1].length;
    }
    check_predecessor_with_width_zero_between_lane_sections(
        cd, current_first, target.lane_section, predecessor_linkage.contact_point, next_length,
    );
}

fn check_appearing_successor_junction(
    cd: &mut CheckerData,
    junction_id_map: &HashMap<i64, Node>,
    road_id_map: &HashMap<i64, Node>,
    road_id: i64,
    successor_junction_id: i64,
) {
    let successor_connections = utils::get_connections_between_road_and_junction(
        road_id, successor_junction_id, road_id_map, junction_id_map, ContactPoint::End,
    );
    for connection in successor_connections {
        let connecting_road_id = match utils::get_connecting_road_id_from_connection(connection) {
            Some(v) => v,
            None => continue,
        };
        let connection_road = match road_id_map.get(&connecting_road_id) {
            Some(v) => *v,
            None => continue,
        };
        let cls = match utils::get_incoming_and_connection_contacting_lane_sections(connection, road_id_map) {
            Some(v) => v,
            None => continue,
        };
        let connection_contact_point = match utils::get_contact_point_from_connection(connection) {
            Some(v) => v,
            None => continue,
        };
        for lane_link in utils::get_lane_links_from_connection(connection) {
            let from_lane_id = match utils::get_from_attribute_from_lane_link(lane_link) {
                Some(v) => v,
                None => continue,
            };
            let to_lane_id = match utils::get_to_attribute_from_lane_link(lane_link) {
                Some(v) => v,
                None => continue,
            };
            let connection_lane = utils::get_lane_from_lane_section(cls.connection, to_lane_id);
            let connection_lane = match connection_lane {
                Some(v) => v,
                None => continue,
            };
            let connection_lane_contact_width = match connection_contact_point {
                ContactPoint::Start => utils::evaluate_lane_width(connection_lane, 0.0),
                ContactPoint::End => {
                    let connection_road_length = match utils::get_road_length(connection_road) {
                        Some(v) => v,
                        None => continue,
                    };
                    let s_cls = match utils::get_s_from_lane_section(cls.connection) {
                        Some(v) => v,
                        None => continue,
                    };
                    utils::evaluate_lane_width(connection_lane, connection_road_length - s_cls)
                }
            };
            if let Some(w) = connection_lane_contact_width {
                if w.abs() < FLOAT_COMPARISON_THRESHOLD {
                    let current_road_lane = utils::get_lane_from_lane_section(cls.incoming, from_lane_id);
                    let current_road_lane = match current_road_lane {
                        Some(v) => v,
                        None => continue,
                    };
                    raise_new_lane_issue(cd, current_road_lane, connection_lane, IssueSeverity::Error, LinkageTag::Successor);
                }
            }
        }
    }
}

fn check_appearing_predecessor_junction(
    cd: &mut CheckerData,
    junction_id_map: &HashMap<i64, Node>,
    road_id_map: &HashMap<i64, Node>,
    road_id: i64,
    predecessor_junction_id: i64,
) {
    let predecessor_connections = utils::get_connections_between_road_and_junction(
        road_id, predecessor_junction_id, road_id_map, junction_id_map, ContactPoint::Start,
    );
    for connection in predecessor_connections {
        let connecting_road_id = match utils::get_connecting_road_id_from_connection(connection) {
            Some(v) => v,
            None => continue,
        };
        let connection_road = match road_id_map.get(&connecting_road_id) {
            Some(v) => *v,
            None => continue,
        };
        let cls = match utils::get_incoming_and_connection_contacting_lane_sections(connection, road_id_map) {
            Some(v) => v,
            None => continue,
        };
        let connection_contact_point = match utils::get_contact_point_from_connection(connection) {
            Some(v) => v,
            None => continue,
        };
        for lane_link in utils::get_lane_links_from_connection(connection) {
            let from_lane_id = match utils::get_from_attribute_from_lane_link(lane_link) {
                Some(v) => v,
                None => continue,
            };
            let to_lane_id = match utils::get_to_attribute_from_lane_link(lane_link) {
                Some(v) => v,
                None => continue,
            };
            let connection_lane = utils::get_lane_from_lane_section(cls.connection, to_lane_id);
            let connection_lane = match connection_lane {
                Some(v) => v,
                None => continue,
            };
            let connection_lane_contact_width = match connection_contact_point {
                ContactPoint::Start => utils::evaluate_lane_width(connection_lane, 0.0),
                ContactPoint::End => {
                    let connection_road_length = match utils::get_road_length(connection_road) {
                        Some(v) => v,
                        None => continue,
                    };
                    let s_cls = match utils::get_s_from_lane_section(cls.connection) {
                        Some(v) => v,
                        None => continue,
                    };
                    utils::evaluate_lane_width(connection_lane, connection_road_length - s_cls)
                }
            };
            if let Some(w) = connection_lane_contact_width {
                if w.abs() < FLOAT_COMPARISON_THRESHOLD {
                    let current_road_lane = utils::get_lane_from_lane_section(cls.incoming, from_lane_id);
                    let current_road_lane = match current_road_lane {
                        Some(v) => v,
                        None => continue,
                    };
                    raise_new_lane_issue(cd, current_road_lane, connection_lane, IssueSeverity::Error, LinkageTag::Predecessor);
                }
            }
        }
    }
}

fn check_road_lane_link_new_lane_appear(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let road_id_map = utils::get_road_id_map(doc);
    let junction_id_map = utils::get_junction_id_map(doc);
    for (road_id, road) in &road_id_map {
        check_appearing_successor_with_width_zero_on_road(cd, *road);
        if let Some(successor_road_id) = utils::get_successor_road_id(*road) {
            check_appearing_successor_road(cd, &road_id_map, *road_id, successor_road_id);
        }
        if let Some(predecessor_road_id) = utils::get_predecessor_road_id(*road) {
            check_appearing_predecessor_road(cd, &road_id_map, *road_id, predecessor_road_id);
        }
        if let Some(successor_junction_id) = utils::get_linked_junction_id(*road, LinkageTag::Successor) {
            check_appearing_successor_junction(cd, &junction_id_map, &road_id_map, *road_id, successor_junction_id);
        }
        if let Some(predecessor_junction_id) = utils::get_linked_junction_id(*road, LinkageTag::Predecessor) {
            check_appearing_predecessor_junction(cd, &junction_id_map, &road_id_map, *road_id, predecessor_junction_id);
        }
    }
}

// ---------------------------------------------------------------------------
// junctions.connection.connect_road_no_incoming_road
// ---------------------------------------------------------------------------

fn check_junctions_connection_connect_road_no_incoming_road(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_junctions_connection_connect_road_no_incoming_road";
    let rule = "asam.net:xodr:1.4.0:junctions.connection.connect_road_no_incoming_road";
    let desc = "Connecting roads shall not be incoming roads.";
    let road_id_map = utils::get_road_id_map(doc);
    for junction in utils::get_junctions(doc) {
        for connection in utils::get_connections_from_junction(junction) {
            let incoming_road_id = match utils::get_incoming_road_id_from_connection(connection) {
                Some(v) => v,
                None => continue,
            };
            let incoming_road = match road_id_map.get(&incoming_road_id) {
                Some(v) => *v,
                None => continue,
            };
            if utils::road_belongs_to_junction(incoming_road) {
                let issue_id = cd
                    .result
                    .register_issue(id, desc, IssueSeverity::Error, rule);
                add_location_pair(cd, id, issue_id, "Connection with connecting road found as incoming road.", connection, connection);
                let junction_id = match utils::get_junction_id(junction) {
                    Some(v) => v,
                    None => continue,
                };
                let successor_junction_id = utils::get_linked_junction_id(incoming_road, LinkageTag::Successor);
                let predecessor_junction_id = utils::get_linked_junction_id(incoming_road, LinkageTag::Predecessor);
                let p = if successor_junction_id == Some(junction_id) {
                    utils::get_end_point_xyz_from_road_reference_line(incoming_road)
                } else if predecessor_junction_id == Some(junction_id) {
                    utils::get_start_point_xyz_from_road_reference_line(incoming_road)
                } else {
                    utils::get_middle_point_xyz_from_road_reference_line(incoming_road)
                };
                if let Some(p) = p {
                    cd.result.add_inertial_location(
                        id, issue_id, p.x, p.y, p.z, "Incoming road which is also a connecting road.",
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// junctions.connection.one_connection_element
// ---------------------------------------------------------------------------

fn check_junctions_connection_one_connection_element(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_junctions_connection_one_connection_element";
    let rule = "asam.net:xodr:1.7.0:junctions.connection.one_connection_element";
    let desc_prefix = "Connecting road";
    let road_id_map = utils::get_road_id_map(doc);
    let mut connecting_road_id_connections: HashMap<i64, Vec<Node>> = HashMap::new();
    for junction in utils::get_junctions(doc) {
        for connection in utils::get_connections_from_junction(junction) {
            let connecting_road_id = match utils::get_connecting_road_id_from_connection(connection) {
                Some(v) => v,
                None => continue,
            };
            connecting_road_id_connections
                .entry(connecting_road_id)
                .or_default()
                .push(connection);
        }
    }
    for (connecting_road_id, connections) in &connecting_road_id_connections {
        if connections.len() > 1 {
            let desc = format!(
                "{} {} shall be represented by only one <connection> element.",
                desc_prefix, connecting_road_id
            );
            let issue_id = cd
                .result
                .register_issue(id, &desc, IssueSeverity::Error, rule);
            for connection in connections {
                add_location_pair(cd, id, issue_id, "Connection with reused connecting road id.", *connection, *connection);
            }
            if let Some(connecting_road) = road_id_map.get(connecting_road_id) {
                if let Some(p) = utils::get_middle_point_xyz_from_road_reference_line(*connecting_road) {
                    cd.result
                        .add_inertial_location(id, issue_id, p.x, p.y, p.z, "Connecting road being reused.");
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// junctions.connection.one_link_to_incoming
// ---------------------------------------------------------------------------

fn raise_lane_linkage_issue(
    cd: &mut CheckerData,
    lane_link: Node,
    connecting_road: Node,
    connecting_lane_section: Node,
    connecting_lane: Node,
    connecting_contact_point: ContactPoint,
) {
    let id = "check_asam_xodr_junctions_connection_one_link_to_incoming";
    let rule = "asam.net:xodr:1.8.0:junctions.connection.one_link_to_incoming";
    let desc = "A connecting road shall only have the <laneLink> element for that direction.";
    let issue_id = cd.result.register_issue(id, desc, IssueSeverity::Error, rule);
    add_location_pair(cd, id, issue_id, "Lane link in opposite direction.", lane_link, lane_link);
    let s = match connecting_contact_point {
        ContactPoint::Start => 0.0,
        ContactPoint::End => match utils::get_road_length(connecting_road) {
            Some(v) => v,
            None => return,
        },
    };
    if let Some(p) = utils::get_middle_point_xyz_at_height_zero_from_lane_by_s(
        connecting_road, connecting_lane_section, connecting_lane, s,
    ) {
        cd.result
            .add_inertial_location(id, issue_id, p.x, p.y, p.z, "Lane link in opposite direction.");
    }
}

fn is_rht_lane_direction_valid(
    to_lane_id: i64,
    to_lane_direction: LaneDirection,
    from_lane_id: i64,
    from_lane_direction: LaneDirection,
    predecessor_road_linkage: Option<RoadLinkage>,
    successor_road_linkage: Option<RoadLinkage>,
    connection_contact_point: ContactPoint,
) -> bool {
    if to_lane_direction == LaneDirection::Both && from_lane_direction == LaneDirection::Both {
        return true;
    }
    let mut to_lane_id = to_lane_id;
    let mut from_lane_id = from_lane_id;
    if from_lane_direction == LaneDirection::Reversed {
        from_lane_id *= -1;
    }
    if to_lane_direction == LaneDirection::Reversed {
        to_lane_id *= -1;
    }
    if connection_contact_point == ContactPoint::Start && predecessor_road_linkage.is_some() {
        if from_lane_direction == LaneDirection::Both {
            if to_lane_id < 0 {
                return false;
            }
        } else if let Some(pl) = predecessor_road_linkage {
            if pl.contact_point == ContactPoint::End {
                if to_lane_direction == LaneDirection::Both && from_lane_id > 0 {
                    return false;
                } else if from_lane_id > 0 || to_lane_id > 0 {
                    return false;
                }
            } else if pl.contact_point == ContactPoint::Start {
                if to_lane_direction == LaneDirection::Both && from_lane_id < 0 {
                    return false;
                } else if from_lane_id < 0 || to_lane_id > 0 {
                    return false;
                }
            }
        }
    }
    if connection_contact_point == ContactPoint::End && successor_road_linkage.is_some() {
        if from_lane_direction == LaneDirection::Both {
            if to_lane_id > 0 {
                return false;
            }
        } else if let Some(sl) = successor_road_linkage {
            if sl.contact_point == ContactPoint::End {
                if to_lane_direction == LaneDirection::Both && from_lane_id > 0 {
                    return false;
                } else if from_lane_id > 0 || to_lane_id < 0 {
                    return false;
                }
            } else if sl.contact_point == ContactPoint::Start {
                if to_lane_direction == LaneDirection::Both && from_lane_id < 0 {
                    return false;
                } else if from_lane_id < 0 || to_lane_id < 0 {
                    return false;
                }
            }
        }
    }
    true
}

fn is_lht_lane_direction_valid(
    to_lane_id: i64,
    to_lane_direction: LaneDirection,
    from_lane_id: i64,
    from_lane_direction: LaneDirection,
    predecessor_road_linkage: Option<RoadLinkage>,
    successor_road_linkage: Option<RoadLinkage>,
    connection_contact_point: ContactPoint,
) -> bool {
    if to_lane_direction == LaneDirection::Both && from_lane_direction == LaneDirection::Both {
        return true;
    }
    let mut to_lane_id = to_lane_id;
    let mut from_lane_id = from_lane_id;
    if from_lane_direction == LaneDirection::Reversed {
        from_lane_id *= -1;
    }
    if to_lane_direction == LaneDirection::Reversed {
        to_lane_id *= -1;
    }
    if connection_contact_point == ContactPoint::Start && predecessor_road_linkage.is_some() {
        if from_lane_direction == LaneDirection::Both {
            if to_lane_id < 0 {
                return false;
            }
        } else if let Some(pl) = predecessor_road_linkage {
            if pl.contact_point == ContactPoint::End {
                if to_lane_direction == LaneDirection::Both && from_lane_id < 0 {
                    return false;
                } else if from_lane_id < 0 || to_lane_id < 0 {
                    return false;
                }
            } else if pl.contact_point == ContactPoint::Start {
                if to_lane_direction == LaneDirection::Both && from_lane_id > 0 {
                    return false;
                } else if from_lane_id > 0 || to_lane_id < 0 {
                    return false;
                }
            }
        }
    }
    if connection_contact_point == ContactPoint::End && successor_road_linkage.is_some() {
        if from_lane_direction == LaneDirection::Both {
            if to_lane_id > 0 {
                return false;
            }
        } else if let Some(sl) = successor_road_linkage {
            if sl.contact_point == ContactPoint::End {
                if to_lane_direction == LaneDirection::Both && from_lane_id < 0 {
                    return false;
                } else if from_lane_id < 0 || to_lane_id > 0 {
                    return false;
                }
            } else if sl.contact_point == ContactPoint::Start {
                if to_lane_direction == LaneDirection::Both && from_lane_id > 0 {
                    return false;
                } else if from_lane_id > 0 || to_lane_id > 0 {
                    return false;
                }
            }
        }
    }
    true
}

fn is_lht_to_rht_lane_direction_valid(
    to_lane_id: i64,
    to_lane_direction: LaneDirection,
    from_lane_id: i64,
    from_lane_direction: LaneDirection,
    predecessor_road_linkage: Option<RoadLinkage>,
    successor_road_linkage: Option<RoadLinkage>,
    connection_contact_point: ContactPoint,
) -> bool {
    if to_lane_direction == LaneDirection::Both && from_lane_direction == LaneDirection::Both {
        return true;
    }
    let mut to_lane_id = to_lane_id;
    let mut from_lane_id = from_lane_id;
    if from_lane_direction == LaneDirection::Reversed {
        from_lane_id *= -1;
    }
    if to_lane_direction == LaneDirection::Reversed {
        to_lane_id *= -1;
    }
    if connection_contact_point == ContactPoint::Start && predecessor_road_linkage.is_some() {
        if from_lane_direction == LaneDirection::Both {
            if to_lane_id > 0 {
                return false;
            }
        } else if let Some(pl) = predecessor_road_linkage {
            if pl.contact_point == ContactPoint::End {
                if to_lane_direction == LaneDirection::Both && from_lane_id < 0 {
                    return false;
                } else if from_lane_id < 0 || to_lane_id > 0 {
                    return false;
                }
            } else if pl.contact_point == ContactPoint::Start {
                if to_lane_direction == LaneDirection::Both && from_lane_id > 0 {
                    return false;
                } else if from_lane_id > 0 || to_lane_id > 0 {
                    return false;
                }
            }
        }
    }
    if connection_contact_point == ContactPoint::End && successor_road_linkage.is_some() {
        if from_lane_direction == LaneDirection::Both {
            if to_lane_id < 0 {
                return false;
            }
        } else if let Some(sl) = successor_road_linkage {
            if sl.contact_point == ContactPoint::End {
                if to_lane_direction == LaneDirection::Both && from_lane_id < 0 {
                    return false;
                } else if from_lane_id < 0 || to_lane_id < 0 {
                    return false;
                }
            } else if sl.contact_point == ContactPoint::Start {
                if to_lane_direction == LaneDirection::Both && from_lane_id > 0 {
                    return false;
                } else if from_lane_id > 0 || to_lane_id < 0 {
                    return false;
                }
            }
        }
    }
    true
}

fn is_rht_to_lht_lane_direction_valid(
    to_lane_id: i64,
    to_lane_direction: LaneDirection,
    from_lane_id: i64,
    from_lane_direction: LaneDirection,
    predecessor_road_linkage: Option<RoadLinkage>,
    successor_road_linkage: Option<RoadLinkage>,
    connection_contact_point: ContactPoint,
) -> bool {
    if to_lane_direction == LaneDirection::Both && from_lane_direction == LaneDirection::Both {
        return true;
    }
    let mut to_lane_id = to_lane_id;
    let mut from_lane_id = from_lane_id;
    if from_lane_direction == LaneDirection::Reversed {
        from_lane_id *= -1;
    }
    if to_lane_direction == LaneDirection::Reversed {
        to_lane_id *= -1;
    }
    if connection_contact_point == ContactPoint::Start && predecessor_road_linkage.is_some() {
        if from_lane_direction == LaneDirection::Both {
            if to_lane_id < 0 {
                return false;
            }
        } else if let Some(pl) = predecessor_road_linkage {
            if pl.contact_point == ContactPoint::End {
                if to_lane_direction == LaneDirection::Both && from_lane_id > 0 {
                    return false;
                } else if from_lane_id > 0 || to_lane_id < 0 {
                    return false;
                }
            } else if pl.contact_point == ContactPoint::Start {
                if to_lane_direction == LaneDirection::Both && from_lane_id < 0 {
                    return false;
                } else if from_lane_id < 0 || to_lane_id < 0 {
                    return false;
                }
            }
        }
    }
    if connection_contact_point == ContactPoint::End && successor_road_linkage.is_some() {
        if from_lane_direction == LaneDirection::Both {
            if to_lane_id > 0 {
                return false;
            }
        } else if let Some(sl) = successor_road_linkage {
            if sl.contact_point == ContactPoint::End {
                if to_lane_direction == LaneDirection::Both && from_lane_id > 0 {
                    return false;
                } else if from_lane_id > 0 || to_lane_id > 0 {
                    return false;
                }
            } else if sl.contact_point == ContactPoint::Start {
                if to_lane_direction == LaneDirection::Both && from_lane_id < 0 {
                    return false;
                } else if from_lane_id < 0 || to_lane_id > 0 {
                    return false;
                }
            }
        }
    }
    true
}

fn check_connection_lane_link_same_direction(
    cd: &mut CheckerData,
    road_id_map: &HashMap<i64, Node>,
    connection: Node,
) {
    let connection_contact_point = match utils::get_contact_point_from_connection(connection) {
        Some(v) => v,
        None => return,
    };
    let incoming_road_id = match utils::get_incoming_road_id_from_connection(connection) {
        Some(v) => v,
        None => return,
    };
    let connecting_road_id = match utils::get_connecting_road_id_from_connection(connection) {
        Some(v) => v,
        None => return,
    };
    let connecting_road = match road_id_map.get(&connecting_road_id) {
        Some(v) => *v,
        None => return,
    };
    let incoming_road = match road_id_map.get(&incoming_road_id) {
        Some(v) => *v,
        None => return,
    };
    let connecting_road_predecessor = utils::get_road_linkage(connecting_road, LinkageTag::Predecessor);
    let connecting_road_successor = utils::get_road_linkage(connecting_road, LinkageTag::Successor);
    let connection_traffic_hand = utils::get_traffic_hand_rule_from_road(connecting_road);
    let incoming_traffic_hand = utils::get_traffic_hand_rule_from_road(incoming_road);
    let cls = match utils::get_incoming_and_connection_contacting_lane_sections(connection, road_id_map) {
        Some(v) => v,
        None => return,
    };
    for lane_link in utils::get_lane_links_from_connection(connection) {
        let from_lane_id = match utils::get_from_attribute_from_lane_link(lane_link) {
            Some(v) => v,
            None => continue,
        };
        let to_lane_id = match utils::get_to_attribute_from_lane_link(lane_link) {
            Some(v) => v,
            None => continue,
        };
        let from_lane = utils::get_lane_from_lane_section(cls.incoming, from_lane_id);
        let to_lane = utils::get_lane_from_lane_section(cls.connection, to_lane_id);
        let (from_lane, to_lane) = match (from_lane, to_lane) {
            (Some(a), Some(b)) => (a, b),
            _ => continue,
        };
        let from_lane_direction = match utils::get_lane_direction(from_lane) {
            Some(v) => v,
            None => continue,
        };
        let to_lane_direction = match utils::get_lane_direction(to_lane) {
            Some(v) => v,
            None => continue,
        };
        let valid = match (connection_traffic_hand, incoming_traffic_hand) {
            (TrafficHandRule::RHT, TrafficHandRule::RHT) => is_rht_lane_direction_valid(
                to_lane_id, to_lane_direction, from_lane_id, from_lane_direction,
                connecting_road_predecessor, connecting_road_successor, connection_contact_point,
            ),
            (TrafficHandRule::RHT, TrafficHandRule::LHT) => is_lht_to_rht_lane_direction_valid(
                to_lane_id, to_lane_direction, from_lane_id, from_lane_direction,
                connecting_road_predecessor, connecting_road_successor, connection_contact_point,
            ),
            (TrafficHandRule::LHT, TrafficHandRule::LHT) => is_lht_lane_direction_valid(
                to_lane_id, to_lane_direction, from_lane_id, from_lane_direction,
                connecting_road_predecessor, connecting_road_successor, connection_contact_point,
            ),
            (TrafficHandRule::LHT, TrafficHandRule::RHT) => is_rht_to_lht_lane_direction_valid(
                to_lane_id, to_lane_direction, from_lane_id, from_lane_direction,
                connecting_road_predecessor, connecting_road_successor, connection_contact_point,
            ),
        };
        if !valid {
            raise_lane_linkage_issue(
                cd, lane_link, connecting_road, cls.connection, to_lane, connection_contact_point,
            );
        }
    }
}

fn check_junctions_connection_one_link_to_incoming(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_junctions_connection_one_link_to_incoming";
    let rule = "asam.net:xodr:1.8.0:junctions.connection.one_link_to_incoming";
    let road_id_map = utils::get_road_id_map(doc);
    let mut connection_road_link_map: HashMap<i64, HashMap<i64, Vec<Node>>> = HashMap::new();
    for junction in utils::get_junctions(doc) {
        for connection in utils::get_connections_from_junction(junction) {
            let incoming_road_id = match utils::get_incoming_road_id_from_connection(connection) {
                Some(v) => v,
                None => continue,
            };
            let connecting_road_id = match utils::get_connecting_road_id_from_connection(connection) {
                Some(v) => v,
                None => continue,
            };
            connection_road_link_map
                .entry(incoming_road_id)
                .or_default()
                .entry(connecting_road_id)
                .or_default()
                .push(connection);
            check_connection_lane_link_same_direction(cd, &road_id_map, connection);
        }
    }
    for (incoming_road_id, connecting_map) in &connection_road_link_map {
        for (connecting_road_id, connections) in connecting_map {
            if connections.len() > 1 {
                let desc = format!(
                    "Connecting road {} shall be represented by at most one <connection> element per incoming road id.",
                    connecting_road_id
                );
                let issue_id = cd
                    .result
                    .register_issue(id, &desc, IssueSeverity::Error, rule);
                let mut has_start = false;
                let mut has_end = false;
                for connection in connections {
                    add_location_pair(
                        cd, id, issue_id,
                        &format!(
                            "Connection with reused (incoming_road_id, connecting_road_id) = ({}, {}) pair.",
                            incoming_road_id, connecting_road_id
                        ),
                        *connection, *connection,
                    );
                    if let Some(cp) = utils::get_contact_point_from_connection(*connection) {
                        if cp == ContactPoint::Start {
                            has_start = true;
                        } else {
                            has_end = true;
                        }
                    }
                }
                if let Some(connecting_road) = road_id_map.get(connecting_road_id) {
                    if has_start {
                        if let Some(p) = utils::get_start_point_xyz_from_road_reference_line(*connecting_road) {
                            cd.result.add_inertial_location(
                                id, issue_id, p.x, p.y, p.z, "Multiple connection elements to the same incoming road.",
                            );
                        }
                    }
                    if has_end {
                        if let Some(p) = utils::get_end_point_xyz_from_road_reference_line(*connecting_road) {
                            cd.result.add_inertial_location(
                                id, issue_id, p.x, p.y, p.z, "Multiple connection elements to the same incoming road.",
                            );
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// junctions.connection.start_along_linkage
// ---------------------------------------------------------------------------

fn check_junction_connection_start_along_linkage(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_junctions_connection_start_along_linkage";
    let rule = "asam.net:xodr:1.7.0:junctions.connection.start_along_linkage";
    let desc = "The value 'start' shall be used to indicate that the connecting road runs along the linkage indicated in the element.";
    let road_id_map = utils::get_road_id_map(doc);
    for junction in utils::get_junctions(doc) {
        for connection in utils::get_connections_from_junction(junction) {
            let contact_point = match utils::get_contact_point_from_connection(connection) {
                Some(v) => v,
                None => continue,
            };
            if contact_point == ContactPoint::Start {
                let connection_road_id = match utils::get_connecting_road_id_from_connection(connection) {
                    Some(v) => v,
                    None => continue,
                };
                let incoming_road_id = match utils::get_incoming_road_id_from_connection(connection) {
                    Some(v) => v,
                    None => continue,
                };
                let connection_road = match road_id_map.get(&connection_road_id) {
                    Some(v) => *v,
                    None => continue,
                };
                let predecessor_linkage = utils::get_road_linkage(connection_road, LinkageTag::Predecessor);
                if predecessor_linkage.is_none() {
                    continue;
                }
                if predecessor_linkage.unwrap().id != incoming_road_id {
                    let issue_id = cd
                        .result
                        .register_issue(id, desc, IssueSeverity::Error, rule);
                    add_location_pair(cd, id, issue_id, "Contact point 'start' not used on predecessor road connection.", connection, connection);
                    if let Some(p) = utils::get_start_point_xyz_from_road_reference_line(connection_road) {
                        cd.result.add_inertial_location(
                            id, issue_id, p.x, p.y, p.z, "Contact point 'start' not used on predecessor road connection.",
                        );
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// junctions.connection.end_opposite_linkage
// ---------------------------------------------------------------------------

fn check_junction_connection_end_opposite_linkage(cd: &mut CheckerData) {
    let doc = match cd.doc {
        Some(d) => d,
        None => return,
    };
    let id = "check_asam_xodr_junctions_connection_end_opposite_linkage";
    let rule = "asam.net:xodr:1.7.0:junctions.connection.end_opposite_linkage";
    let desc = "The value 'end' shall be used to indicate that the connecting road runs along the opposite direction of the linkage indicated in the element.";
    let road_id_map = utils::get_road_id_map(doc);
    for junction in utils::get_junctions(doc) {
        for connection in utils::get_connections_from_junction(junction) {
            let contact_point = match utils::get_contact_point_from_connection(connection) {
                Some(v) => v,
                None => continue,
            };
            if contact_point == ContactPoint::End {
                let connection_road_id = match utils::get_connecting_road_id_from_connection(connection) {
                    Some(v) => v,
                    None => continue,
                };
                let incoming_road_id = match utils::get_incoming_road_id_from_connection(connection) {
                    Some(v) => v,
                    None => continue,
                };
                let connection_road = match road_id_map.get(&connection_road_id) {
                    Some(v) => *v,
                    None => continue,
                };
                let successor_linkage = utils::get_road_linkage(connection_road, LinkageTag::Successor);
                if successor_linkage.is_none() {
                    continue;
                }
                if successor_linkage.unwrap().id != incoming_road_id {
                    let issue_id = cd
                        .result
                        .register_issue(id, desc, IssueSeverity::Error, rule);
                    add_location_pair(cd, id, issue_id, "Contact point 'end' not used on successor road connection.", connection, connection);
                    if let Some(p) = utils::get_end_point_xyz_from_road_reference_line(connection_road) {
                        cd.result.add_inertial_location(
                            id, issue_id, p.x, p.y, p.z, "Contact point 'end' not used on successor road connection.",
                        );
                    }
                }
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
            id: "check_asam_xodr_road_lane_level_true_one_side",
            description: "Road lane level must be true on one side.",
            rule_uid: "asam.net:xodr:1.7.0:road.lane.level_true_one_side",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_lane_level_true_one_side,
        },
        CheckerSpec {
            id: "check_asam_xodr_road_lane_access_no_mix_of_deny_or_allow",
            description: "Lane access must not mix deny and allow.",
            rule_uid: "asam.net:xodr:1.7.0:road.lane.access.no_mix_of_deny_or_allow",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_lane_access_no_mix,
        },
        CheckerSpec {
            id: "check_asam_xodr_road_lane_link_lanes_across_lane_sections",
            description: "Lane links across lane sections.",
            rule_uid: "asam.net:xodr:1.4.0:road.lane.link.lanes_across_lane_sections",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_lanes_across_lane_sections,
        },
        CheckerSpec {
            id: "check_asam_xodr_road_linkage_is_junction_needed",
            description: "Road linkage may require a junction.",
            rule_uid: "asam.net:xodr:1.4.0:road.linkage.is_junction_needed",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_road_linkage_is_junction_needed,
        },
        CheckerSpec {
            id: "check_asam_xodr_road_lane_link_zero_width_at_start",
            description: "Lane link zero width at start.",
            rule_uid: "asam.net:xodr:1.7.0:road.lane.link.zero_width_at_start",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: |cd| {
                check_road_lane_link_zero_width_at_start(cd);
                check_junction_road_lane_link_zero_width_at_start(cd);
            },
        },
        CheckerSpec {
            id: "check_asam_xodr_road_lane_link_zero_width_at_end",
            description: "Lane link zero width at end.",
            rule_uid: "asam.net:xodr:1.7.0:road.lane.link.zero_width_at_end",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: |cd| {
                check_road_lane_link_zero_width_at_end(cd);
                check_junction_road_lane_link_zero_width_at_end(cd);
            },
        },
        CheckerSpec {
            id: "check_asam_xodr_road_lane_link_new_lane_appear",
            description: "New lane appears in lane link.",
            rule_uid: "asam.net:xodr:1.4.0:road.lane.link.new_lane_appear",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_road_lane_link_new_lane_appear,
        },
        CheckerSpec {
            id: "check_asam_xodr_junctions_connection_connect_road_no_incoming_road",
            description: "Connection road with no incoming road.",
            rule_uid: "asam.net:xodr:1.4.0:junctions.connection.connect_road_no_incoming_road",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_junctions_connection_connect_road_no_incoming_road,
        },
        CheckerSpec {
            id: "check_asam_xodr_junctions_connection_one_connection_element",
            description: "One connection element.",
            rule_uid: "asam.net:xodr:1.7.0:junctions.connection.one_connection_element",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: Some("<=1.7.0"),
            version_required: true,
            run: check_junctions_connection_one_connection_element,
        },
        CheckerSpec {
            id: "check_asam_xodr_junctions_connection_one_link_to_incoming",
            description: "One link to incoming road.",
            rule_uid: "asam.net:xodr:1.8.0:junctions.connection.one_link_to_incoming",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_junctions_connection_one_link_to_incoming,
        },
        CheckerSpec {
            id: "check_asam_xodr_junctions_connection_start_along_linkage",
            description: "Connection start along linkage.",
            rule_uid: "asam.net:xodr:1.7.0:junctions.connection.start_along_linkage",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_junction_connection_start_along_linkage,
        },
        CheckerSpec {
            id: "check_asam_xodr_junctions_connection_end_opposite_linkage",
            description: "Connection end opposite linkage.",
            rule_uid: "asam.net:xodr:1.7.0:junctions.connection.end_opposite_linkage",
            preconditions: crate::opendrive::checks::BASIC_PRECONDITIONS,
            applicable_versions: None,
            version_required: true,
            run: check_junction_connection_end_opposite_linkage,
        },
    ]
}
