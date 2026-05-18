use std::collections::BTreeSet;

#[derive(Debug, Default)]
pub(crate) struct IssueAggregation {
    seen: BTreeSet<String>,
}

impl IssueAggregation {
    pub(crate) fn accepts(&mut self, key: impl Into<String>) -> bool {
        self.seen.insert(key.into())
    }
}

pub(crate) fn key(lang: &str, pattern: &str, parts: &[&str]) -> String {
    let mut key = format!("{lang}:{pattern}");
    for part in parts {
        key.push(':');
        key.push_str(part);
    }
    key
}

pub(crate) fn line_key(lang: &str, pattern: &str, path: &str, line: Option<usize>) -> String {
    let line = line_label(line);
    key(lang, pattern, &[path, line.as_str()])
}

fn line_label(line: Option<usize>) -> String {
    line.map(|line| line.to_string())
        .unwrap_or_else(|| "none".to_string())
}
