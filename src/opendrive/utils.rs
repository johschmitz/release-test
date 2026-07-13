// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// OpenDRIVE helper functions, ported from qc_opendrive/base/utils.py.

use roxmltree::{Document, Node};

use crate::opendrive::models::*;

pub const EPSILON: f64 = 1.0e-6;

pub fn to_int(s: &str) -> Option<i64> {
    s.trim().parse::<i64>().ok()
}

pub fn to_float(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok()
}

pub fn xml_string_to_bool(value: &str) -> bool {
    value.eq_ignore_ascii_case("true")
}

/// Read the OpenDRIVE schema version from the header's revMajor.revMinor.
/// Returns e.g. "1.7.0". Mirrors get_standard_schema_version.
pub fn get_standard_schema_version(doc: &Document) -> Option<String> {
    let root = doc.root_element();
    let header = root.children().find(|n| n.has_tag_name("header"))?;
    let rev_major = header.attribute("revMajor")?;
    let rev_minor = header.attribute("revMinor")?;
    Some(format!("{}.{}.0", rev_major, rev_minor))
}

/// Produce an lxml-style xpath for a node: `/OpenDRIVE/road/lanes/laneSection[1]/left/lane[1]`
/// 1-based sibling indexing, no namespaces.
///
/// Mirrors `lxml.etree.getpath`: an element is given a `[n]` index (1-based
/// position among same-tag siblings) only when it has at least one sibling
/// with the *same* tag name. The document root element is never indexed. This
/// matches the xpaths asserted by the Python reference test suite.
pub fn node_xpath(node: Node) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = Some(node);
    while let Some(n) = cur {
        if n.is_element() {
            // Count preceding same-tag siblings to compute 1-based position.
            let mut preceding = 0;
            let mut sib = n.prev_sibling_element();
            while let Some(s) = sib {
                if s.tag_name().name() == n.tag_name().name() {
                    preceding += 1;
                }
                sib = s.prev_sibling_element();
            }
            // Determine whether any same-tag sibling exists (before or after).
            let mut has_same_tag_sibling = preceding > 0;
            if !has_same_tag_sibling {
                let mut sib = n.next_sibling_element();
                while let Some(s) = sib {
                    if s.tag_name().name() == n.tag_name().name() {
                        has_same_tag_sibling = true;
                        break;
                    }
                    sib = s.next_sibling_element();
                }
            }
            // lxml does not index the document root element.
            let is_root = n.parent_element().is_none();
            if is_root {
                parts.push(n.tag_name().name().to_string());
            } else if has_same_tag_sibling {
                // preceding counts prior same-tag siblings; lxml uses 1-based position.
                parts.push(format!("{}[{}]", n.tag_name().name(), preceding + 1));
            } else {
                parts.push(n.tag_name().name().to_string());
            }
        }
        cur = n.parent_element();
    }
    parts.reverse();
    format!("/{}", parts.join("/"))
}

/// Get the root element's line number (1-based) from the document.
pub fn node_row(doc: &Document, node: Node) -> Option<u32> {
    let pos = doc.text_pos_at(node.range().start);
    Some(pos.row as u32)
}

// ---------------------------------------------------------------------------
// Traversal
// ---------------------------------------------------------------------------

pub fn get_roads<'a, 'input>(doc: &'a Document<'input>) -> Vec<Node<'a, 'input>> {
    doc.descendants().filter(|n| n.has_tag_name("road")).collect()
}

pub fn get_junctions<'a, 'input>(doc: &'a Document<'input>) -> Vec<Node<'a, 'input>> {
    doc.descendants().filter(|n| n.has_tag_name("junction")).collect()
}

pub fn get_lanes<'a, 'input>(doc: &'a Document<'input>) -> Vec<Node<'a, 'input>> {
    doc.descendants().filter(|n| n.has_tag_name("lane")).collect()
}

pub fn get_lane_sections<'a, 'input>(road: Node<'a, 'input>) -> Vec<Node<'a, 'input>> {
    road.descendants().filter(|n| n.has_tag_name("laneSection")).collect()
}

pub fn get_last_lane_section<'a, 'input>(road: Node<'a, 'input>) -> Option<Node<'a, 'input>> {
    get_lane_sections(road).into_iter().last()
}

pub fn get_first_lane_section<'a, 'input>(road: Node<'a, 'input>) -> Option<Node<'a, 'input>> {
    get_lane_sections(road).into_iter().next()
}

pub fn get_road_id_map<'a, 'input>(
    doc: &'a Document<'input>,
) -> std::collections::HashMap<i64, Node<'a, 'input>> {
    let mut m = std::collections::HashMap::new();
    for road in get_roads(doc) {
        if let Some(id) = to_int(road.attribute("id").unwrap_or("")) {
            m.insert(id, road);
        }
    }
    m
}

pub fn get_junction_id_map<'a, 'input>(
    doc: &'a Document<'input>,
) -> std::collections::HashMap<i64, Node<'a, 'input>> {
    let mut m = std::collections::HashMap::new();
    for junction in get_junctions(doc) {
        if let Some(id) = to_int(junction.attribute("id").unwrap_or("")) {
            m.insert(id, junction);
        }
    }
    m
}

pub fn get_left_lanes_from_lane_section<'a, 'input>(
    lane_section: Node<'a, 'input>,
) -> Vec<Node<'a, 'input>> {
    lane_section
        .children()
        .find(|n| n.has_tag_name("left"))
        .map(|left| left.children().filter(|n| n.has_tag_name("lane")).collect())
        .unwrap_or_default()
}

pub fn get_right_lanes_from_lane_section<'a, 'input>(
    lane_section: Node<'a, 'input>,
) -> Vec<Node<'a, 'input>> {
    lane_section
        .children()
        .find(|n| n.has_tag_name("right"))
        .map(|right| right.children().filter(|n| n.has_tag_name("lane")).collect())
        .unwrap_or_default()
}

pub fn get_left_and_right_lanes_from_lane_section<'a, 'input>(
    lane_section: Node<'a, 'input>,
) -> Vec<Node<'a, 'input>> {
    let mut v = get_left_lanes_from_lane_section(lane_section);
    v.extend(get_right_lanes_from_lane_section(lane_section));
    v
}

// ---------------------------------------------------------------------------
// Linkage
// ---------------------------------------------------------------------------

pub fn get_road_linkage(road: Node, linkage_tag: LinkageTag) -> Option<RoadLinkage> {
    let road_link = road.children().find(|n| n.has_tag_name("link"))?;
    let linkage = road_link.children().find(|n| n.has_tag_name(linkage_tag.value()))?;
    if linkage.attribute("elementType") == Some("road") {
        let road_id = to_int(linkage.attribute("elementId")?)?;
        let contact_point = ContactPoint::from_str(linkage.attribute("contactPoint")?)?;
        Some(RoadLinkage {
            id: road_id,
            contact_point,
        })
    } else {
        None
    }
}

pub fn get_linked_junction_id(road: Node, linkage_tag: LinkageTag) -> Option<i64> {
    let road_link = road.children().find(|n| n.has_tag_name("link"))?;
    let linkage = road_link.children().find(|n| n.has_tag_name(linkage_tag.value()))?;
    if linkage.attribute("elementType") == Some("junction") {
        to_int(linkage.attribute("elementId")?)
    } else {
        None
    }
}

pub fn get_predecessor_road_id(road: Node) -> Option<i64> {
    get_road_linkage(road, LinkageTag::Predecessor).map(|l| l.id)
}

pub fn get_successor_road_id(road: Node) -> Option<i64> {
    get_road_linkage(road, LinkageTag::Successor).map(|l| l.id)
}

pub fn get_predecessor_lane_ids(lane: Node) -> Vec<i64> {
    let mut ids = Vec::new();
    for link in lane.children().filter(|n| n.has_tag_name("link")) {
        for pred in link.children().filter(|n| n.has_tag_name("predecessor")) {
            if let Some(id) = to_int(pred.attribute("id").unwrap_or("")) {
                ids.push(id);
            }
        }
    }
    ids
}

