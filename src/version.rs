// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Version gating + RuleUid parsing, ported from qc_opendrive/version.py.

use anyhow::{bail, Result};
use semver::{Version, VersionReq};

/// Parsed rule UID: `emanating_entity:standard:definition_setting:rule_full_name`.
#[derive(Debug, Clone)]
pub struct RuleUid {
    pub emanating_entity: String,
    pub standard: String,
    pub definition_setting: String,
    pub rule_full_name: String,
    pub rule_uid: String,
}

impl RuleUid {
    pub fn parse(rule_uid: &str) -> Result<Self> {
        let parts: Vec<&str> = rule_uid.splitn(4, ':').collect();
        if parts.len() != 4 {
            bail!("Invalid rule uid: {rule_uid}");
        }
        Ok(RuleUid {
            emanating_entity: parts[0].to_string(),
            standard: parts[1].to_string(),
            definition_setting: parts[2].to_string(),
            rule_full_name: parts[3].to_string(),
            rule_uid: rule_uid.to_string(),
        })
    }
}

fn is_lower_bound(expression: &str) -> bool {
    expression.starts_with('>')
}

/// Split a comma-separated version expression into clauses, dropping spaces.
fn get_version_clauses(applicable_versions: &str) -> Vec<String> {
    applicable_versions
        .split(',')
        .map(|vc| vc.split_whitespace().collect::<String>())
        .filter(|vc| !vc.is_empty())
        .collect()
}

/// A clause is valid if it matches `^[<>]=?\d+\.\d+\.\d+$`.
fn is_valid_clause(clause: &str) -> bool {
    let bytes = clause.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    // Must start with a comparison operator.
    if bytes[0] != b'<' && bytes[0] != b'>' {
        return false;
    }
    let mut i = 1;
    if i < bytes.len() && bytes[i] == b'=' {
        i += 1;
    }
    // remainder must be x.y.z
    let rest = &clause[i..];
    let nums: Vec<&str> = rest.split('.').collect();
    if nums.len() != 3 {
        return false;
    }
    nums.iter().all(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

pub fn is_valid_version_expression(version_expression: &str) -> bool {
    let clauses = get_version_clauses(version_expression);
    if clauses.is_empty() {
        return version_expression.trim().is_empty();
    }
    clauses.iter().all(|c| is_valid_clause(c))
}

pub fn has_lower_bound(applicable_versions: &str) -> bool {
    get_version_clauses(applicable_versions).iter().any(|c| is_lower_bound(c))
}

/// Match a version against a comma-separated applicable-version expression.
/// All clauses must match (logical AND). Invalid clauses force a mismatch.
pub fn matches(version: &str, applicable_versions: &str) -> bool {
    let Ok(v) = Version::parse(version) else {
        return false;
    };
    let clauses = get_version_clauses(applicable_versions);
    if clauses.is_empty() {
        return true;
    }
    clauses.iter().all(|clause| {
        if !is_valid_clause(clause) {
            return false;
        }
        match VersionReq::parse(clause) {
            Ok(req) => req.matches(&v),
            Err(_) => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_uid_parse() {
        let r = RuleUid::parse("asam.net:xodr:1.0.0:xml.valid_xml_document").unwrap();
        assert_eq!(r.emanating_entity, "asam.net");
        assert_eq!(r.standard, "xodr");
        assert_eq!(r.definition_setting, "1.0.0");
        assert_eq!(r.rule_full_name, "xml.valid_xml_document");
    }

    #[test]
    fn test_is_valid_clause() {
        assert!(is_valid_clause(">=1.0.0"));
        assert!(is_valid_clause("<1.2.3"));
        assert!(is_valid_clause(">0.0.1"));
        assert!(!is_valid_clause("1.0.0"));
        assert!(!is_valid_clause(">=1.0"));
        assert!(!is_valid_clause("foo"));
    }

    #[test]
    fn test_has_lower_bound() {
        assert!(has_lower_bound("<1.0.0,>0.0.1"));
        assert!(!has_lower_bound("<1.0.0"));
    }

    #[test]
    fn test_matches() {
        assert!(matches("1.7.0", ">=1.0.0"));
        assert!(matches("1.8.0", ">=1.0.0"));
        assert!(!matches("1.0.0", ">=1.1.0"));
        assert!(matches("1.4.0", "<1.5.0,>=1.0.0"));
        assert!(!matches("1.6.0", "<1.5.0"));
    }

    #[test]
    fn test_is_valid_version_expression() {
        assert!(is_valid_version_expression(">=1.0.0"));
        assert!(is_valid_version_expression("<1.0.0,>0.0.1"));
        assert!(!is_valid_version_expression(">=1.0"));
    }
}
