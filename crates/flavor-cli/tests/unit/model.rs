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
fn empty_scan_exits_one() {
    let report = Report::with_scan(PathBuf::from("root"), ScanStats::default(), Vec::new());

    assert!(report.is_empty_scan());
    assert_eq!(report.exit_code(false), 1);
    assert_eq!(report.exit_code(true), 1);
}

#[test]
fn clean_scan_exits_zero() {
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
fn deny_always_exits_one() {
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
fn strict_warnings_exit_one() {
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

#[test]
fn allow_empty_exit_zero() {
    let report =
        Report::with_scan_allow_empty(PathBuf::from("root"), ScanStats::default(), Vec::new());

    // With allowEmptyScan: true, a 0-match report is no longer considered
    // empty — exit 0 regardless of strict_warnings.
    assert!(!report.is_empty_scan());
    assert_eq!(report.exit_code(false), 0);
    assert_eq!(report.exit_code(true), 0);
}

#[test]
fn allow_empty_still_denies() {
    let issues = vec![issue(
        Severity::Deny,
        NAMING_TOO_MANY_WORDS,
        "sample.rs",
        Some(1),
        "long name",
    )];
    let report = Report::with_scan_allow_empty(PathBuf::from("root"), ScanStats::default(), issues);

    // allowEmptyScan only quiets the 0-match path; actual deny issues still
    // exit 1 (otherwise the opt-out would silence real failures too).
    assert_eq!(report.exit_code(false), 1);
}