pub fn get_successor_lane_ids(lane: Node) -> Vec<i64> {
    let mut ids = Vec::new();
    for link in lane.children().filter(|n| n.has_tag_name("link")) {
        for succ in link.children().filter(|n| n.has_tag_name("successor")) {
            if let Some(id) = to_int(succ.attribute("id").unwrap_or("")) {
                ids.push(id);
            }
        }
    }
    ids
}

pub fn get_lane_from_lane_section<'a, 'input>(
    lane_section: Node<'a, 'input>,
    lane_id: i64,
) -> Option<Node<'a, 'input>> {
    for lane in get_left_and_right_lanes_from_lane_section(lane_section) {
        if get_lane_id(lane) == Some(lane_id) {
            return Some(lane);
        }
    }
    None
}

pub fn get_lane_level_from_lane(lane: Node) -> bool {
    lane.attribute("level") == Some("true")
}

pub fn get_type_from_lane<'a>(lane: Node<'a, 'a>) -> Option<&'a str> {
    lane.attribute("type")
}

pub fn get_lane_id(lane: Node) -> Option<i64> {
    to_int(lane.attribute("id").unwrap_or(""))
}

pub fn get_road_junction_id(road: Node) -> Option<i64> {
    to_int(road.attribute("junction").unwrap_or(""))
}

pub fn get_road_link_element<'a, 'input>(
    road: Node<'a, 'input>,
    link_id: i64,
    linkage_tag: LinkageTag,
) -> Option<Node<'a, 'input>> {
    let tag = match linkage_tag {
        LinkageTag::Predecessor => "predecessor",
        LinkageTag::Successor => "successor",
    };
    for link in road.children().filter(|n| n.has_tag_name("link")) {
        for linkage in link.children().filter(|n| n.has_tag_name(tag)) {
            if let Some(id) = to_int(linkage.attribute("elementId").unwrap_or("")) {
                if id == link_id {
                    return Some(linkage);
                }
            }
        }
    }
    None
}

pub fn road_belongs_to_junction(road: Node) -> bool {
    match get_road_junction_id(road) {
        Some(id) => id != -1,
        None => false,
    }
}

pub fn get_incoming_road_id_from_connection(connection: Node) -> Option<i64> {
    to_int(connection.attribute("incomingRoad").unwrap_or(""))
}

pub fn get_connecting_road_id_from_connection(connection: Node) -> Option<i64> {
    to_int(connection.attribute("connectingRoad").unwrap_or(""))
}

pub fn get_contact_point_from_connection(connection: Node) -> Option<ContactPoint> {
    ContactPoint::from_str(connection.attribute("contactPoint")?)
}

pub fn get_from_attribute_from_lane_link(lane_link: Node) -> Option<i64> {
    to_int(lane_link.attribute("from").unwrap_or(""))
}

pub fn get_to_attribute_from_lane_link(lane_link: Node) -> Option<i64> {
    to_int(lane_link.attribute("to").unwrap_or(""))
}

pub fn get_lane_links_from_connection<'a, 'input>(connection: Node<'a, 'input>) -> Vec<Node<'a, 'input>> {
    connection.children().filter(|n| n.has_tag_name("laneLink")).collect()
}

pub fn get_connections_from_junction<'a, 'input>(junction: Node<'a, 'input>) -> Vec<Node<'a, 'input>> {
    junction.children().filter(|n| n.has_tag_name("connection")).collect()
}

pub fn get_junction_id(junction: Node) -> Option<i64> {
    to_int(junction.attribute("id").unwrap_or(""))
}

pub fn get_connecting_lane_ids(lane: Node, linkage_tag: LinkageTag) -> Vec<i64> {
    match linkage_tag {
        LinkageTag::Predecessor => get_predecessor_lane_ids(lane),
        LinkageTag::Successor => get_successor_lane_ids(lane),
    }
}

// ---------------------------------------------------------------------------
// Geometry / math
// ---------------------------------------------------------------------------

pub fn get_length_from_geometry(geometry: Node) -> Option<f64> {
    to_float(geometry.attribute("length").unwrap_or(""))
}

pub fn is_valid_param_poly3(param_poly3: &ParamPoly3) -> bool {
    let vals = [
        param_poly3.u.a,
        param_poly3.u.b,
        param_poly3.u.c,
        param_poly3.u.d,
        param_poly3.v.a,
        param_poly3.v.b,
        param_poly3.v.c,
        param_poly3.v.d,
    ];
    !vals.iter().any(|v| v.is_nan())
}

fn parse_param_poly3(geometry: Node, range: ParamPoly3Range) -> Option<ParamPoly3> {
    let mut found: Option<Node> = None;
    for el in geometry.descendants().filter(|n| n.has_tag_name("paramPoly3")) {
        found = Some(el);
    }
    let pp = found?;
    if pp.attribute("pRange") != Some(range.as_str()) {
        return None;
    }
    let parsed = ParamPoly3 {
        u: Poly3 {
            a: to_float(pp.attribute("aU").unwrap_or(""))?,
            b: to_float(pp.attribute("bU").unwrap_or(""))?,
            c: to_float(pp.attribute("cU").unwrap_or(""))?,
            d: to_float(pp.attribute("dU").unwrap_or(""))?,
        },
        v: Poly3 {
            a: to_float(pp.attribute("aV").unwrap_or(""))?,
            b: to_float(pp.attribute("bV").unwrap_or(""))?,
            c: to_float(pp.attribute("cV").unwrap_or(""))?,
            d: to_float(pp.attribute("dV").unwrap_or(""))?,
        },
        range,
    };
    if is_valid_param_poly3(&parsed) {
        Some(parsed)
    } else {
        None
    }
}

impl ParamPoly3Range {
    fn as_str(self) -> &'static str {
        match self {
            ParamPoly3Range::ArcLength => "arcLength",
            ParamPoly3Range::Normalized => "normalized",
        }
    }
}

pub fn get_normalized_param_poly3_from_geometry(geometry: Node) -> Option<ParamPoly3> {
    parse_param_poly3(geometry, ParamPoly3Range::Normalized)
}

pub fn get_arclen_param_poly3_from_geometry(geometry: Node) -> Option<ParamPoly3> {
    parse_param_poly3(geometry, ParamPoly3Range::ArcLength)
}

/// Arc-length integrand sqrt(du^2 + dv^2) for numerical integration.
pub fn arc_length_integrand(t: f64, du: &Poly3, dv: &Poly3) -> f64 {
    let du_t = du.deriv(t);
    let dv_t = dv.deriv(t);
    (du_t * du_t + dv_t * dv_t).sqrt()
}

/// Adaptive Simpson integration of `f` over [a, b] with given tolerance.
/// Returns (integral, estimated_error). Faithful enough for the 0.001 tolerance.
pub fn adaptive_simpson<F>(f: &F, a: f64, b: f64, tol: f64) -> (f64, f64)
where
    F: Fn(f64) -> f64,
{
    fn simpson<F: Fn(f64) -> f64>(f: &F, a: f64, b: f64) -> f64 {
        let m = (a + b) / 2.0;
        let h = (b - a) / 6.0;
        h * (f(a) + 4.0 * f(m) + f(b))
    }

    fn recurse<F: Fn(f64) -> f64>(
        f: &F,
        a: f64,
        b: f64,
        eps: f64,
        whole: f64,
        fa: f64,
        fb: f64,
        fm: f64,
        depth: u32,
    ) -> (f64, f64) {
        let m = (a + b) / 2.0;
        let lm = (a + m) / 2.0;
        let rm = (m + b) / 2.0;
        let flm = f(lm);
        let frm = f(rm);
        let left = (m - a) / 6.0 * (fa + 4.0 * flm + fm);
        let right = (b - m) / 6.0 * (fm + 4.0 * frm + fb);
        let delta = left + right - whole;
        if depth <= 0 || delta.abs() <= 15.0 * eps {
            (left + right + delta / 15.0, delta.abs() / 15.0)
        } else {
            let (l, le) = recurse(f, a, m, eps / 2.0, (m - a) / 6.0 * (fa + 4.0 * flm + fm), fa, fm, flm, depth - 1);
            let (r, re) = recurse(f, m, b, eps / 2.0, (b - m) / 6.0 * (fm + 4.0 * frm + fb), fm, fb, frm, depth - 1);
            (l + r, le + re)
        }
    }

    let fa = f(a);
    let fb = f(b);
    let fm = f((a + b) / 2.0);
    let whole = simpson(f, a, b);
    recurse(f, a, b, tol, whole, fa, fb, fm, 50)
}

