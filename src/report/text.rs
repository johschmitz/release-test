// Ported from qc-opendrive (ASAM e.V., MPL-2.0). See LICENSE.
// Text report formatter, ported from qc_framework text_formatter.py::_dump.

use std::io::Write;

use crate::result::Result;

pub fn write_text_report<W: Write>(result: &Result, w: &mut W) -> std::io::Result<()> {
    let columns = 100;
    let line_sep1 = "=".repeat(columns);
    let line_sep2 = "-".repeat(columns);

    let p = |w: &mut W, val: &str, indent: usize| -> std::io::Result<()> {
        if indent > 0 {
            write!(w, "{}", " ".repeat(indent))?;
        }
        writeln!(w, "{}", val)
    };

    let mut addressed_rules: Vec<String> = Vec::new();
    let mut violated: [Vec<String>; 3] = [Vec::new(), Vec::new(), Vec::new()];

    p(w, &format!("ASAM QUALITY FRAMEWORK v{} - RESULT REPORT", result.bundle.version), 0)?;
    p(w, &line_sep1, 0)?;
    p(w, "", 0)?;

    p(w, "BUNDLES", 0)?;
    p(w, &line_sep2, 0)?;
    p(w, "", 0)?;

    let bundle = &result.bundle;
    p(w, &format!(" -> {} @ {}", bundle.name, bundle.version), 0)?;
    p(w, &format!("Description: {}", bundle.description), 4)?;
    p(w, &format!("Summary: {}", bundle.summary), 4)?;
    if !bundle.params.is_empty() {
        p(w, "Parameters: ", 4)?;
        for param in &bundle.params {
            p(w, &format!(" -> {}: {} [String]", param.name, param.value), 4)?;
        }
    }
    if !bundle.checkers.is_empty() {
        p(w, "Checkers: ", 4)?;
        for checker in &bundle.checkers {
            let status = checker.status.map(|st| st.name()).unwrap_or("NONE");
            p(w, &format!(" -> [{}] {}", status, checker.checker_id), 4)?;
            p(w, &format!("Description: {}", checker.description), 8)?;
            p(w, &format!("Summary: {}", checker.summary), 8)?;
            if !checker.addressed_rules.is_empty() {
                p(w, "Addressed Rules:", 8)?;
                for rule in &checker.addressed_rules {
                    addressed_rules.push(rule.rule_uid.clone());
                    p(w, &format!(" -> {}", rule.rule_uid), 8)?;
                }
            }
            if !checker.issues.is_empty() {
                p(w, "Issues:", 8)?;
                for (i, issue) in checker.issues.iter().enumerate() {
                    let idx = i + 1;
                    let level_name = match issue.level {
                        crate::result::IssueSeverity::Error => "ERROR",
                        crate::result::IssueSeverity::Warning => "WARNING",
                        crate::result::IssueSeverity::Information => "INFORMATION",
                    };
                    match issue.level {
                        crate::result::IssueSeverity::Error => violated[0].push(issue.rule_uid.clone()),
                        crate::result::IssueSeverity::Warning => violated[1].push(issue.rule_uid.clone()),
                        crate::result::IssueSeverity::Information => violated[2].push(issue.rule_uid.clone()),
                    }
                    p(w, &format!(" {:>3}. [{}] @ {}", idx, level_name, issue.rule_uid), 8)?;
                    p(w, &format!("Description: {}", issue.description), 14)?;
                    if !issue.locations.is_empty() {
                        p(w, "Locations:", 14)?;
                        for loc in &issue.locations {
                            for fl in &loc.file {
                                p(w, &format!(" -> File @ col: {}, row: {}, offset: {}: {}", fl.column.map(|c| c as i64).unwrap_or(-1), fl.row.map(|r| r as i64).unwrap_or(-1), fl.offset.map(|o| o as i64).unwrap_or(-1), loc.description), 14)?;
                            }
                            for xl in &loc.xml {
                                p(w, &format!(" -> XML @ `{}`: {}", xl.xpath, loc.description), 14)?;
                            }
                            for il in &loc.inertial {
                                p(w, &format!(" -> Inertial @ [{}; {}; {}]: {}", il.x, il.y, il.z, loc.description), 14)?;
                            }
                        }
                    }
                }
            }
            p(w, "", 8)?;
        }
    }
    p(w, &line_sep1, 0)?;
    p(w, "", 0)?;

    p(w, "ADDRESSED RULES", 0)?;
    p(w, &line_sep2, 0)?;
    p(w, "", 0)?;
    p(w, &format!("Addressed rules: {}", addressed_rules.len()), 0)?;
    for r in &addressed_rules {
        p(w, &format!(" -> {}", r), 0)?;
    }
    p(w, "", 0)?;

    let flagged_total = violated.iter().map(|v| v.len()).sum::<usize>();
    p(w, &format!("Flagged Rules: {}", flagged_total), 0)?;
    let names = ["ERROR", "WARNING", "INFORMATION"];
    for (i, rs) in violated.iter().enumerate() {
        p(w, &format!("with {}: {}", names[i], rs.len()), 2)?;
        for r in rs {
            p(w, &format!(" -> {}", r), 2)?;
        }
    }

    p(w, &line_sep2, 0)?;
    p(w, "", 0)?;
    p(w, "NOTES", 0)?;
    p(w, &line_sep2, 0)?;
    p(w, "", 0)?;
    p(w, "Rule UID format:", 0)?;
    p(w, "  <emanating-entity>:<standard>:x.y.z:rule_set.for_rules.rule_name", 0)?;
    p(w, "", 0)?;
    p(w, "Known limitations: none (xsd-schema resolves XSD 1.1 conditional type assignment before the abstract-type check).", 0)?;

    Ok(())
}
