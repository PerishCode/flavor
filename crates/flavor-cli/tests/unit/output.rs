use std::path::PathBuf;

use crate::{
    model::{issue, Report, ScanStats, Severity},
    output::{text_report, text_rules},
    rules::{self, FS_TOO_MANY_CHILDREN, NAMING_TOO_MANY_WORDS},
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

#[test]
fn text_rules_lists_every_registered_rule_with_target_severity_and_payload() {
    let descriptors: Vec<_> = rules::known_rule_ids()
        .into_iter()
        .filter_map(rules::descriptor)
        .collect();
    let text = text_rules(&descriptors);

    // Every registered rule shows up by id and carries its bad-flavor + action-hint lines.
    for id in rules::known_rule_ids() {
        assert!(text.contains(id), "rules listing should mention {id}");
    }
    assert!(text.contains("bad flavor:"));
    assert!(text.contains("action hint:"));

    // A spot-check on shape: naming rule is a file-target deny with a max_words payload,
    // fs/too-many-children is a dir-target deny with a max_children payload.
    assert!(text.contains(&format!(
        "{NAMING_TOO_MANY_WORDS} (file, deny, max_words=4)"
    )));
    assert!(text.contains(&format!(
        "{FS_TOO_MANY_CHILDREN} (dir, deny, max_children=10)"
    )));
}