pub fn get_contact_lane_section_from_linked_road<'a, 'input>(
    linkage: &RoadLinkage,
    road_id_map: &std::collections::HashMap<i64, Node<'a, 'input>>,
) -> Option<ContactingLaneSection<'a, 'input>> {
    let linked_road = road_id_map.get(&linkage.id)?;
    match linkage.contact_point {
        ContactPoint::Start => get_first_lane_section(*linked_road).map(|ls| ContactingLaneSection {
            lane_section: ls,
            linkage_tag: LinkageTag::Predecessor,
        }),
        ContactPoint::End => get_last_lane_section(*linked_road).map(|ls| ContactingLaneSection {
            lane_section: ls,
            linkage_tag: LinkageTag::Successor,
        }),
    }
}

pub fn get_contact_lane_section_from_junction_connection_road<'a, 'input>(
    connection_road: Node<'a, 'input>,
    contact_point: ContactPoint,
) -> Option<Node<'a, 'input>> {
    match contact_point {
        ContactPoint::Start => get_first_lane_section(connection_road),
        ContactPoint::End => get_last_lane_section(connection_road),
    }
}

pub fn get_incoming_and_connection_contacting_lane_sections<'a, 'input>(
    connection: Node<'a, 'input>,
    road_id_map: &std::collections::HashMap<i64, Node<'a, 'input>>,
) -> Option<ContactingLaneSections<'a, 'input>> {
    let connection_road_id = get_connecting_road_id_from_connection(connection)?;
    let incoming_road_id = get_incoming_road_id_from_connection(connection)?;
    let connection_road = road_id_map.get(&connection_road_id)?;
    let incoming_road = road_id_map.get(&incoming_road_id)?;

    let connection_contact_point = get_contact_point_from_connection(connection)?;
    let connection_lane_section =
        get_contact_lane_section_from_junction_connection_road(*connection_road, connection_contact_point)?;

    let connection_road_linkage = match connection_contact_point {
        ContactPoint::Start => get_road_linkage(*connection_road, LinkageTag::Predecessor),
        ContactPoint::End => get_road_linkage(*connection_road, LinkageTag::Successor),
    }?;

    let incoming_lane_section = match connection_road_linkage.contact_point {
        ContactPoint::Start => get_first_lane_section(*incoming_road),
        ContactPoint::End => get_last_lane_section(*incoming_road),
    }?;

    Some(ContactingLaneSections {
        incoming: incoming_lane_section,
        connection: connection_lane_section,
    })
}

pub fn is_valid_offset_poly3(offset_poly3: &OffsetPoly3) -> bool {
    let vals = [
        offset_poly3.poly3.a,
        offset_poly3.poly3.b,
        offset_poly3.poly3.c,
        offset_poly3.poly3.d,
        offset_poly3.s_offset,
    ];
    !vals.iter().any(|v| v.is_nan())
}

pub fn get_poly3_from_width<'a, 'input>(width: Node<'a, 'input>) -> Option<OffsetPoly3<'a, 'input>> {
    let poly3 = Poly3 {
        a: to_float(width.attribute("a").unwrap_or(""))?,
        b: to_float(width.attribute("b").unwrap_or(""))?,
        c: to_float(width.attribute("c").unwrap_or(""))?,
        d: to_float(width.attribute("d").unwrap_or(""))?,
    };
    let s_offset = to_float(width.attribute("sOffset").unwrap_or(""))?;
    let op = OffsetPoly3 {
        poly3,
        s_offset,
        xml_element: Some(width),
    };
    if is_valid_offset_poly3(&op) {
        Some(op)
    } else {
        None
    }
}

pub fn get_lane_width_poly3_list<'a, 'input>(lane: Node<'a, 'input>) -> Vec<OffsetPoly3<'a, 'input>> {
    lane.descendants()
        .filter(|n| n.has_tag_name("width"))
        .filter_map(|w| get_poly3_from_width(w))
        .collect()
}

/// Evaluate lane width at s_start_from_lane_section (>= 0). Mirrors Python.
pub fn evaluate_lane_width(lane: Node, s_start_from_lane_section: f64) -> Option<f64> {
    let lane_id = get_lane_id(lane)?;
    if lane_id == 0 {
        return Some(0.0);
    }
    let lane_width_poly3_list = get_lane_width_poly3_list(lane);
    if lane_width_poly3_list.is_empty() {
        return None;
    }
    let mut count = 0;
    for w in &lane_width_poly3_list {
        if w.s_offset > s_start_from_lane_section {
            break;
        }
        count += 1;
    }
    if count == 0 {
        return None;
    }
    let index = count - 1;
    let poly3 = &lane_width_poly3_list[index].poly3;
    Some(poly3.evaluate(s_start_from_lane_section - lane_width_poly3_list[index].s_offset))
}

pub fn get_connections_between_road_and_junction<'a, 'input: 'a>(
    road_id: i64,
    junction_id: i64,
    road_id_map: &std::collections::HashMap<i64, Node<'a, 'input>>,
    junction_id_map: &std::collections::HashMap<i64, Node<'a, 'input>>,
    incoming_road_contact_point: ContactPoint,
) -> Vec<Node<'a, 'input>> {
    let mut linkage_connections = Vec::new();
    let junction = match junction_id_map.get(&junction_id) {
        Some(j) => *j,
        None => return linkage_connections,
    };
    for connection in get_connections_from_junction(junction) {
        let incoming_road_id = get_incoming_road_id_from_connection(connection);
        let connecting_road_id = get_connecting_road_id_from_connection(connection);
        if incoming_road_id.is_none() || connecting_road_id.is_none() {
            continue;
        }
        if incoming_road_id != Some(road_id) {
            continue;
        }
        let connecting_road = match road_id_map.get(&connecting_road_id.unwrap()) {
            Some(r) => *r,
            None => continue,
        };
        let connection_contact_point = match get_contact_point_from_connection(connection) {
            Some(cp) => cp,
            None => continue,
        };
        let connection_road_linkage = match connection_contact_point {
            ContactPoint::Start => get_road_linkage(connecting_road, LinkageTag::Predecessor),
            ContactPoint::End => get_road_linkage(connecting_road, LinkageTag::Successor),
        };
        if connection_road_linkage.is_none() {
            continue;
        }
        if connection_road_linkage.unwrap().contact_point == incoming_road_contact_point {
            linkage_connections.push(connection);
        }
    }
    linkage_connections
}

pub fn get_connections_of_connecting_road<'a, 'input: 'a>(
    connecting_road_id: i64,
    junction: Node<'a, 'input>,
    connecting_road_contact_point: ContactPoint,
) -> Vec<Node<'a, 'input>> {
    let mut linkage_connections = Vec::new();
    for connection in get_connections_from_junction(junction) {
        let connection_connecting_road_id = get_connecting_road_id_from_connection(connection);
        if connection_connecting_road_id.is_none() {
            continue;
        }
        if connection_connecting_road_id != Some(connecting_road_id) {
            continue;
        }
        let contact_point = get_contact_point_from_connection(connection);
        if contact_point.is_none() {
            continue;
        }
        if contact_point == Some(connecting_road_contact_point) {
            linkage_connections.push(connection);
        }
    }
    linkage_connections
}

pub fn get_traffic_hand_rule_from_road(road: Node) -> TrafficHandRule {
    match road.attribute("rule") {
        None => TrafficHandRule::RHT,
        Some(r) => TrafficHandRule::from_str(r).unwrap_or(TrafficHandRule::RHT),
    }
}

pub fn get_road_length(road: Node) -> Option<f64> {
    to_float(road.attribute("length").unwrap_or(""))
}

pub fn get_s_from_lane_section(lane_section: Node) -> Option<f64> {
    to_float(lane_section.attribute("s").unwrap_or(""))
}

