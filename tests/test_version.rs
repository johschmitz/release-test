// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Port of qc-opendrive/tests/test_version.py.

use xodr_qcr::version;

#[test]
fn test_has_lower_bound() {
    let cases = [
        (">1.0.0", true),
        (">1.0.0,<2.0.0", true),
        (">1.0.0,<=2.0.0", true),
        (">=1.0.0", true),
        (">=1.0.0,<2.0.0", true),
        (">=1.0.0,<=2.0.0", true),
        (">=1.0.0,>=1.0.1,<=2.0.0", true),
        (">=1.0.0,<=1.0.1,<=2.0.0", true),
        ("<=2.0.0", false),
        ("<2.0.0", false),
        ("<=2.0.0,<=3.0.0", false),
        ("<2.0.0,<3.0.0", false),
        ("", false),
    ];
    for (applicable_version, expected) in cases {
        assert_eq!(
            version::has_lower_bound(applicable_version),
            expected,
            "has_lower_bound({applicable_version})"
        );
    }
}

#[test]
fn test_match() {
    let cases = [
        ("1.7.0", ">=1.7.0", true),
        ("1.7.0", "<=1.7.0", true),
        ("1.7.0", ">1.7.0", false),
        ("1.7.0", "<1.7.0", false),
        ("1.7.0", "<1.7.0,<1.8.0", false),
        ("1.7.0", ">1.7.0,>1.6.0", false),
        ("1.7.0", ">=1.7.0,>1.6.0", true),
        ("1.7.0", ">=1.7.0,>1.8.0", false),
        ("1.7.0", "<=1.7.0,>1.8.0", false),
        ("1.7.0", "<=1.7.0,<1.8.0", true),
        ("1.7.0", "<=1.7.0,<1.6.0", false),
        ("1.7.0", "<=1.7.0,<1.8", false),
        ("1.7.0", "", true),
    ];
    for (version, applicable_version, expected) in cases {
        assert_eq!(
            version::matches(version, applicable_version),
            expected,
            "matches({version}, {applicable_version})"
        );
    }
}

#[test]
fn test_is_valid_version_expression() {
    let cases = [
        ("1.7.0", false),
        (">1.7.0", true),
        (">=1.7.0", true),
        ("<1.7.0", true),
        ("<=1.7.0", true),
        ("==1.7.0", false),
        ("!=1.7.0", false),
        ("<1.7", false),
        ("", true),
    ];
    for (version_expression, expected) in cases {
        assert_eq!(
            version::is_valid_version_expression(version_expression),
            expected,
            "is_valid_version_expression({version_expression})"
        );
    }
}
