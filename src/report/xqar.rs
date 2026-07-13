// XQAR report serializer, conforming to xqar_result_format.xsd.

use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use quick_xml::Result;

use crate::result::{IssueSeverity, Result as QcResult};

pub fn write_xqar<W: Write>(result: &QcResult, w: &mut W) -> Result<()> {
    let mut writer = Writer::new(w);

    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), Some("no"))))?;

    // <CheckerResults version="...">
    let mut root = BytesStart::new("CheckerResults");
    root.push_attribute(("version", result.bundle.version.as_str()));
    writer.write_event(Event::Start(root))?;

    // <CheckerBundle ...>
    let mut bundle = BytesStart::new("CheckerBundle");
    bundle.push_attribute(("name", result.bundle.name.as_str()));
    bundle.push_attribute(("version", result.bundle.version.as_str()));
    bundle.push_attribute(("description", result.bundle.description.as_str()));
    bundle.push_attribute(("summary", result.bundle.summary.as_str()));
    writer.write_event(Event::Start(bundle))?;

    for checker in &result.bundle.checkers {
        write_checker(&mut writer, checker)?;
    }

    writer.write_event(Event::End(BytesEnd::new("CheckerBundle")))?;
    writer.write_event(Event::End(BytesEnd::new("CheckerResults")))?;
    Ok(())
}

fn write_checker(
    writer: &mut Writer<impl Write>,
    checker: &crate::result::Checker,
) -> Result<()> {
    let mut cs = BytesStart::new("Checker");
    cs.push_attribute(("checkerId", checker.checker_id.as_str()));
    cs.push_attribute(("description", checker.description.as_str()));
    cs.push_attribute(("summary", checker.summary.as_str()));
    let status = checker
        .status
        .map(|s| s.as_str())
        .unwrap_or("none");
    cs.push_attribute(("status", status));
    writer.write_event(Event::Start(cs))?;

    for param in &checker.params {
        let mut ps = BytesStart::new("Param");
        ps.push_attribute(("name", param.name.as_str()));
        ps.push_attribute(("value", param.value.as_str()));
        writer.write_event(Event::Empty(ps))?;
    }

    for issue in &checker.issues {
        write_issue(writer, issue)?;
    }

    for rule in &checker.addressed_rules {
        let mut rs = BytesStart::new("AddressedRule");
        rs.push_attribute(("ruleUID", rule.rule_uid.as_str()));
        writer.write_event(Event::Empty(rs))?;
    }

    writer.write_event(Event::End(BytesEnd::new("Checker")))?;
    Ok(())
}

fn write_issue(
    writer: &mut Writer<impl Write>,
    issue: &crate::result::Issue,
) -> Result<()> {
    let mut is = BytesStart::new("Issue");
    is.push_attribute(("issueId", issue.issue_id.to_string().as_str()));
    is.push_attribute(("description", issue.description.as_str()));
    let level = match issue.level {
        IssueSeverity::Error => "error",
        IssueSeverity::Warning => "warning",
        IssueSeverity::Information => "information",
    };
    is.push_attribute(("level", level));
    is.push_attribute(("ruleUID", issue.rule_uid.as_str()));
    writer.write_event(Event::Start(is))?;

    for loc in &issue.locations {
        let mut ls = BytesStart::new("Locations");
        if !loc.description.is_empty() {
            ls.push_attribute(("description", loc.description.as_str()));
        }
        writer.write_event(Event::Start(ls))?;

        for fl in &loc.file {
            let mut f = BytesStart::new("FileLocation");
            if let Some(row) = fl.row {
                f.push_attribute(("row", row.to_string().as_str()));
            }
            if let Some(col) = fl.column {
                f.push_attribute(("column", col.to_string().as_str()));
            }
            if let Some(off) = fl.offset {
                f.push_attribute(("offset", off.to_string().as_str()));
            }
            writer.write_event(Event::Empty(f))?;
        }
        for xl in &loc.xml {
            let mut x = BytesStart::new("XMLLocation");
            x.push_attribute(("xpath", xl.xpath.as_str()));
            writer.write_event(Event::Empty(x))?;
        }
        for il in &loc.inertial {
            let mut i = BytesStart::new("InertialLocation");
            i.push_attribute(("x", il.x.to_string().as_str()));
            i.push_attribute(("y", il.y.to_string().as_str()));
            i.push_attribute(("z", il.z.to_string().as_str()));
            writer.write_event(Event::Empty(i))?;
        }

        writer.write_event(Event::End(BytesEnd::new("Locations")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("Issue")))?;
    Ok(())
}
