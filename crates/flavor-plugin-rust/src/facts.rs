use std::collections::BTreeSet;

use flavor_plugin_core::Span;
use tree_sitter::Node;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct RustFacts {
    pub names: Vec<RustNameFact>,
    pub match_arms: Vec<RustMatchArmFact>,
    pub test_attributes: Vec<RustTestAttributeFact>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RustNameKind {
    Function,
    Method,
    Binding,
    Parameter,
}

impl RustNameKind {
    pub fn label(self) -> &'static str {
        match self {
            RustNameKind::Function => "function",
            RustNameKind::Method => "method",
            RustNameKind::Binding => "binding",
            RustNameKind::Parameter => "parameter",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RustNameFact {
    pub kind: RustNameKind,
    pub name: String,
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RustMatchArmFact {
    pub span: Span,
    pub line: usize,
    pub lines: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RustTestAttributeFact {
    pub span: Span,
    pub line: usize,
}

pub fn collect(root: Node<'_>, source: &[u8]) -> RustFacts {
    let mut collector = Collector {
        source,
        trait_methods: BTreeSet::new(),
        facts: RustFacts::default(),
    };
    collector.collect_trait_methods(root, false);
    collector.collect_node(root, ImplContext::None);
    collector.facts
}

struct Collector<'a> {
    source: &'a [u8],
    trait_methods: BTreeSet<String>,
    facts: RustFacts,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ImplContext {
    None,
    Inherent,
    Trait,
}

impl Collector<'_> {
    fn collect_trait_methods(&mut self, node: Node<'_>, in_trait: bool) {
        let next_in_trait = in_trait || node.kind() == "trait_item";
        if next_in_trait && node.kind() == "function_signature_item" {
            if let Some(name) = node
                .child_by_field_name("name")
                .and_then(|name| node_text(name, self.source))
            {
                self.trait_methods.insert(name);
            }
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.collect_trait_methods(child, next_in_trait);
        }
    }

    fn collect_node(&mut self, node: Node<'_>, impl_context: ImplContext) {
        let next_impl_context = if node.kind() == "impl_item" {
            if node.child_by_field_name("trait").is_some() {
                ImplContext::Trait
            } else {
                ImplContext::Inherent
            }
        } else {
            impl_context
        };

        match node.kind() {
            "function_item" => self.collect_function_item(node, next_impl_context),
            "function_signature_item" => {
                self.collect_named_child(node, "name", RustNameKind::Method);
            }
            "let_declaration" => {
                if let Some(pattern) = node.child_by_field_name("pattern") {
                    self.collect_pattern_names(pattern, RustNameKind::Binding);
                }
            }
            "parameter" => {
                if let Some(pattern) = node.child_by_field_name("pattern") {
                    self.collect_pattern_names(pattern, RustNameKind::Parameter);
                }
            }
            "match_arm" => {
                if let Some(value) = node.child_by_field_name("value") {
                    self.facts.match_arms.push(RustMatchArmFact {
                        span: span_for(value),
                        line: line_for(value),
                        lines: line_span(value),
                    });
                }
            }
            "attribute_item" | "inner_attribute_item" if is_test_attribute(node, self.source) => {
                self.facts.test_attributes.push(RustTestAttributeFact {
                    span: span_for(node),
                    line: line_for(node),
                });
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.collect_node(child, next_impl_context);
        }
    }

    fn collect_function_item(&mut self, node: Node<'_>, impl_context: ImplContext) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let Some(name) = node_text(name_node, self.source) else {
            return;
        };
        if impl_context == ImplContext::Trait && self.trait_methods.contains(&name) {
            return;
        }
        self.facts.names.push(RustNameFact {
            kind: match impl_context {
                ImplContext::None => RustNameKind::Function,
                ImplContext::Inherent | ImplContext::Trait => RustNameKind::Method,
            },
            name,
            span: span_for(name_node),
            line: line_for(name_node),
        });
    }

    fn collect_named_child(&mut self, node: Node<'_>, field: &str, kind: RustNameKind) {
        let Some(name) = node.child_by_field_name(field) else {
            return;
        };
        let Some(text) = node_text(name, self.source) else {
            return;
        };
        self.facts.names.push(RustNameFact {
            kind,
            name: text,
            span: span_for(name),
            line: line_for(name),
        });
    }

    fn collect_pattern_names(&mut self, node: Node<'_>, kind: RustNameKind) {
        match node.kind() {
            "identifier" => {
                let Some(name) = node_text(node, self.source) else {
                    return;
                };
                if name == "self" {
                    return;
                }
                self.facts.names.push(RustNameFact {
                    kind,
                    name,
                    span: span_for(node),
                    line: line_for(node),
                });
            }
            "field_pattern" => {
                if let Some(pattern) = node.child_by_field_name("pattern") {
                    self.collect_pattern_names(pattern, kind);
                }
            }
            "mut_pattern" | "ref_pattern" | "reference_pattern" => {
                if let Some(pattern) = first_named_child(node) {
                    self.collect_pattern_names(pattern, kind);
                }
            }
            "tuple_pattern" | "tuple_struct_pattern" | "slice_pattern" | "struct_pattern" => {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    self.collect_pattern_names(child, kind);
                }
            }
            _ => {}
        }
    }
}

fn is_test_attribute(node: Node<'_>, source: &[u8]) -> bool {
    let Some(text) = node_text(node, source) else {
        return false;
    };
    let trimmed = text.trim();
    trimmed.starts_with("#[test")
        || (trimmed.starts_with("#[cfg") && attr_words(trimmed).any(|word| word == "test"))
}

fn attr_words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|word| !word.is_empty())
}

fn first_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    let child = node.named_children(&mut cursor).next();
    child
}

fn node_text(node: Node<'_>, source: &[u8]) -> Option<String> {
    node.utf8_text(source).ok().map(str::to_string)
}

fn line_for(node: Node<'_>) -> usize {
    node.start_position().row + 1
}

fn line_span(node: Node<'_>) -> usize {
    node.end_position()
        .row
        .saturating_sub(node.start_position().row)
        + 1
}

fn span_for(node: Node<'_>) -> Span {
    Span::from_usize(node.start_byte(), node.end_byte())
}
