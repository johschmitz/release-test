// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Port of qc-opendrive/tests/test_utils.py.
//
// The Rust API has no `get_root_without_default_namespace` (it parses directly),
// so the namespace test is adapted to assert `get_roads` returns a non-empty
// collection for both a namespaced and a non-namespaced file. Numeric tolerances
// for `get_point_xyz_from_road` are kept at 1e-6 to match the Python reference.

use roxmltree::Document;
use xodr_qcr::opendrive::utils;

fn parse(path: &str) -> Document<'static> {
    let text = std::fs::read_to_string(path).unwrap();
    // Leak the text first so the Document can borrow a 'static str for the
    // duration of the test. This mirrors the Python pattern where the parsed
    // tree outlives the helper call.
    let leaked: &'static str = Box::leak(text.into_boxed_str());
    Document::parse(leaked).unwrap()
}

#[test]
fn test_get_roads_non_empty() {
    // file containing namespace
    let doc = parse("tests/data/utils/namespace.xodr");
    assert!(!utils::get_roads(&doc).is_empty());

    // file does not contain namespace
    let doc = parse("tests/data/utils/Ex_Bidirectional_Junction.xodr");
    assert!(!utils::get_roads(&doc).is_empty());
}

#[test]
fn test_get_road_id_map() {
    let doc = parse("tests/data/utils/Ex_Bidirectional_Junction.xodr");
    let road_id_map = utils::get_road_id_map(&doc);
    assert_eq!(road_id_map.len(), 6);
}

#[test]
fn test_get_junction_id_map() {
    let doc = parse("tests/data/utils/Ex_Bidirectional_Junction.xodr");
    let junction_id_map = utils::get_junction_id_map(&doc);
    assert_eq!(junction_id_map.len(), 1);
}

#[test]
fn test_get_point_xyz_from_road_invalid_s() {
    let doc = parse("tests/data/utils/simple_line.xodr");
    let road = utils::get_roads(&doc)[0];

    let point = utils::get_point_xyz_from_road(road, -0.001, -10.0, -20.0);
    assert!(point.is_none());

    let point = utils::get_point_xyz_from_road(road, 100.001, -10.0, -20.0);
    assert!(point.is_none());
}