pub fn get_borders_from_lane<'a, 'input>(lane: Node<'a, 'input>) -> Vec<OffsetPoly3<'a, 'input>> {
    let mut border_list = Vec::new();
    for border in lane.descendants().filter(|n| n.has_tag_name("border")) {
        let a = match to_float(border.attribute("a").unwrap_or("")) {
            Some(v) => v,
            None => continue,
        };
        let b = match to_float(border.attribute("b").unwrap_or("")) {
            Some(v) => v,
            None => continue,
        };
        let c = match to_float(border.attribute("c").unwrap_or("")) {
            Some(v) => v,
            None => continue,
        };
        let d = match to_float(border.attribute("d").unwrap_or("")) {
            Some(v) => v,
            None => continue,
        };
        let s_offset = match to_float(border.attribute("sOffset").unwrap_or("")) {
            Some(v) => v,
            None => continue,
        };
        let poly3 = Poly3 { a, b, c, d };
        let op = OffsetPoly3 {
            poly3,
            s_offset,
            xml_element: Some(border),
        };
        if is_valid_offset_poly3(&op) {
            border_list.push(op);
        }
    }
    border_list
}

pub fn get_sorted_lane_sections_with_length_from_road<'a, 'input>(
    road: Node<'a, 'input>,
) -> Vec<LaneSectionWithLength<'a, 'input>> {
    let lane_sections = get_lane_sections(road);
    for ls in &lane_sections {
        if get_s_from_lane_section(*ls).is_none() {
            return Vec::new();
        }
    }
    let mut sorted = lane_sections;
    sorted.sort_by(|a, b| {
        get_s_from_lane_section(*a)
            .unwrap()
            .partial_cmp(&get_s_from_lane_section(*b).unwrap())
            .unwrap()
    });
    let mut result = Vec::new();
    for i in 0..sorted.len() {
        let s_start = get_s_from_lane_section(sorted[i]).unwrap();
        let s_end = if i < sorted.len() - 1 {
            get_s_from_lane_section(sorted[i + 1]).unwrap()
        } else {
            match get_road_length(road) {
                Some(l) => l,
                None => return Vec::new(),
            }
        };
        result.push(LaneSectionWithLength {
            lane_section: sorted[i],
            length: s_end - s_start,
        });
    }
    result
}

/// Parse a Poly3 from a node's attributes, returning None if any is missing/invalid.
fn poly3_from_attrs(node: Node, names: (&str, &str, &str, &str)) -> Option<Poly3> {
    let a = to_float(node.attribute(names.0).unwrap_or(""))?;
    let b = to_float(node.attribute(names.1).unwrap_or(""))?;
    let c = to_float(node.attribute(names.2).unwrap_or(""))?;
    let d = to_float(node.attribute(names.3).unwrap_or(""))?;
    Some(Poly3 { a, b, c, d })
}

pub fn get_road_elevations<'a, 'input>(road: Node<'a, 'input>) -> Vec<OffsetPoly3<'a, 'input>> {
    let elevation_profile = road.children().find(|n| n.has_tag_name("elevationProfile"));
    let mut list = Vec::new();
    if let Some(ep) = elevation_profile {
        for elevation in ep.descendants().filter(|n| n.has_tag_name("elevation")) {
            let poly3 = match poly3_from_attrs(elevation, ("a", "b", "c", "d")) {
                Some(p) => p,
                None => continue,
            };
            let s_offset = match to_float(elevation.attribute("s").unwrap_or("")) {
                Some(v) => v,
                None => continue,
            };
            let op = OffsetPoly3 {
                poly3,
                s_offset,
                xml_element: Some(elevation),
            };
            if is_valid_offset_poly3(&op) {
                list.push(op);
            }
        }
    }
    list
}

pub fn get_road_superelevations<'a, 'input>(road: Node<'a, 'input>) -> Vec<OffsetPoly3<'a, 'input>> {
    let lateral_profile = road.children().find(|n| n.has_tag_name("lateralProfile"));
    let mut list = Vec::new();
    if let Some(lp) = lateral_profile {
        for se in lp.descendants().filter(|n| n.has_tag_name("superelevation")) {
            let poly3 = match poly3_from_attrs(se, ("a", "b", "c", "d")) {
                Some(p) => p,
                None => continue,
            };
            let s_offset = match to_float(se.attribute("s").unwrap_or("")) {
                Some(v) => v,
                None => continue,
            };
            let op = OffsetPoly3 {
                poly3,
                s_offset,
                xml_element: Some(se),
            };
            if is_valid_offset_poly3(&op) {
                list.push(op);
            }
        }
    }
    list
}

pub fn get_lane_offsets_from_road<'a, 'input>(road: Node<'a, 'input>) -> Vec<OffsetPoly3<'a, 'input>> {
    let lanes = road.children().find(|n| n.has_tag_name("lanes"));
    let mut list = Vec::new();
    if let Some(lanes) = lanes {
        for lo in lanes.descendants().filter(|n| n.has_tag_name("laneOffset")) {
            let poly3 = match poly3_from_attrs(lo, ("a", "b", "c", "d")) {
                Some(p) => p,
                None => continue,
            };
            let s_offset = match to_float(lo.attribute("s").unwrap_or("")) {
                Some(v) => v,
                None => continue,
            };
            let op = OffsetPoly3 {
                poly3,
                s_offset,
                xml_element: Some(lo),
            };
            if is_valid_offset_poly3(&op) {
                list.push(op);
            }
        }
    }
    list
}

/// Check if two cubic equations are the same. Mirrors are_same_equations.
pub fn are_same_equations(first: &OffsetPoly3, second: &OffsetPoly3) -> bool {
    let a3 = first.poly3.a - second.poly3.a - first.poly3.b * first.s_offset
        + second.poly3.b * second.s_offset
        + first.poly3.c * first.s_offset.powi(2)
        - second.poly3.c * second.s_offset.powi(2)
        - first.poly3.d * first.s_offset.powi(3)
        + second.poly3.d * second.s_offset.powi(3);
    let b3 = first.poly3.b - second.poly3.b
        - 2.0 * first.poly3.c * first.s_offset
        + 2.0 * second.poly3.c * second.s_offset
        + 3.0 * first.poly3.d * first.s_offset.powi(2)
        - 3.0 * second.poly3.d * second.s_offset.powi(2);
    let c3 = first.poly3.c - second.poly3.c
        - 3.0 * first.poly3.d * first.s_offset
        + 3.0 * second.poly3.d * second.s_offset;
    let d3 = first.poly3.d - second.poly3.d;
    a3.abs() < EPSILON && b3.abs() < EPSILON && c3.abs() < EPSILON && d3.abs() < EPSILON
}

pub fn get_road_plan_view_geometry_list<'a, 'input>(road: Node<'a, 'input>) -> Vec<Node<'a, 'input>> {
    road.children()
        .find(|n| n.has_tag_name("planView"))
        .map(|pv| pv.children().filter(|n| n.has_tag_name("geometry")).collect())
        .unwrap_or_default()
}

pub fn is_line_geometry(geometry: Node) -> bool {
    geometry.children().any(|n| n.has_tag_name("line"))
}

pub fn get_lane_direction(lane: Node) -> Option<LaneDirection> {
    match lane.attribute("direction") {
        None => Some(LaneDirection::Standard),
        Some(d) => LaneDirection::from_str(d),
    }
}

pub fn get_heading_from_geometry(geometry: Node) -> Option<f64> {
    to_float(geometry.attribute("hdg").unwrap_or(""))
}

pub fn get_s_from_geometry(geometry: Node) -> Option<f64> {
    to_float(geometry.attribute("s").unwrap_or(""))
}

pub fn get_x_from_geometry(geometry: Node) -> Option<f64> {
    to_float(geometry.attribute("x").unwrap_or(""))
}

pub fn get_y_from_geometry(geometry: Node) -> Option<f64> {
    to_float(geometry.attribute("y").unwrap_or(""))
}

