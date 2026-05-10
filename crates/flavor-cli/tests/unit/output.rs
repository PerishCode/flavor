use std::path::PathBuf;

use crate::{
    model::{issue, Report, ScanStats, Severity},
    output::text_report,
    rules::NAMING_TOO_MANY_WORDS,
};

#[test]
fn text_groups_guidance() {
    let report = Report::with_scan(
        PathBuf::from("root"),
        ScanStats::default(),
        vec![issue(
            Severity::Deny,
            NAMING_TOO_MANY_WORDS,
            "sample.rs",
            Some(1),
            "long name",
        )],
    );
    let text = text_report(&report);

    assert!(text.contains("guidance:\n- core/naming/too-many-words"));
    assert!(text.contains("bad flavor:"));
    assert!(text.contains("action hint:"));
    assert!(text.contains("scan: matched 0, scanned 0"));
    assert!(text.contains("\nissues:\n"));
}

#[test]
fn shows_scan_summary() {
    let report = Report::with_scan(
        PathBuf::from("root"),
        ScanStats {
            matched_files: 3,
            scanned_files: 1,
            generated_files: 1,
            unsupported_files: 1,
            excluded_entries: 2,
        },
        Vec::new(),
    );
    let text = text_report(&report);

    assert!(text.starts_with("flavor passed: no issues\n"));
    assert!(text.contains("scan: matched 3, scanned 1, generated 1, unsupported 1, excluded 2"));
}
