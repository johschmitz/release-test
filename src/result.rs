// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Core result / checker-bundle data model, ported from qc_baselib/result.py.

/// Severity of an issue. Mirrors `qc_baselib.IssueSeverity`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IssueSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
}

impl IssueSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueSeverity::Error => "error",
            IssueSeverity::Warning => "warning",
            IssueSeverity::Information => "information",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            IssueSeverity::Error => "ERROR",
            IssueSeverity::Warning => "WARNING",
            IssueSeverity::Information => "INFORMATION",
        }
    }
}

/// Status of a checker run. Mirrors `qc_baselib.StatusType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum StatusType {
    Completed,
    Skipped,
    Error,
    #[default]
    None,
}

impl StatusType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusType::Completed => "completed",
            StatusType::Skipped => "skipped",
            StatusType::Error => "error",
            StatusType::None => "none",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            StatusType::Completed => "COMPLETED",
            StatusType::Skipped => "SKIPPED",
            StatusType::Error => "ERROR",
            StatusType::None => "NONE",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Default)]
pub struct FileLocation {
    pub row: Option<u32>,
    pub column: Option<u32>,
    pub offset: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct XmlLocation {
    pub xpath: String,
}

#[derive(Clone, Debug, Default)]
pub struct InertialLocation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Location {
    pub description: String,
    pub file: Vec<FileLocation>,
    pub xml: Vec<XmlLocation>,
    pub inertial: Vec<InertialLocation>,
}

#[derive(Clone, Debug)]
pub struct RuleUid {
    pub rule_uid: String,
}

#[derive(Clone, Debug)]
pub struct Issue {
    pub issue_id: u32,
    pub description: String,
    pub level: IssueSeverity,
    pub rule_uid: String,
    pub locations: Vec<Location>,
}

#[derive(Clone, Debug, Default)]
pub struct Checker {
    pub checker_id: String,
    pub description: String,
    pub status: Option<StatusType>,
    pub summary: String,
    pub addressed_rules: Vec<RuleUid>,
    pub issues: Vec<Issue>,
    pub params: Vec<Param>,
}

impl Checker {
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CheckerBundle {
    pub name: String,
    pub version: String,
    pub description: String,
    pub summary: String,
    pub checkers: Vec<Checker>,
    pub params: Vec<Param>,
}

#[derive(Debug, Default)]
pub struct Result {
    pub bundle: CheckerBundle,
    pub next_id: u32,
}

impl Result {
    pub fn new() -> Self {
        Result {
            bundle: CheckerBundle::default(),
            next_id: 1,
        }
    }

    pub fn register_checker_bundle(
        &mut self,
        name: &str,
        description: &str,
        version: &str,
        summary: &str,
    ) {
        self.bundle.name = name.to_string();
        self.bundle.description = description.to_string();
        self.bundle.version = version.to_string();
        self.bundle.summary = summary.to_string();
    }

    pub fn set_result_version(&mut self, version: &str) {
        self.bundle.version = version.to_string();
    }

    pub fn register_checker(&mut self, checker_id: &str, description: &str) {
        if self.bundle.checkers.iter().any(|c| c.checker_id == checker_id) {
            return;
        }
        self.bundle.checkers.push(Checker {
            checker_id: checker_id.to_string(),
            description: description.to_string(),
            status: None,
            summary: String::new(),
            addressed_rules: Vec::new(),
            issues: Vec::new(),
            params: Vec::new(),
        });
    }

    pub fn register_rule_by_uid(&mut self, checker_id: &str, rule_uid: &str) {
        if let Some(c) = self
            .bundle
            .checkers
            .iter_mut()
            .find(|c| c.checker_id == checker_id)
        {
            if !c.addressed_rules.iter().any(|r| r.rule_uid == rule_uid) {
                c.addressed_rules.push(RuleUid {
                    rule_uid: rule_uid.to_string(),
                });
            }
        }
    }

    /// Returns the new issue id.
    pub fn register_issue(
        &mut self,
        checker_id: &str,
        description: &str,
        level: IssueSeverity,
        rule_uid: &str,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        if let Some(c) = self
            .bundle
            .checkers
            .iter_mut()
            .find(|c| c.checker_id == checker_id)
        {
            c.issues.push(Issue {
                issue_id: id,
                description: description.to_string(),
                level,
                rule_uid: rule_uid.to_string(),
                locations: Vec::new(),
            });
        }
        id
    }

    pub fn add_file_location(
        &mut self,
        checker_id: &str,
        issue_id: u32,
        row: Option<u32>,
        column: Option<u32>,
        offset: Option<u64>,
        description: &str,
    ) {
        if let Some(c) = self
            .bundle
            .checkers
            .iter_mut()
            .find(|c| c.checker_id == checker_id)
        {
            if let Some(issue) = c.issues.iter_mut().find(|i| i.issue_id == issue_id) {
                let loc: Option<()> = if let Some(last) = issue.locations.last_mut() {
                    if last.file.is_empty() && last.xml.is_empty() && last.inertial.is_empty() {
                        last.description = description.to_string();
                        last.file.push(FileLocation { row, column, offset });
                        return;
                    }
                    None
                } else {
                    None
                };
                if loc.is_none() {
                    issue.locations.push(Location {
                        description: description.to_string(),
                        file: vec![FileLocation { row, column, offset }],
                        xml: Vec::new(),
                        inertial: Vec::new(),
                    });
                }
            }
        }
    }