pub fn get_geometry_from_road_by_s<'a, 'input>(road: Node<'a, 'input>, s: f64) -> Option<Node<'a, 'input>> {
    let length = get_road_length(road)?;
    if s < 0.0 || s > length {
        return None;
    }
    let geometries = get_road_plan_view_geometry_list(road);
    if geometries.is_empty() {
        return None;
    }
    let s_list: Vec<f64> = geometries
        .iter()
        .map(|g| get_s_from_geometry(*g).unwrap_or(0.0))
        .collect();
    let mut idx = s_list.partition_point(|&x| x <= s);
    if idx == 0 {
        idx = 0;
    } else {
        idx -= 1;
    }
    Some(geometries[idx])
}

pub fn get_geometry_arc<'a, 'input>(geometry: Node<'a, 'input>) -> Option<Node<'a, 'input>> {
    geometry.children().find(|n| n.has_tag_name("arc"))
}

pub fn get_geometry_line<'a, 'input>(geometry: Node<'a, 'input>) -> Option<Node<'a, 'input>> {
    geometry.children().find(|n| n.has_tag_name("line"))
}

pub fn get_geometry_spiral<'a, 'input>(geometry: Node<'a, 'input>) -> Option<Node<'a, 'input>> {
    geometry.children().find(|n| n.has_tag_name("spiral"))
}

pub fn calculate_line_point(s: f64, s0: f64, x0: f64, y0: f64, heading: f64) -> Point2D {
    Point2D {
        x: x0 + (s - s0) * heading.cos(),
        y: y0 + (s - s0) * heading.sin(),
    }
}

pub fn get_curvature_from_arc(arc: Node) -> Option<f64> {
    to_float(arc.attribute("curvature").unwrap_or(""))
}

pub fn calculate_arc_point(
    s: f64,
    s0: f64,
    x0: f64,
    y0: f64,
    heading: f64,
    curvature: f64,
) -> Point2D {
    let radius = 1.0 / curvature;
    let theta_f = (s - s0) * curvature - std::f64::consts::FRAC_PI_2;
    Point2D {
        x: x0 + radius * (theta_f + heading).cos() - radius * heading.sin(),
        y: y0 + radius * (theta_f + heading).sin() + radius * heading.cos(),
    }
}

pub fn get_curv_start_from_spiral(spiral: Node) -> Option<f64> {
    to_float(spiral.attribute("curvStart").unwrap_or(""))
}

pub fn get_curv_end_from_spiral(spiral: Node) -> Option<f64> {
    to_float(spiral.attribute("curvEnd").unwrap_or(""))
}

// ---------------------------------------------------------------------------
// Clothoid (spiral) geometry, ported from pyclothoids (G. Bertoli / MIT).
// ---------------------------------------------------------------------------

struct Clothoid {
    x0: f64,
    y0: f64,
    theta0: f64,
    k0: f64,
    k1: f64,
}

impl Clothoid {
    fn new(x0: f64, y0: f64, theta0: f64, k0: f64, k1: f64) -> Self {
        Clothoid {
            x0,
            y0,
            theta0,
            k0,
            k1,
        }
    }

    /// Curvature at arc-length `s` from start.
    fn heading(&self, s: f64) -> f64 {
        self.theta0 + self.k0 * s + 0.5 * self.k1 * s * s
    }

    /// Generalized Fresnel cosine/sine integrals.
    ///
    /// Faithful port of `G2lib::GeneralizedFresnelCS(a, b, c, C, S)` from the
    /// C++ `Clothoids` library (the one `pyclothoids` wraps). Computes
    ///
    ///   C = ∫_0^1 cos(a·u² + b·u + c) du
    ///   S = ∫_0^1 sin(a·u² + b·u + c) du
    ///
    /// via `evalXYaSmall`/`evalXYaLarge` + a rotation by `c`.
    fn generalized_fresnel_cs(a: f64, b: f64, c: f64) -> (f64, f64) {
        let (xx, yy) = if a.abs() < A_THRESOLD {
            Clothoid::eval_xy_a_small(a, b, A_SERIE_SIZE)
        } else {
            Clothoid::eval_xy_a_large(a, b)
        };
        let cosc = c.cos();
        let sinc = c.sin();
        (xx * cosc - yy * sinc, xx * sinc + yy * cosc)
    }

    /// `evalXYazero(nk, b, X0, Y0)` from the C++ source: fills `x0[0..nk]`,
    /// `y0[0..nk]`.
    fn eval_xy_azero(nk: usize, b: f64, x0: &mut [f64], y0: &mut [f64]) {
        let sb = b.sin();
        let cb = b.cos();
        let b2 = b * b;
        if b.abs() < 1e-3 {
            x0[0] = 1.0 - (b2 / 6.0) * (1.0 - (b2 / 20.0) * (1.0 - (b2 / 42.0)));
            y0[0] = (b / 2.0) * (1.0 - (b2 / 12.0) * (1.0 - (b2 / 30.0)));
        } else {
            x0[0] = sb / b;
            y0[0] = (1.0 - cb) / b;
        }
        // Use the recurrence in the stable part.
        let mut m = (2.0 * b).floor() as i32;
        if m >= nk as i32 {
            m = nk as i32 - 1;
        }
        if m < 1 {
            m = 1;
        }
        for k in 1..m as usize {
            x0[k] = (sb - k as f64 * y0[k - 1]) / b;
            y0[k] = (k as f64 * x0[k - 1] - cb) / b;
        }
        // Use Lommel for the unstable part.
        if (m as usize) < nk {
            let a = b * sb;
            let d = sb - b * cb;
            let bb = b * d;
            let c = -b2 * sb;
            let mut rla = Clothoid::lommel_reduced(m as f64 + 0.5, 1.5, b);
            let mut rld = Clothoid::lommel_reduced(m as f64 + 0.5, 0.5, b);
            for k in m as usize..nk {
                let rlb = Clothoid::lommel_reduced(k as f64 + 1.5, 0.5, b);
                let rlc = Clothoid::lommel_reduced(k as f64 + 1.5, 1.5, b);
                x0[k] = (k as f64 * a * rla + bb * rlb + cb) / (1.0 + k as f64);
                y0[k] = (c * rlc + sb) / (2.0 + k as f64) + d * rld;
                rla = rlc;
                rld = rlb;
            }
        }
    }

    /// `LommelReduced(mu, nu, b)` from the C++ source.
    fn lommel_reduced(mu: f64, nu: f64, b: f64) -> f64 {
        let mut tmp = 1.0 / ((mu + nu + 1.0) * (mu - nu + 1.0));
        let mut res = tmp;
        for n in 1..=100 {
            let nf = n as f64;
            tmp *= (-b / (2.0 * nf + mu - nu + 1.0)) * (b / (2.0 * nf + mu + nu + 1.0));
            res += tmp;
            if tmp.abs() < res.abs() * 1e-50 {
                break;
            }
        }
        res
    }

    /// `evalXYaSmall` (scalar `p` variant) from the C++ source.
    fn eval_xy_a_small(a: f64, b: f64, p: i32) -> (f64, f64) {
        let nkk = (4 * p + 3) as usize; // max 43
        let mut x0 = [0.0f64; 43];
        let mut y0 = [0.0f64; 43];
        Clothoid::eval_xy_azero(nkk, b, &mut x0, &mut y0);
        let mut x = x0[0] - (a / 2.0) * y0[2];
        let mut y = y0[0] + (a / 2.0) * x0[2];
        let mut t = 1.0f64;
        let aa = -a * a / 4.0;
        for n in 1..=p {
            t *= aa / ((2 * n * (2 * n - 1)) as f64);
            let bf = a / (4 * n + 2) as f64;
            let jj = (4 * n) as usize;
            x += t * (x0[jj] - bf * y0[jj + 2]);
            y += t * (y0[jj] + bf * x0[jj + 2]);
        }
        (x, y)
    }

    /// `evalXYaLarge` (scalar variant) from the C++ source.
    fn eval_xy_a_large(a: f64, b: f64) -> (f64, f64) {
        let s = if a > 0.0 { 1.0 } else { -1.0 };
        let absa = a.abs();
        let z = M_1_SQRT_PI * absa.sqrt();
        let ell = s * b * M_1_SQRT_PI / absa.sqrt();
        let g = -0.5 * s * (b * b) / absa;
        let cg = g.cos() / z;
        let sg = g.sin() / z;
        let (cl, sl) = Clothoid::fresnel_cs(ell);
        let (cz, sz) = Clothoid::fresnel_cs(ell + z);
        let dc0 = cz - cl;
        let ds0 = sz - sl;
        (cg * dc0 - s * sg * ds0, sg * dc0 + s * cg * ds0)
    }

