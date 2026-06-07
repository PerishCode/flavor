use crate::{
    config::RuleSettings,
    model::{issue, Issue},
    rules::{PAYLOAD_MAX_WORDS, PAYLOAD_MIN_OCCURRENCES},
};

const DEFAULT_AFFIX_MIN_OCCURRENCES: usize = 15;
const MAX_EXAMPLES: usize = 5;
const IGNORED_AFFIXES: &[&str] = &[
    "new", "default", "test", "tests", "ok", "run", "runs", "main", "mod",
];

pub(crate) fn check_name(
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
    path: &str,
    line: usize,
    kind: &str,
    name: &str,
) {
    if !rule.enabled {
        return;
    }

    let word_count = count_name_words(name);
    let max_words = rule.usize(PAYLOAD_MAX_WORDS).unwrap_or(4);
    if word_count <= max_words {
        return;
    }

    issues.push(issue(
        rule.severity,
        rule.id,
        path,
        Some(line),
        format!("{kind} `{name}` has {word_count} words; max is {max_words}"),
    ));
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NameFact {
    pub(crate) kind: String,
    pub(crate) name: String,
    pub(crate) line: usize,
}

pub(crate) fn check_affix_pressure(
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
    path: &str,
    names: &[NameFact],
) {
    if !rule.enabled {
        return;
    }
    let min_occurrences = rule
        .usize(PAYLOAD_MIN_OCCURRENCES)
        .unwrap_or(DEFAULT_AFFIX_MIN_OCCURRENCES);
    for bucket in affix_buckets(names) {
        if bucket.names.len() < min_occurrences {
            continue;
        }
        issues.push(issue(
            rule.severity,
            rule.id,
            path,
            Some(bucket.line),
            format!(
                "{} {} names share the {} `{}`; consider whether `{}` wants to move from the name into a module, namespace, type, or factory; if the remaining names do not form a clear family, `{}` may be too broad and should be named more sharply; examples: {}",
                bucket.names.len(),
                bucket.kind,
                bucket.side.label(),
                bucket.affix,
                bucket.affix,
                bucket.affix,
                examples(&bucket.names),
            ),
        ));
    }
}

pub(crate) fn count_name_words(name: &str) -> usize {
    split_name_words(name).len()
}

pub(crate) fn split_name_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let normalized = name
        .strip_prefix("r#")
        .unwrap_or(name)
        .trim_matches('_')
        .trim_matches('$');

    for part in normalized
        .split(['_', '-', '$'])
        .filter(|part| !part.is_empty())
    {
        split_camel_part(part, &mut words);
    }

    words
}

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum AffixSide {
    Prefix,
    Suffix,
}

impl AffixSide {
    fn label(self) -> &'static str {
        match self {
            AffixSide::Prefix => "prefix",
            AffixSide::Suffix => "suffix",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct AffixBucket {
    kind: String,
    side: AffixSide,
    affix: String,
    line: usize,
    names: Vec<String>,
}

fn affix_buckets(names: &[NameFact]) -> Vec<AffixBucket> {
    use std::collections::BTreeMap;

    let mut buckets = BTreeMap::<(String, AffixSide, String), AffixBucket>::new();
    for fact in names {
        let words = split_name_words(&fact.name);
        if words.len() < 2 {
            continue;
        }
        for (side, affix) in [
            (AffixSide::Prefix, words.first().unwrap()),
            (AffixSide::Suffix, words.last().unwrap()),
        ] {
            let affix = affix.to_ascii_lowercase();
            if IGNORED_AFFIXES.contains(&affix.as_str()) {
                continue;
            }
            let key = (fact.kind.clone(), side, affix.clone());
            let bucket = buckets.entry(key).or_insert_with(|| AffixBucket {
                kind: fact.kind.clone(),
                side,
                affix,
                line: fact.line,
                names: Vec::new(),
            });
            bucket.line = bucket.line.min(fact.line);
            bucket.names.push(fact.name.clone());
        }
    }
    buckets.into_values().collect()
}

fn examples(names: &[String]) -> String {
    names
        .iter()
        .take(MAX_EXAMPLES)
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn split_camel_part(part: &str, words: &mut Vec<String>) {
    let chars: Vec<char> = part.chars().collect();
    let mut current = String::new();

    for (index, ch) in chars.iter().enumerate() {
        if should_start_word(&chars, index) && !current.is_empty() {
            words.push(current);
            current = String::new();
        }
        current.push(*ch);
    }

    if !current.is_empty() {
        words.push(current);
    }
}

fn should_start_word(chars: &[char], index: usize) -> bool {
    if index == 0 || !chars[index].is_uppercase() {
        return false;
    }

    let prev = chars[index - 1];
    let next = chars.get(index + 1).copied();

    prev.is_lowercase()
        || prev.is_ascii_digit()
        || (prev.is_uppercase() && next.is_some_and(char::is_lowercase))
}