#[test]
fn test_get_point_xyz_from_road() {
    let cases: &[(&str, f64, f64, f64, f64, f64, f64)] = &[
        ("simple_line.xodr", 30.0, -10.0, -20.0, 30.0, -10.0, -20.0),
        ("simple_line_heading.xodr", 30.0, 10.0, 20.0, -10.0, 30.0, 20.0),
        ("Ex_Line-Spiral-Arc.xodr", 0.0, 0.0, 0.0, -56.53979238754325, -34.39446366782007, 0.0),
        ("Ex_Line-Spiral-Arc.xodr", 230.0, 0.0, 0.0, 111.21223886865663, 94.90682833835331, 0.0),
        ("Ex_Line-Spiral-Arc.xodr", 230.0, 10.0, 0.0, 101.26793707002476, 95.96080259692272, 0.0),
        ("Ex_Line-Spiral-Arc.xodr", 230.0, -10.0, 0.0, 121.1565406672885, 93.8528540797839, 0.0),
        ("Ex_Line-Spiral-Arc.xodr", 120.0, 10.0, 0.0, 52.61995309748058, 14.38548916295234, 0.0),
        ("Ex_Line-Spiral-Arc.xodr", 120.0, -10.0, 0.0, 60.789014235577646, -3.870097382596728, 0.0),
        ("Ex_Line-Spiral-Arc.xodr", 150.0, 10.0, 0.0, 74.12961916583495, 29.09285535716386, 0.0),
        ("Ex_Line-Spiral-Arc.xodr", 150.0, 10.0, 20.0, 74.12961916583495, 29.09285535716386, 20.0),
        ("Ex_Line-Spiral-Arc.xodr", 150.0, -10.0, 0.0, 88.45632996588242, 15.137735950587523, 0.0),
        ("simple_line_elevation.xodr", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        ("simple_line_elevation.xodr", 5.0, 0.0, 0.0, 5.0, 0.0, 5.0),
        ("simple_line_elevation.xodr", 5.0, 10.0, 0.0, 5.0, 10.0, 5.0),
        ("simple_line_elevation.xodr", 5.0, -10.0, 0.0, 5.0, -10.0, 5.0),
        ("simple_line_heading_and_elevation.xodr", 0.0, 5.0, 0.0, -5.0, 0.0, 0.0),
        ("simple_line_heading_and_elevation.xodr", 0.0, -5.0, 0.0, 5.0, 0.0, 0.0),
        ("simple_line_heading_and_elevation.xodr", 20.0, -5.0, 0.0, 5.0, 20.0, 20.0),
        ("simple_line_heading_and_elevation.xodr", 20.0, 5.0, 0.0, -5.0, 20.0, 20.0),
        ("Ex_Line-Spiral-Arc_elevation.xodr", 150.0, 10.0, 0.0, 74.12961916583495, 29.09285535716386, 150.0),
        ("Ex_Line-Spiral-Arc_elevation.xodr", 150.0, -10.0, 0.0, 88.45632996588242, 15.137735950587523, 150.0),
        ("simple_line_elevation.xodr", 0.0, 0.0, 10.0, 0.0, 0.0, 10.0),
        ("simple_line_elevation.xodr", 0.0, 0.0, -10.0, 0.0, 0.0, -10.0),
        ("simple_line_superelevation.xodr", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        ("simple_line_superelevation.xodr", 0.0, 5.0, 0.0, 0.0, 3.535534483629909, 3.5355333282354717),
        ("simple_line_superelevation.xodr", 0.0, -5.0, 0.0, 0.0, -3.535534483629909, -3.5355333282354717),
        ("simple_line_superelevation.xodr", 50.0, 5.0, 0.0, 50.0, 3.535534483629909, 3.5355333282354717),
        ("simple_line_superelevation.xodr", 50.0, -5.0, 0.0, 50.0, -3.535534483629909, -3.5355333282354717),
        ("simple_line_superelevation.xodr", 0.0, 0.0, 10.0, 0.0, -7.0710666564709435, 7.071068967259818),
        ("simple_line_heading_and_elevation_and_superelevation.xodr", 50.0, 5.0, 0.0, -3.5355344833850797, 50.0, 53.5355333282354717),
        ("simple_line_heading_and_elevation_and_superelevation.xodr", 50.0, -5.0, 0.0, 3.53553448388698, 50.0, 50.0 - 3.5355333282354717),
        ("Ex_Line-Spiral-Arc_elevation_and_superelevation.xodr", 150.0, -5.0, 0.0, 83.82560356938673, 19.64835535961951, 150.0 - 3.5355333282354717),
        ("Ex_Line-Spiral-Arc_elevation_and_superelevation.xodr", 150.0, 5.0, 0.0, 78.76034556233064, 24.58223594813187, 153.5355333282354717),
        ("Ex_Line-Spiral-Arc_elevation_and_superelevation.xodr", 150.0, 5.0, 10.0, 83.82560191408653, 19.648356971986246, 160.6066022954953),
        ("Ex_Line-Spiral-Arc_superelevation.xodr", 150.0, -5.0, 0.0, 83.82560356938673, 19.64835535961951, -3.5355333282354717),
        ("Ex_Line-Spiral-Arc_superelevation.xodr", 150.0, 5.0, 0.0, 78.76034556233064, 24.58223594813187, 3.5355333282354717),
        ("Ex_Line-Spiral-Arc_superelevation.xodr", 150.0, 5.0, 10.0, 83.82560191408653, 19.648356971986246, 10.60660229549529),
    ];

    for (file_name, s, t, h, x, y, z) in cases {
        let path = format!("tests/data/utils/{file_name}");
        let doc = parse(&path);
        let road = utils::get_roads(&doc)[0];
        let point = utils::get_point_xyz_from_road(road, *s, *t, *h);
        let point = point.expect(&format!("expected a point for {file_name} s={s}"));
        assert!((point.x - x).abs() <= 1e-6, "{file_name} s={s} t={t} h={h}: x {} != {x}", point.x);
        assert!((point.y - y).abs() <= 1e-6, "{file_name} s={s} t={t} h={h}: y {} != {y}", point.y);
        assert!((point.z - z).abs() <= 1e-6, "{file_name} s={s} t={t} h={h}: z {} != {z}", point.z);
    }
}