    /// Fresnel integrals C(x) and S(x):
    ///   C(x) = ∫_0^x cos(t²) dt,  S(x) = ∫_0^x sin(t²) dt
    ///
    /// Faithful port of `G2lib::FresnelCS` from the C++ `Clothoids` library
    /// (A&S power series for x<1, rational approximation for 1≤x<6, asymptotic
    /// for x≥6). Returns (C(x), S(x)) with the correct sign for negative x.
    fn fresnel_cs(x: f64) -> (f64, f64) {
        let sgn = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();
        let (mut c, mut s) = if x < 1.0 {
            let s_arg = M_PI_2 * (x * x);
            let t = -s_arg * s_arg;
            // Cosine integral series.
            let mut twofn = 0.0f64;
            let mut fact = 1.0f64;
            let mut denterm = 1.0f64;
            let mut numterm = 1.0f64;
            let mut sum = 1.0f64;
            loop {
                twofn += 2.0;
                fact *= twofn * (twofn - 1.0);
                denterm += 4.0;
                numterm *= t;
                let term = numterm / (fact * denterm);
                sum += term;
                if term.abs() <= 1e-15 * sum.abs() {
                    break;
                }
            }
            let c = x * sum;
            // Sine integral series.
            let mut twofn = 1.0f64;
            let mut fact = 1.0f64;
            let mut denterm = 3.0f64;
            let mut numterm = 1.0f64;
            let mut sum = 1.0 / 3.0;
            loop {
                twofn += 2.0;
                fact *= twofn * (twofn - 1.0);
                denterm += 4.0;
                numterm *= t;
                let term = numterm / (fact * denterm);
                sum += term;
                if term.abs() <= 1e-15 * sum.abs() {
                    break;
                }
            }
            let s = M_PI_2 * sum * (x * x * x);
            (c, s)
        } else if x < 6.0 {
            // Rational approximation for f.
            let mut sumn = 0.0f64;
            let mut sumd = FD[11];
            for k in (0..=10).rev() {
                sumn = FN[k] + x * sumn;
                sumd = FD[k] + x * sumd;
            }
            let f = sumn / sumd;
            // Rational approximation for g.
            let mut sumn = 0.0f64;
            let mut sumd = GD[11];
            for k in (0..=10).rev() {
                sumn = GN[k] + x * sumn;
                sumd = GD[k] + x * sumd;
            }
            let g = sumn / sumd;
            let u = M_PI_2 * (x * x);
            let sinu = u.sin();
            let cosu = u.cos();
            (0.5 + f * sinu - g * cosu, 0.5 - f * cosu - g * sinu)
        } else {
            // x >= 6; asymptotic expansions for f and g.
            let s_arg = M_PI * x * x;
            let t = -1.0 / (s_arg * s_arg);
            // Expansion for f.
            let mut numterm = -1.0f64;
            let mut term = 1.0f64;
            let mut sum = 1.0f64;
            loop {
                numterm += 4.0;
                term *= numterm * (numterm - 2.0) * t;
                sum += term;
                if term.abs() <= 0.1e-15 * sum.abs() {
                    break;
                }
            }
            let f = sum / (M_PI * x);
            // Expansion for g.
            let mut numterm = -1.0f64;
            let mut term = 1.0f64;
            let mut sum = 1.0f64;
            loop {
                numterm += 4.0;
                term *= numterm * (numterm + 2.0) * t;
                sum += term;
                if term.abs() <= 0.1e-15 * sum.abs() {
                    break;
                }
            }
            let g = sum / ((M_PI * x) * (M_PI * x) * x);
            let u = M_PI_2 * (x * x);
            let sinu = u.sin();
            let cosu = u.cos();
            (0.5 + f * sinu - g * cosu, 0.5 - f * cosu - g * sinu)
        };
        if sgn < 0.0 {
            c = -c;
            s = -s;
        }
        (c, s)
    }

    /// Position at arc-length `s` from start.
    ///
    /// Faithful port of `G2lib::ClothoidData::eval`:
    ///   X(s) = x0 + s·C,  Y(s) = y0 + s·S
    /// where (C, S) = GeneralizedFresnelCS(k1·s², k0·s, θ0).
    /// This is exact for any start curvature `k0` (including `k0 != 0`),
    /// matching the C++ `Clothoids` library that `pyclothoids` wraps.
    fn point(&self, s: f64) -> Point2D {
        if s <= 0.0 {
            return Point2D {
                x: self.x0,
                y: self.y0,
            };
        }
        let sdk = s * self.k1;
        let (c, ss) = Clothoid::generalized_fresnel_cs(sdk * s, self.k0 * s, self.theta0);
        Point2D {
            x: self.x0 + s * c,
            y: self.y0 + s * ss,
        }
    }

    fn theta(&self, s: f64) -> f64 {
        self.heading(s)
    }
}

// Constants from the C++ `Clothoids` library (G2lib.cc / Numbers.hxx).
const A_THRESOLD: f64 = 0.01;
const A_SERIE_SIZE: i32 = 10;
const M_PI: f64 = 3.141592653589793238462643383279502884197;
const M_PI_2: f64 = 1.570796326794896619231321691639751442098;
const M_1_SQRT_PI: f64 = 0.564189583547756286948079451561; // 1/sqrt(pi)

// FresnelCS rational-approximation coefficients (Fresnel.cc fn/fd/gn/gd).
const FN: [f64; 12] = [
    0.49999988085884732562,
    1.3511177791210715095,
    1.3175407836168659241,
    1.1861149300293854992,
    0.7709627298888346769,
    0.4173874338787963957,
    0.19044202705272903923,
    0.06655998896627697537,
    0.022789258616785717418,
    0.0040116689358507943804,
    0.0012192036851249883877,
    0.0, // padding; FD/GD have length 12, FN/GN length 11
];
const FD: [f64; 12] = [
    1.0,
    2.7022305772400260215,
    4.2059268151438492767,
    4.5221882840107715516,
    3.7240352281630359588,
    2.4589286254678152943,
    1.3125491629443702962,
    0.5997685720120932908,
    0.20907680750378849485,
    0.07159621634657901433,
    0.012602969513793714191,
    0.0038302423512931250065,
];
const GN: [f64; 12] = [
    0.50000014392706344801,
    0.032346434925349128728,
    0.17619325157863254363,
    0.038606273170706486252,
    0.023693692309257725361,
    0.007092018516845033662,
    0.0012492123212412087428,
    0.00044023040894778468486,
    -8.80266827476172521e-6,
    -1.4033554916580018648e-8,
    2.3509221782155474353e-10,
    0.0,
];
const GD: [f64; 12] = [
    1.0,
    2.0646987497019598937,
    2.9109311766948031235,
    2.6561936751333032911,
    2.0195563983177268073,
    1.1167891129189363902,
    0.57267874755973172715,
    0.19408481169593070798,
    0.07634808341431248904,
    0.011573247407207865977,
    0.0044099273693067311209,
    -0.00009070958410429993314,
];

fn make_clothoid(
    x0: f64,
    y0: f64,
    theta0: f64,
    curv_start: f64,
    curv_end: f64,
    length: f64,
) -> Clothoid {
    let kd = (curv_end - curv_start) / length;
    Clothoid::new(x0, y0, theta0, curv_start, kd)
}

pub fn calculate_spiral_point(
    s: f64,
    s0: f64,
    x0: f64,
    y0: f64,
    heading: f64,
    curv_start: f64,
    curv_end: f64,
    length: f64,
) -> Point2D {
    let clothoid = make_clothoid(x0, y0, heading, curv_start, curv_end, length);
    clothoid.point(s - s0)
}

pub fn calculate_spiral_point_heading(
    s: f64,
    s0: f64,
    x0: f64,
    y0: f64,
    heading: f64,
    curv_start: f64,
    curv_end: f64,
    length: f64,
) -> f64 {
    let clothoid = make_clothoid(x0, y0, heading, curv_start, curv_end, length);
    clothoid.theta(s - s0)
}

