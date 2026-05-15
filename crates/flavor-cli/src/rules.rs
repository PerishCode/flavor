use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::model::Severity;

pub(crate) const NAMING_TOO_MANY_WORDS: &str = "core/naming/too-many-words";
pub(crate) const DISPATCH_BRANCH_TOO_LONG: &str = "core/dispatch/branch-too-long";
pub(crate) const FS_CHILDREN_SHAPE: &str = "core/fs/children-shape";
pub(crate) const FS_FORBIDDEN_EXTENSION: &str = "core/fs/forbidden-extension";
pub(crate) const FS_NAME_SHAPE: &str = "core/fs/name-shape";
pub(crate) const FS_TOO_MANY_CHILDREN: &str = "core/fs/too-many-children";
pub(crate) const SOURCE_TOO_LONG: &str = "core/source/too-long";
pub(crate) const SOURCE_TOO_DEEP: &str = "core/source/too-deep";
pub(crate) const RUST_TESTS_IN_SOURCE: &str = "rust/tests/in-source";
pub(crate) const RUST_PARSE_ERROR: &str = "rust/parse/error";
pub(crate) const SVELTE_COMPONENT_TOO_LONG: &str = "svelte/component/too-long";
pub(crate) const SVELTE_PARSE_ERROR: &str = "svelte/parse/error";
pub(crate) const SVELTE_SCRIPT_TOO_LONG: &str = "svelte/script/too-long";
pub(crate) const SVELTE_STYLE_TOO_LONG: &str = "svelte/style/too-long";
pub(crate) const SVELTE_TEMPLATE_TOO_COMPLEX: &str = "svelte/template/too-complex";
pub(crate) const TS_PARSE_ERROR: &str = "ts/parse/error";
pub(crate) const TSX_REQUIRES_PRIMITIVE: &str = "tsx/component/requires-primitive-composition";
pub(crate) const TSX_NO_INTRINSICS: &str = "tsx/jsx/no-intrinsic-elements";
pub(crate) const VUE_PARSE_ERROR: &str = "vue/parse/error";

