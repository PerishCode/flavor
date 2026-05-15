use std::fmt::Write as _;

use crate::{
    cli::OutputFormat,
    model::{Report, Severity},
    rules::{self, RuleDescriptor, RuleTarget},
};

pub(crate) fn print_rules(format: OutputFormat) -> Result<(), String> {
    let descriptors: Vec<RuleDescriptor> = rules::known_rule_ids()
        .into_iter()
        .filter_map(rules::descriptor)
        .collect();

    match format {
        OutputFormat::Text => {
            print!("{}", text_rules(&descriptors));
            Ok(())
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&descriptors)
                .map_err(|error| format!("failed to serialize rule descriptors: {error}"))?;
            println!("{json}");
            Ok(())
        }
    }
}

pub(crate) fn text_rules(descriptors: &[RuleDescriptor]) -> String {
    let mut text = String::new();
    for (index, descriptor) in descriptors.iter().enumerate() {
        if index > 0 {
            text.push('\n');
        }
        let target = match descriptor.target {
            RuleTarget::File => "file",
            RuleTarget::Dir => "dir",
        };
        let severity = severity_label(descriptor.default_severity);
        let enabled = if descriptor.default_enabled {
            ""
        } else {
            ", off-by-default"
        };
        let payload = if descriptor.default_payload.is_empty() {
            String::new()
        } else {
            let pairs = descriptor
                .default_payload
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>();
            format!(", {}", pairs.join(", "))
        };
        writeln!(
            &mut text,
            "{} ({target}, {severity}{enabled}{payload})",
            descriptor.id
        )
        .expect("write string");
        writeln!(&mut text, "  bad flavor: {}", descriptor.bad_flavor).expect("write string");
        writeln!(&mut text, "  action hint: {}", descriptor.action_hint).expect("write string");
    }
    text
}

pub(crate) fn print_report(report: &Report, format: OutputFormat) -> Result<(), String> {
    match format {
        OutputFormat::Text => {
            print_text_report(report);
            Ok(())
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(report)
                .map_err(|error| format!("failed to serialize guard report: {error}"))?;
            println!("{json}");
            Ok(())
        }
    }
}

fn print_text_report(report: &Report) {
    print!("{}", text_report(report));
}

pub(crate) fn text_report(report: &Report) -> String {
    let deny_count = report.deny_count();
    let warning_count = report.warning_count();

    if report.issues.is_empty() {
        let mut text = "flavor passed: no issues\n".to_string();
        push_scan_summary(&mut text, report);
        return text;
    }

    let mut text =
        format!("flavor found {deny_count} deny issue(s) and {warning_count} warning(s)\n");
    push_scan_summary(&mut text, report);

    if !report.guidance.is_empty() {
        text.push_str("\nguidance:\n");
        for guide in &report.guidance {
            writeln!(&mut text, "- {}", guide.rule).expect("write string");
            writeln!(&mut text, "  bad flavor: {}", guide.bad_flavor).expect("write string");
            writeln!(&mut text, "  action hint: {}", guide.action_hint).expect("write string");
        }
    }

    text.push_str("\nissues:\n");
    for issue in &report.issues {
        let location = match issue.line {
            Some(line) => format!("{}:{line}", issue.path),
            None => issue.path.clone(),
        };
        writeln!(
            &mut text,
            "{} {} {location} - {}",
            severity_label(issue.severity),
            issue.rule,
            issue.message
        )
        .expect("write string");
    }

    text
}

fn push_scan_summary(text: &mut String, report: &Report) {
    writeln!(
        text,
        "scan: matched {}, scanned {}, generated {}, unsupported {}, excluded {}",
        report.scan.matched_files,
        report.scan.scanned_files,
        report.scan.generated_files,
        report.scan.unsupported_files,
        report.scan.excluded_entries,
    )
    .expect("write string");
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Deny => "deny",
        Severity::Warning => "warning",
    }
}
