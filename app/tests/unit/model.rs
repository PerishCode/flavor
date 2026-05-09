use std::path::PathBuf;

use crate::{
    model::{issue, Report, Severity},
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
    let report = Report::new(PathBuf::from("root"), issues);

    assert_eq!(report.guidance.len(), 1);
    assert_eq!(report.guidance[0].rule, NAMING_TOO_MANY_WORDS);
    assert!(report.guidance[0].bad_flavor.contains("scenario"));
    assert!(report.guidance[0].action_hint.contains("namespace"));
}