pub(crate) const PAYLOAD_ALLOWED: &str = "allowed";
pub(crate) const PAYLOAD_ALLOWED_INTRINSICS: &str = "allowed_intrinsics";
pub(crate) const PAYLOAD_CASE: &str = "case";
pub(crate) const PAYLOAD_EXTENSIONS: &str = "extensions";
pub(crate) const PAYLOAD_FORBIDDEN: &str = "forbidden";
pub(crate) const PAYLOAD_MAX_WORDS: &str = "max_words";
pub(crate) const PAYLOAD_MAX_BRANCH_LINES: &str = "max_branch_lines";
pub(crate) const PAYLOAD_MAX_BLOCKS: &str = "max_blocks";
pub(crate) const PAYLOAD_MAX_CHILDREN: &str = "max_children";
pub(crate) const PAYLOAD_MAX_LINES: &str = "max_lines";
pub(crate) const PAYLOAD_MAX_DEPTH: &str = "max_depth";
pub(crate) const PAYLOAD_PRIMITIVE_SOURCES: &str = "primitive_sources";
pub(crate) const PAYLOAD_REQUIRED: &str = "required";

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RuleTarget {
    File,
    Dir,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuleDescriptor {
    pub(crate) id: &'static str,
    pub(crate) target: RuleTarget,
    pub(crate) default_enabled: bool,
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
            default_enabled: true,
            default_severity: Severity::Deny,
            default_payload: payload([(PAYLOAD_MAX_WORDS, 4)]),
            bad_flavor: "Names may be carrying scenario, path, or assertion context that belongs near an owner boundary.",
            action_hint: "Consider lifting repeated context into a namespace, object, class, module, impl block, or test module before shortening names.",
        }),
        DISPATCH_BRANCH_TOO_LONG => Some(RuleDescriptor {
            id: DISPATCH_BRANCH_TOO_LONG,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Warning,
            default_payload: payload([(PAYLOAD_MAX_BRANCH_LINES, 24)]),
            bad_flavor: "Dispatch may be carrying implementation bodies instead of routing quickly to named behavior.",
            action_hint: "Keep switch/match arms as handoff points; extract the branch body into a local concept when the branch grows a second-stage flow.",
        }),
        FS_CHILDREN_SHAPE => Some(RuleDescriptor {
            id: FS_CHILDREN_SHAPE,
            target: RuleTarget::Dir,
            default_enabled: false,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "A directory boundary may have drifted from its declared direct-child shape.",
            action_hint: "Keep the expected root skeleton explicit; move extra context into an existing owner directory or revise the boundary contract.",
        }),
        FS_FORBIDDEN_EXTENSION => Some(RuleDescriptor {
            id: FS_FORBIDDEN_EXTENSION,
            target: RuleTarget::File,
            default_enabled: false,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "A file type may be escaping a boundary that declared it should not own that artifact kind.",
            action_hint: "Move the artifact to the owning package or replace it with the configured primitive surface for this boundary.",
        }),
        FS_NAME_SHAPE => Some(RuleDescriptor {
            id: FS_NAME_SHAPE,
            target: RuleTarget::File,
            default_enabled: false,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "A file name may be carrying context that should live in a directory boundary or component identity.",
            action_hint: "Use directory ownership for context and keep file names aligned with the declared casing and word-count shape.",
        }),
        FS_TOO_MANY_CHILDREN => Some(RuleDescriptor {
            id: FS_TOO_MANY_CHILDREN,
            target: RuleTarget::Dir,
            default_enabled: true,
            default_severity: Severity::Deny,
            default_payload: payload([(PAYLOAD_MAX_CHILDREN, 10)]),
            bad_flavor: "The directory may be acting as a mixed ownership shelf instead of a clear boundary.",
            action_hint: "Look for real owner or runtime-boundary groups before adding utility buckets or thin routing folders.",
        }),
        SOURCE_TOO_LONG => Some(RuleDescriptor {
            id: SOURCE_TOO_LONG,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Deny,
            default_payload: payload([(PAYLOAD_MAX_LINES, 500)]),
            bad_flavor: "The file may be carrying multiple concepts, fixture weight, flow stages, or view/model pressure.",
            action_hint: "Look for concept, fixture, flow, model, or view boundaries; avoid mechanical line-count cuts.",
        }),
        SOURCE_TOO_DEEP => Some(RuleDescriptor {
            id: SOURCE_TOO_DEEP,
            target: RuleTarget::Dir,
            default_enabled: true,
            default_severity: Severity::Warning,
            default_payload: payload([(PAYLOAD_MAX_DEPTH, 4)]),
            bad_flavor: "Path depth may be explaining ownership that belongs at module or package level.",
            action_hint: "Use this as boundary-review pressure; module/package changes should wait until ownership is stable.",
        }),
        RUST_TESTS_IN_SOURCE => Some(RuleDescriptor {
            id: RUST_TESTS_IN_SOURCE,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "Production source may be carrying test-only modules, fixtures, or private-shape pressure.",
            action_hint: "Consider moving test cases into sibling tests paths and exposing only intentional test seams.",
        }),
        RUST_PARSE_ERROR => Some(RuleDescriptor {
            id: RUST_PARSE_ERROR,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "The Rust source could not be parsed, so AST checks cannot be trusted.",
            action_hint: "Check syntax or parser coverage before treating downstream style results as complete.",
        }),
        SVELTE_COMPONENT_TOO_LONG => Some(RuleDescriptor {
            id: SVELTE_COMPONENT_TOO_LONG,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Warning,
            default_payload: payload([(PAYLOAD_MAX_LINES, 500)]),
            bad_flavor: "A Svelte component may be carrying state, view, and style pressure in one file.",
            action_hint: "Use the script/template/style breakdown to decide whether state, composition, or CSS ownership should split first.",
        }),
        SVELTE_PARSE_ERROR => Some(RuleDescriptor {
            id: SVELTE_PARSE_ERROR,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "The Svelte component structure could not be parsed, so component checks cannot be trusted.",
            action_hint: "Check Svelte block structure or parser coverage before treating downstream style results as complete.",
        }),
        SVELTE_SCRIPT_TOO_LONG => Some(RuleDescriptor {
            id: SVELTE_SCRIPT_TOO_LONG,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Warning,
            default_payload: payload([(PAYLOAD_MAX_LINES, 180)]),
            bad_flavor: "A Svelte script block may be carrying controller or state-machine behavior inside a view file.",
            action_hint: "Look for named state, request, derivation, or command boundaries before extracting generic helpers.",
        }),
        SVELTE_STYLE_TOO_LONG => Some(RuleDescriptor {
            id: SVELTE_STYLE_TOO_LONG,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Warning,
            default_payload: payload([(PAYLOAD_MAX_LINES, 240)]),
            bad_flavor: "Scoped Svelte CSS may be absorbing design-system or layout primitive pressure.",
            action_hint: "Check whether repeated visual rules belong in a component primitive, theme token, or narrower local style boundary.",
        }),
        SVELTE_TEMPLATE_TOO_COMPLEX => Some(RuleDescriptor {
            id: SVELTE_TEMPLATE_TOO_COMPLEX,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Warning,
            default_payload: payload([(PAYLOAD_MAX_BLOCKS, 18)]),
            bad_flavor: "A Svelte template may be carrying too many conditional or iterative view states.",
            action_hint: "Look for product states, repeated list item views, or named render fragments before flattening the component mechanically.",
        }),
        TS_PARSE_ERROR => Some(RuleDescriptor {
            id: TS_PARSE_ERROR,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "The TypeScript or Vue script source could not be parsed, so AST checks cannot be trusted.",
            action_hint: "Check syntax or parser coverage before treating downstream style results as complete.",
        }),
        TSX_REQUIRES_PRIMITIVE => Some(RuleDescriptor {
            id: TSX_REQUIRES_PRIMITIVE,
            target: RuleTarget::File,
            default_enabled: false,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "A renderer component may be minting local UI atoms instead of composing the configured primitive package.",
            action_hint: "Import and compose a primitive from the configured source, or move the new primitive into that package if it is durable.",
        }),
        TSX_NO_INTRINSICS => Some(RuleDescriptor {
            id: TSX_NO_INTRINSICS,
            target: RuleTarget::File,
            default_enabled: false,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "Renderer JSX may be reaching straight for DOM tags instead of product or shared UI primitives.",
            action_hint: "Compose named components and primitives at this layer; keep raw DOM ownership inside lower-level UI packages.",
        }),
        VUE_PARSE_ERROR => Some(RuleDescriptor {
            id: VUE_PARSE_ERROR,
            target: RuleTarget::File,
            default_enabled: true,
            default_severity: Severity::Deny,
            default_payload: BTreeMap::new(),
            bad_flavor: "The Vue SFC structure could not be parsed, so Vue script checks cannot be trusted.",
            action_hint: "Check the SFC block structure before treating downstream script style results as complete.",
        }),
        _ => None,
    }
}

pub(crate) fn known_rule_ids() -> Vec<&'static str> {
    vec![
        NAMING_TOO_MANY_WORDS,
        DISPATCH_BRANCH_TOO_LONG,
        FS_CHILDREN_SHAPE,
        FS_FORBIDDEN_EXTENSION,
        FS_NAME_SHAPE,
        FS_TOO_MANY_CHILDREN,
        SOURCE_TOO_LONG,
        SOURCE_TOO_DEEP,
        RUST_TESTS_IN_SOURCE,
        RUST_PARSE_ERROR,
        SVELTE_COMPONENT_TOO_LONG,
        SVELTE_PARSE_ERROR,
        SVELTE_SCRIPT_TOO_LONG,
        SVELTE_STYLE_TOO_LONG,
        SVELTE_TEMPLATE_TOO_COMPLEX,
        TS_PARSE_ERROR,
        TSX_REQUIRES_PRIMITIVE,
        TSX_NO_INTRINSICS,
        VUE_PARSE_ERROR,
    ]
}

fn payload<const N: usize>(values: [(&'static str, usize); N]) -> BTreeMap<&'static str, Value> {
    values
        .into_iter()
        .map(|(key, value)| (key, Value::from(value)))
        .collect()
}