pub fn calculate_poly3_arclen_point(
    s: f64,
    poly3_arclen: &ParamPoly3,
    s0: f64,
    x0: f64,
    y0: f64,
    heading: f64,
) -> Point2D {
    let x = poly3_arclen.u.evaluate(s - s0);
    let y = poly3_arclen.v.evaluate(s - s0);
    Point2D {
        x: (heading.cos() * x) - (heading.sin() * y) + x0,
        y: (heading.sin() * x) + (heading.cos() * y) + y0,
    }
}

pub fn calculate_poly3_norm_point(
    s: f64,
    poly3_norm: &ParamPoly3,
    s0: f64,
    x0: f64,
    y0: f64,
    heading: f64,
    length: f64,
) -> Point2D {
    let x = poly3_norm.u.evaluate((s - s0) / length);
    let y = poly3_norm.v.evaluate((s - s0) / length);
    Point2D {
        x: (heading.cos() * x) - (heading.sin() * y) + x0,
        y: (heading.sin() * x) + (heading.cos() * y) + y0,
    }
}

pub fn get_point_xy_from_geometry(geometry: Node, s: f64) -> Option<Point2D> {
    let x0 = get_x_from_geometry(geometry)?;
    let y0 = get_y_from_geometry(geometry)?;
    let s0 = get_s_from_geometry(geometry)?;
    let heading = get_heading_from_geometry(geometry)?;
    let length = get_length_from_geometry(geometry)?;

    if let Some(line) = get_geometry_line(geometry) {
        let _ = line;
        return Some(calculate_line_point(s, s0, x0, y0, heading));
    } else if let Some(arc) = get_geometry_arc(geometry) {
        let curvature = get_curvature_from_arc(arc)?;
        return Some(calculate_arc_point(s, s0, x0, y0, heading, curvature));
    } else if let Some(spiral) = get_geometry_spiral(geometry) {
        let curv_start = get_curv_start_from_spiral(spiral)?;
        let curv_end = get_curv_end_from_spiral(spiral)?;
        return Some(calculate_spiral_point(
            s, s0, x0, y0, heading, curv_start, curv_end, length,
        ));
    } else if let Some(pp) = get_arclen_param_poly3_from_geometry(geometry) {
        return Some(calculate_poly3_arclen_point(s, &pp, s0, x0, y0, heading));
    } else if let Some(pp) = get_normalized_param_poly3_from_geometry(geometry) {
        return Some(calculate_poly3_norm_point(s, &pp, s0, x0, y0, heading, length));
    }
    None
}

pub fn get_elevation_from_road_by_s<'a, 'input>(road: Node<'a, 'input>, s: f64) -> Option<OffsetPoly3<'a, 'input>> {
    let length = get_road_length(road)?;
    if s < 0.0 || s > length {
        return None;
    }
    let elevation_list = get_road_elevations(road);
    if elevation_list.is_empty() {
        return Some(zero_offset_poly3());
    }
    let s_list: Vec<f64> = elevation_list.iter().map(|e| e.s_offset).collect();
    let mut idx = s_list.partition_point(|&x| x <= s);
    if idx == 0 {
        idx = 0;
    } else {
        idx -= 1;
    }
    Some(elevation_list[idx])
}

pub fn zero_offset_poly3<'a, 'input>() -> OffsetPoly3<'a, 'input> {
    OffsetPoly3 {
        poly3: Poly3 {
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
        },
        s_offset: 0.0,
        xml_element: None,
    }
}

pub fn calculate_elevation_value(elevation: &OffsetPoly3, s: f64) -> f64 {
    elevation.poly3.evaluate(s - elevation.s_offset)
}

pub fn get_point_xy_from_road_reference_line(road: Node, s: f64) -> Option<Point2D> {
    let geometry = get_geometry_from_road_by_s(road, s)?;
    let p = get_point_xy_from_geometry(geometry, s)?;
    Some(Point2D { x: p.x, y: p.y })
}

pub fn get_point_xyz_from_road_reference_line(road: Node, s: f64) -> Option<Point3D> {
    let p2 = get_point_xy_from_road_reference_line(road, s)?;
    let elevation = get_elevation_from_road_by_s(road, s)?;
    let z = calculate_elevation_value(&elevation, s);
    Some(Point3D {
        x: p2.x,
        y: p2.y,
        z,
    })
}

pub fn get_start_point_xyz_from_road_reference_line(road: Node) -> Option<Point3D> {
    get_point_xyz_from_road_reference_line(road, 0.0)
}

pub fn get_end_point_xyz_from_road_reference_line(road: Node) -> Option<Point3D> {
    let end_s = get_road_length(road)?;
    get_point_xyz_from_road_reference_line(road, end_s)
}

pub fn get_middle_point_xyz_from_road_reference_line(road: Node) -> Option<Point3D> {
    let middle_s = get_road_length(road)? / 2.0;
    get_point_xyz_from_road_reference_line(road, middle_s)
}

pub fn get_heading_from_geometry_by_s(geometry: Node, s: f64) -> Option<f64> {
    let x0 = get_x_from_geometry(geometry)?;
    let y0 = get_y_from_geometry(geometry)?;
    let s0 = get_s_from_geometry(geometry)?;
    let heading = get_heading_from_geometry(geometry)?;
    let length = get_length_from_geometry(geometry)?;

    if get_geometry_line(geometry).is_some() {
        return Some(heading);
    } else if let Some(arc) = get_geometry_arc(geometry) {
        let curvature = get_curvature_from_arc(arc)?;
        return Some(calculate_arc_point_heading(s, s0, heading, curvature));
    } else if let Some(spiral) = get_geometry_spiral(geometry) {
        let curv_start = get_curv_start_from_spiral(spiral)?;
        let curv_end = get_curv_end_from_spiral(spiral)?;
        return Some(calculate_spiral_point_heading(
            s, s0, x0, y0, heading, curv_start, curv_end, length,
        ));
    } else if let Some(pp) = get_arclen_param_poly3_from_geometry(geometry) {
        return Some(calculate_poly3_arclen_heading(s, &pp, s0, heading));
    } else if let Some(pp) = get_normalized_param_poly3_from_geometry(geometry) {
        return Some(calculate_poly3_norm_heading(s, &pp, s0, heading, length));
    }
    None
}

pub fn calculate_arc_point_heading(s: f64, s0: f64, heading: f64, curvature: f64) -> f64 {
    heading + curvature * (s - s0)
}

pub fn calculate_poly3_arclen_heading(s: f64, poly3_arclen: &ParamPoly3, s0: f64, heading: f64) -> f64 {
    let du = poly3_arclen.u.deriv(s - s0);
    let dv = poly3_arclen.v.deriv(s - s0);
    heading + dv.atan2(du)
}

pub fn calculate_poly3_norm_heading(
    s: f64,
    poly3_norm: &ParamPoly3,
    s0: f64,
    heading: f64,
    length: f64,
) -> f64 {
    let du = poly3_norm.u.deriv((s - s0) / length);
    let dv = poly3_norm.v.deriv((s - s0) / length);
    heading + dv.atan2(du)
}

pub fn get_heading_from_road_reference_line(road: Node, s: f64) -> Option<f64> {
    let geometry = get_geometry_from_road_by_s(road, s)?;
    get_heading_from_geometry_by_s(geometry, s)
}

pub fn calculate_elevation_angle(elevation: &OffsetPoly3, s: f64) -> Option<f64> {
    let ds = elevation.poly3.deriv(s - elevation.s_offset);
    if ds.is_nan() {
        return None;
    }
    Some(ds.atan())
}

pub fn get_pitch_from_road_reference_line(road: Node, s: f64) -> Option<f64> {
    let elevation = get_elevation_from_road_by_s(road, s)?;
    Some(-calculate_elevation_angle(&elevation, s)?)
}