    pub fn add_xml_location(
        &mut self,
        checker_id: &str,
        issue_id: u32,
        xpath: &str,
        description: &str,
    ) {
        if let Some(c) = self
            .bundle
            .checkers
            .iter_mut()
            .find(|c| c.checker_id == checker_id)
        {
            if let Some(issue) = c.issues.iter_mut().find(|i| i.issue_id == issue_id) {
                let loc: Option<()> = if let Some(last) = issue.locations.last_mut() {
                    if last.file.is_empty() && last.xml.is_empty() && last.inertial.is_empty() {
                        last.description = description.to_string();
                        last.xml.push(XmlLocation {
                            xpath: xpath.to_string(),
                        });
                        return;
                    }
                    None
                } else {
                    None
                };
                if loc.is_none() {
                    issue.locations.push(Location {
                        description: description.to_string(),
                        file: Vec::new(),
                        xml: vec![XmlLocation {
                            xpath: xpath.to_string(),
                        }],
                        inertial: Vec::new(),
                    });
                }
            }
        }
    }

    pub fn add_inertial_location(
        &mut self,
        checker_id: &str,
        issue_id: u32,
        x: f64,
        y: f64,
        z: f64,
        description: &str,
    ) {
        if let Some(c) = self
            .bundle
            .checkers
            .iter_mut()
            .find(|c| c.checker_id == checker_id)
        {
            if let Some(issue) = c.issues.iter_mut().find(|i| i.issue_id == issue_id) {
                issue.locations.push(Location {
                    description: description.to_string(),
                    file: Vec::new(),
                    xml: Vec::new(),
                    inertial: vec![InertialLocation { x, y, z }],
                });
            }
        }
    }

    pub fn set_checker_status(&mut self, checker_id: &str, status: StatusType) {
        if let Some(c) = self
            .bundle
            .checkers
            .iter_mut()
            .find(|c| c.checker_id == checker_id)
        {
            c.status = Some(status);
        }
    }

    pub fn get_checker_status(&self, checker_id: &str) -> Option<StatusType> {
        self.bundle
            .checkers
            .iter()
            .find(|c| c.checker_id == checker_id)
            .and_then(|c| c.status)
    }

    pub fn add_checker_summary(&mut self, checker_id: &str, content: &str) {
        if let Some(c) = self
            .bundle
            .checkers
            .iter_mut()
            .find(|c| c.checker_id == checker_id)
        {
            if c.summary.is_empty() {
                c.summary = content.to_string();
            } else {
                c.summary.push(' ');
                c.summary.push_str(content);
            }
        }
    }

    pub fn add_checker_bundle_summary(&mut self, content: &str) {
        if self.bundle.summary.is_empty() {
            self.bundle.summary = content.to_string();
        } else {
            self.bundle.summary.push(' ');
            self.bundle.summary.push_str(content);
        }
    }

    /// True if every checker in `ids` completed without any issue.
    pub fn all_checkers_completed_without_issue(&self, ids: &[&str]) -> bool {
        for id in ids {
            let Some(c) = self.bundle.checkers.iter().find(|c| &c.checker_id == id) else {
                return false;
            };
            let completed = matches!(c.status, Some(StatusType::Completed));
            if !completed || !c.issues.is_empty() {
                return false;
            }
        }
        true
    }

    pub fn has_issue_in_checkers(&self, ids: &[&str]) -> bool {
        self.bundle
            .checkers
            .iter()
            .any(|c| ids.contains(&c.checker_id.as_str()) && !c.issues.is_empty())
    }

    /// Generate summaries for bundle and each checker (mirrors Python generate_summary).
    pub fn generate_summaries(&mut self) {
        let mut n_total = 0;
        let mut n_completed = 0;
        let mut n_skipped = 0;
        let mut n_error = 0;
        let mut n_none = 0;
        for c in &self.bundle.checkers {
            n_total += 1;
            match c.status {
                Some(StatusType::Completed) => n_completed += 1,
                Some(StatusType::Skipped) => n_skipped += 1,
                Some(StatusType::Error) => n_error += 1,
                _ => n_none += 1,
            }
        }
        let bundle_summary = format!(
            "{} checker(s) are executed. {} checker(s) are completed. {} checker(s) are skipped. {} checker(s) have internal error and {} checker(s) do not contain status.",
            n_total, n_completed, n_skipped, n_error, n_none
        );
        if self.bundle.summary.is_empty() {
            self.bundle.summary = bundle_summary;
        } else {
            self.bundle.summary.push(' ');
            self.bundle.summary.push_str(&bundle_summary);
        }

        for c in &mut self.bundle.checkers {
            let n_issues = c.issues.len();
            let summary = format!("{} issue(s) are found.", n_issues);
            if c.summary.is_empty() {
                c.summary = summary;
            } else {
                c.summary.push(' ');
                c.summary.push_str(&summary);
            }
        }
    }

    pub fn total_issue_count(&self) -> usize {
        self.bundle.checkers.iter().map(|c| c.issues.len()).sum()
    }

    pub fn has_errors(&self) -> bool {
        self.bundle
            .checkers
            .iter()
            .flat_map(|c| &c.issues)
            .any(|i| matches!(i.level, IssueSeverity::Error))
    }
}

/// Helper used by checkers to record an issue with a single file location.
pub fn report_file_issue(
    result: &mut Result,
    checker_id: &str,
    rule_uid: &str,
    description: &str,
    row: Option<u32>,
    column: Option<u32>,
) {
    let id = result.register_issue(checker_id, description, IssueSeverity::Error, rule_uid);
    result.add_file_location(checker_id, id, row, column, None, description);
}

/// Helper used by checkers to record an issue with an xml (xpath) location.
pub fn report_xml_issue(
    result: &mut Result,
    checker_id: &str,
    rule_uid: &str,
    description: &str,
    xpath: &str,
) {
    let id = result.register_issue(checker_id, description, IssueSeverity::Error, rule_uid);
    result.add_xml_location(checker_id, id, xpath, description);
}
