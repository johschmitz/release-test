// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// OpenDRIVE data model, ported from qc_opendrive/base/models.py.

use roxmltree::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkageTag {
    Predecessor,
    Successor,
}

impl LinkageTag {
    pub fn value(self) -> &'static str {
        match self {
            LinkageTag::Predecessor => "predecessor",
            LinkageTag::Successor => "successor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactPoint {
    Start,
    End,
}

impl ContactPoint {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "start" => Some(ContactPoint::Start),
            "end" => Some(ContactPoint::End),
            _ => None,
        }
    }

    pub fn value(self) -> &'static str {
        match self {
            ContactPoint::Start => "start",
            ContactPoint::End => "end",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RoadLinkage {
    pub id: i64,
    pub contact_point: ContactPoint,
}

#[derive(Debug, Clone, Copy)]
pub struct Poly3 {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

impl Poly3 {
    pub fn evaluate(&self, x: f64) -> f64 {
        self.a + self.b * x + self.c * x * x + self.d * x * x * x
    }

    /// First derivative at x.
    pub fn deriv(&self, x: f64) -> f64 {
        self.b + 2.0 * self.c * x + 3.0 * self.d * x * x
    }

    /// Third-order polynomial coefficients [a, b, c, d] (constant term first),
    /// matching numpy.polynomial.Polynomial ordering.
    pub fn coeffs(&self) -> [f64; 4] {
        [self.a, self.b, self.c, self.d]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamPoly3Range {
    ArcLength,
    Normalized,
}

impl ParamPoly3Range {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "arcLength" => Some(ParamPoly3Range::ArcLength),
            "normalized" => Some(ParamPoly3Range::Normalized),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParamPoly3 {
    pub u: Poly3,
    pub v: Poly3,
    pub range: ParamPoly3Range,
}

#[derive(Debug, Clone, Copy)]
pub struct ContactingLaneSection<'a, 'input> {
    pub lane_section: Node<'a, 'input>,
    pub linkage_tag: LinkageTag,
}

#[derive(Debug, Clone, Copy)]
pub struct ContactingLaneSections<'a, 'input> {
    pub incoming: Node<'a, 'input>,
    pub connection: Node<'a, 'input>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficHandRule {
    LHT,
    RHT,
}

impl TrafficHandRule {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "LHT" => Some(TrafficHandRule::LHT),
            "RHT" => Some(TrafficHandRule::RHT),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LaneSectionWithLength<'a, 'input> {
    pub lane_section: Node<'a, 'input>,
    pub length: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct OffsetPoly3<'a, 'input> {
    pub poly3: Poly3,
    pub s_offset: f64,
    pub xml_element: Option<Node<'a, 'input>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneDirection {
    Standard,
    Reversed,
    Both,
}

impl LaneDirection {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "standard" => Some(LaneDirection::Standard),
            "reversed" => Some(LaneDirection::Reversed),
            "both" => Some(LaneDirection::Both),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}
