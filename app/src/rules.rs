use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::model::Severity;

pub(crate) const NAMING_TOO_MANY_WORDS: &str = "core/naming/too-many-words";
pub(crate) const FS_TOO_MANY_CHILDREN: &str = "core/fs/too-many-children";
pub(crate) const SOURCE_TOO_LONG: &str = "core/source/too-long";
pub(crate) const SOURCE_TOO_DEEP: &str = "core/source/too-deep";
pub(crate) const RUST_TESTS_IN_SOURCE: &str = "rust/tests/in-source";
pub(crate) const RUST_PARSE_ERROR: &str = "rust/parse/error";
pub(crate) const TS_PARSE_ERROR: &str = "ts/parse/error";

pub(crate) const PAYLOAD_MAX_WORDS: &str = "max_words";
pub(crate) const PAYLOAD_MAX_CHILDREN: &str = "max_children";
pub(crate) const PAYLOAD_MAX_LINES: &str = "max_lines";
pub(crate) const PAYLOAD_MAX_DEPTH: &str = "max_depth";

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RuleTarget {
    File,
    Dir,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RuleDescriptor {
    pub(crate) id: &'static str,
    pub(crate) target: RuleTarget,
    pub(crate) default_severity: Severity,
    pub(crate) default_payload: BTreeMap<&'static str, Value>,
    pub(crate) bad_flavor: &'static str,
    pub(crate) action_hint: &'static str,
}

pub(crate) fn descriptor(rule_id: &str) -> Option<RuleDescriptor> {
    match rule_id {
        NAMING_TOO_MANY_WORDS => Some(RuleDescriptor {
            id: NAMING_TOO_MANY_WORDS,
            target: RuleTarget::File,
            default_severity: Severity::Deny,
            default_payload: payload([(PAYLOAD_MAX_WORDS, 4)]),
            bad_flavor: "Names may be carrying scenario, path, or assertion context that belongs near an owner boundary.",
            action_hint: "Consider lifting repeated context into a namespace, object, class, module, impl block, or test module before shortening names.",
        }),
        FS_TOO_MANY_CHILDREN => Some(RuleDescriptor {
            id: FS_TOO_MANY_CHILDREN,
            target: RuleTarget::Dir,
            default_severity: Severity::Deny,
            default_payload: payload([(PAYLOAD_MAX_CHILDREN, 10)]),
            bad_flavor: "The directory may be acting as a mixed ownership shelf instead of a clear boundary.",
            action_hint: "Look for real owner or runtime-boundary groups before adding utility buckets or thin routing folders.",
        }),
        SOURCE_TOO_LONG => Some(RuleDescriptor {
            id: SOURCE_TOO_LONG,
            target: RuleTarget::File,
            default_severity: Severity::Deny,
            default_payload: payload([(PAYLOAD_MAX_LINES, 500)]),
            bad_flavor: "The file may be carrying multiple concepts, fixture weight, flow stages, or view/model pressure.",
            action_hint: "Look for concept, fixture, flow, model, or view boundaries; avoid mechanical line-count cuts.",
        }),
        SOURCE_TOO_DEEP => Some(RuleDescriptor {
            id: SOURCE_TOO_DEEP,
            target: RuleTarget::Dir,
            default_severity: Severity::Warning,
            default_payload: payload([(PAYLOAD_MAX_DEPTH, 4)]),
            bad_flavor: "Path depth may be explaining ownership that belongs at module or package level.",
            action_hint: "Use this as boundary-review pressure; module/package changes should wait until ownership is stable.",
        }),
        RUST_TESTS_IN_SOURCE => Some(RuleDescriptor {
            id: RUST_TESTS_IN_SOURCE,
            target: RuleTarget::File,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "Production source may be carrying test-only modules, fixtures, or private-shape pressure.",
            action_hint: "Consider moving test cases into sibling tests paths and exposing only intentional test seams.",
        }),
        RUST_PARSE_ERROR => Some(RuleDescriptor {
            id: RUST_PARSE_ERROR,
            target: RuleTarget::File,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "The Rust source could not be parsed, so AST checks cannot be trusted.",
            action_hint: "Check syntax or parser coverage before treating downstream style results as complete.",
        }),
        TS_PARSE_ERROR => Some(RuleDescriptor {
            id: TS_PARSE_ERROR,
            target: RuleTarget::File,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "The TypeScript or Vue script source could not be parsed, so AST checks cannot be trusted.",
            action_hint: "Check syntax or parser coverage before treating downstream style results as complete.",
        }),
        _ => None,
    }
}

pub(crate) fn known_rule_ids() -> Vec<&'static str> {
    vec![
        NAMING_TOO_MANY_WORDS,
        FS_TOO_MANY_CHILDREN,
        SOURCE_TOO_LONG,
        SOURCE_TOO_DEEP,
        RUST_TESTS_IN_SOURCE,
        RUST_PARSE_ERROR,
        TS_PARSE_ERROR,
    ]
}

fn payload<const N: usize>(values: [(&'static str, usize); N]) -> BTreeMap<&'static str, Value> {
    values
        .into_iter()
        .map(|(key, value)| (key, Value::from(value)))
        .collect()
}