pub fn get_superelevation_from_road_by_s<'a, 'input>(road: Node<'a, 'input>, s: f64) -> Option<OffsetPoly3<'a, 'input>> {
    let length = get_road_length(road)?;
    if s < 0.0 || s > length {
        return None;
    }
    let se_list = get_road_superelevations(road);
    if se_list.is_empty() {
        return Some(zero_offset_poly3());
    }
    let s_list: Vec<f64> = se_list.iter().map(|e| e.s_offset).collect();
    let mut idx = s_list.partition_point(|&x| x <= s);
    if idx == 0 {
        idx = 0;
    } else {
        idx -= 1;
    }
    Some(se_list[idx])
}

pub fn get_roll_from_road_reference_line(road: Node, s: f64) -> Option<f64> {
    let se = get_superelevation_from_road_by_s(road, s)?;
    Some(se.poly3.evaluate(s - se.s_offset))
}

/// 3D point on a lane given (s, t, h) offsets. Uses yaw/roll rotation.
pub fn get_point_xyz_from_road(road: Node, s: f64, t: f64, h: f64) -> Option<Point3D> {
    let yaw = get_heading_from_road_reference_line(road, s)?;
    let roll = get_roll_from_road_reference_line(road, s)?;
    // rotation about z (yaw) then x (roll): R = Rz(yaw) * Rx(roll)
    let cy = yaw.cos();
    let sy = yaw.sin();
    let cr = roll.cos();
    let sr = roll.sin();
    // Rz * Rx
    let r00 = cy;
    let r01 = -sy * cr;
    let r02 = sy * sr;
    let r10 = sy;
    let r11 = cy * cr;
    let r12 = -cy * sr;
    let r20 = 0.0;
    let r21 = sr;
    let r22 = cr;
    let dx = 0.0 * r00 + t * r01 + h * r02;
    let dy = 0.0 * r10 + t * r11 + h * r12;
    let dz = 0.0 * r20 + t * r21 + h * r22;
    let ref_line = get_point_xyz_from_road_reference_line(road, s)?;
    Some(Point3D {
        x: ref_line.x + dx,
        y: ref_line.y + dy,
        z: ref_line.z + dz,
    })
}

pub fn get_lane_section_from_road_by_s<'a, 'input>(road: Node<'a, 'input>, s: f64) -> Option<Node<'a, 'input>> {
    let length = get_road_length(road)?;
    if s < 0.0 || s > length {
        return None;
    }
    let lane_sections = get_lane_sections(road);
    if lane_sections.is_empty() {
        return None;
    }
    let s_list: Vec<f64> = lane_sections
        .iter()
        .map(|l| get_s_from_lane_section(*l).unwrap_or(0.0))
        .collect();
    let mut idx = s_list.partition_point(|&x| x <= s);
    if idx == 0 {
        idx = 0;
    } else {
        idx -= 1;
    }
    Some(lane_sections[idx])
}

pub fn get_lane_offset_from_road_by_s<'a, 'input>(road: Node<'a, 'input>, s: f64) -> Option<OffsetPoly3<'a, 'input>> {
    let length = get_road_length(road)?;
    if s < 0.0 || s > length {
        return None;
    }
    let lane_offset_list = get_lane_offsets_from_road(road);
    if lane_offset_list.is_empty() {
        return Some(zero_offset_poly3());
    }
    let s_list: Vec<f64> = lane_offset_list.iter().map(|l| l.s_offset).collect();
    let mut idx = s_list.partition_point(|&x| x <= s);
    if idx == 0 {
        // s < first s_offset -> zero
        return Some(zero_offset_poly3());
    }
    idx -= 1;
    Some(lane_offset_list[idx])
}

pub fn get_lane_offset_value_from_road_by_s(road: Node, s: f64) -> Option<f64> {
    let lo = get_lane_offset_from_road_by_s(road, s)?;
    Some(lo.poly3.evaluate(s - lo.s_offset))
}

pub fn evaluate_lane_border(lane: Node, s_start_from_lane_section: f64) -> Option<f64> {
    let lane_id = get_lane_id(lane)?;
    if lane_id == 0 {
        return Some(0.0);
    }
    let lane_border_poly3_list = get_borders_from_lane(lane);
    if lane_border_poly3_list.is_empty() {
        return None;
    }
    let mut count = 0;
    for b in &lane_border_poly3_list {
        if b.s_offset > s_start_from_lane_section {
            break;
        }
        count += 1;
    }
    if count == 0 {
        return None;
    }
    let index = count - 1;
    let poly3 = &lane_border_poly3_list[index].poly3;
    Some(poly3.evaluate(s_start_from_lane_section - lane_border_poly3_list[index].s_offset))
}

pub fn get_outer_border_points_from_lane_group_by_s(
    lane_group: &[Node],
    lane_offset: f64,
    s_section: f64,
    s: f64,
) -> std::collections::HashMap<i64, Option<f64>> {
    let mut id_to_width: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
    let mut id_to_border_point_t: std::collections::HashMap<i64, Option<f64>> = std::collections::HashMap::new();
    for lane in lane_group {
        let lane_id = match get_lane_id(*lane) {
            Some(id) => id,
            None => continue,
        };
        let width = evaluate_lane_width(*lane, s - s_section);
        if width.is_none() {
            let border_point_t = evaluate_lane_border(*lane, s - s_section);
            id_to_border_point_t.insert(lane_id, border_point_t);
        } else {
            id_to_width.insert(lane_id, width.unwrap());
        }
    }
    if !id_to_width.is_empty() {
        id_to_border_point_t.clear();
        for (lane_id, _) in &id_to_width {
            let border_t = if *lane_id > 0 {
                let mut bt = lane_offset;
                for i in 1..=(*lane_id) {
                    if let Some(w) = id_to_width.get(&i) {
                        bt += *w;
                    }
                }
                bt
            } else {
                let mut bt = lane_offset;
                for i in (*lane_id..=-1).rev() {
                    if let Some(w) = id_to_width.get(&i) {
                        bt -= *w;
                    }
                }
                bt
            };
            id_to_border_point_t.insert(*lane_id, Some(border_t));
        }
    }
    id_to_border_point_t
}

pub fn get_t_middle_point_from_lane_by_s(
    road: Node,
    lane_section: Node,
    lane: Node,
    s: f64,
) -> Option<f64> {
    let lane_id = get_lane_id(lane)?;
    let lane_offset = get_lane_offset_value_from_road_by_s(road, s)?;
    let s_section = get_s_from_lane_section(lane_section)?;
    if lane_id == 0 {
        return Some(0.0);
    }
    if lane_id > 0 {
        let left_lanes = get_left_lanes_from_lane_section(lane_section);
        let border_points = get_outer_border_points_from_lane_group_by_s(&left_lanes, lane_offset, s_section, s);
        let t_outer = border_points.get(&lane_id).and_then(|o| *o)?;
        let t_inner = if lane_id > 1 {
            border_points.get(&(lane_id - 1)).and_then(|o| *o)?
        } else {
            lane_offset
        };
        Some((t_outer + t_inner) / 2.0)
    } else {
        let right_lanes = get_right_lanes_from_lane_section(lane_section);
        let border_points = get_outer_border_points_from_lane_group_by_s(&right_lanes, lane_offset, s_section, s);
        let t_outer = border_points.get(&lane_id).and_then(|o| *o)?;
        let t_inner = if lane_id < -1 {
            border_points.get(&(lane_id + 1)).and_then(|o| *o)?
        } else {
            lane_offset
        };
        Some((t_outer + t_inner) / 2.0)
    }
}

pub fn get_middle_point_xyz_at_height_zero_from_lane_by_s(
    road: Node,
    lane_section: Node,
    lane: Node,
    s: f64,
) -> Option<Point3D> {
    let t = get_t_middle_point_from_lane_by_s(road, lane_section, lane, s)?;
    get_point_xyz_from_road(road, s, t, 0.0)
}

pub fn get_s_offset_from_access(access: Node) -> Option<f64> {
    to_float(access.attribute("sOffset").unwrap_or(""))
}

pub fn get_point_xyz_from_contact_point(road: Node, contact_point: ContactPoint) -> Option<Point3D> {
    match contact_point {
        ContactPoint::Start => get_start_point_xyz_from_road_reference_line(road),
        ContactPoint::End => get_end_point_xyz_from_road_reference_line(road),
    }
}
