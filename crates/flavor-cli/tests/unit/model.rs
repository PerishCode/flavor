use std::path::PathBuf;

use crate::{
    model::{issue, Report, ScanStats, Severity},
    rules::NAMING_TOO_MANY_WORDS,
};

#[test]
fn report_guides_rules() {
    let issues = vec![
        issue(
            Severity::Deny,
            NAMING_TOO_MANY_WORDS,
            "sample.rs",
            Some(1),
            "long name",
        ),
        issue(
            Severity::Deny,
            NAMING_TOO_MANY_WORDS,
            "sample.rs",
            Some(2),
            "another long name",
        ),
    ];
    let report = Report::with_scan(PathBuf::from("root"), ScanStats::default(), issues);

    assert_eq!(report.guidance.len(), 1);
    assert_eq!(report.guidance[0].rule, NAMING_TOO_MANY_WORDS);
    assert!(report.guidance[0].bad_flavor.contains("scenario"));
    assert!(report.guidance[0].action_hint.contains("namespace"));
}

#[test]
fn exit_code_signals_empty_scan_even_without_issues() {
    let report = Report::with_scan(PathBuf::from("root"), ScanStats::default(), Vec::new());

    assert!(report.is_empty_scan());
    assert_eq!(report.exit_code(false), 1);
    assert_eq!(report.exit_code(true), 1);
}

#[test]
fn exit_code_zero_when_scan_matched_and_no_issues() {
    let stats = ScanStats {
        matched_files: 5,
        scanned_files: 5,
        ..ScanStats::default()
    };
    let report = Report::with_scan(PathBuf::from("root"), stats, Vec::new());

    assert!(!report.is_empty_scan());
    assert_eq!(report.exit_code(false), 0);
    assert_eq!(report.exit_code(true), 0);
}

#[test]
fn exit_code_one_for_deny_regardless_of_strict_flag() {
    let stats = ScanStats {
        matched_files: 5,
        scanned_files: 5,
        ..ScanStats::default()
    };
    let issues = vec![issue(
        Severity::Deny,
        NAMING_TOO_MANY_WORDS,
        "sample.rs",
        Some(1),
        "long name",
    )];
    let report = Report::with_scan(PathBuf::from("root"), stats, issues);

    assert_eq!(report.exit_code(false), 1);
}

#[test]
fn exit_code_respects_strict_warnings() {
    let stats = ScanStats {
        matched_files: 5,
        scanned_files: 5,
        ..ScanStats::default()
    };
    let issues = vec![issue(
        Severity::Warning,
        NAMING_TOO_MANY_WORDS,
        "sample.rs",
        Some(1),
        "long name",
    )];
    let report = Report::with_scan(PathBuf::from("root"), stats, issues);

    assert_eq!(report.exit_code(false), 0);
    assert_eq!(report.exit_code(true), 1);
}
