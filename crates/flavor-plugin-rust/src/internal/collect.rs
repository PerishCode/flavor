use std::collections::BTreeSet;

use flavor_core::{LineIndex, SourceText, Span, SyntaxNode};
use flavor_grammar::{GrammarNode, GrammarToken, GrammarTree};

use crate::{
    internal::grammar,
    model::{RustFacts, RustMatchArmFact, RustNameFact, RustNameKind, RustTestAttributeFact},
};

type RustNode = GrammarNode;
type RustToken = GrammarToken;

const ATTRIBUTE_TOKENS: &[&str] = &["ATTRIBUTE", "INNER_ATTRIBUTE"];
const NAME_TOKENS: &[&str] = &["IDENTIFIER"];

pub(crate) fn collect(syntax: &SyntaxNode, source: &SourceText) -> RustFacts {
    let tree = GrammarTree::new(syntax.clone(), grammar::schema());
    let root = tree.root();
    let mut collector = Collector {
        line_index: source.line_index(),
        trait_methods: BTreeSet::new(),
        facts: RustFacts::default(),
    };
    collector.collect_trait_methods(root.clone(), false);
    collector.collect_node(root.clone(), ImplContext::None);
    collector.collect_test_attributes(root);
    collector.facts
}

struct Collector {
    line_index: LineIndex,
    trait_methods: BTreeSet<String>,
    facts: RustFacts,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ImplContext {
    None,
    Inherent,
    Trait,
}

impl Collector {
    fn collect_trait_methods(&mut self, node: RustNode, in_trait: bool) {
        let next_in_trait = in_trait || node.is("trait_item");
        if next_in_trait && node.is("function_signature_item") {
            if let Some(name) = node.child_tokens_any(NAME_TOKENS).next() {
                self.trait_methods.insert(name.text().to_string());
            }
        }

        for child in node.children() {
            self.collect_trait_methods(child, next_in_trait);
        }
    }

    fn collect_node(&mut self, node: RustNode, impl_context: ImplContext) {
        let next_impl_context = if node.is("impl_item") {
            if impl_item_is_trait(&node) {
                ImplContext::Trait
            } else {
                ImplContext::Inherent
            }
        } else {
            impl_context
        };

        match node.kind_name() {
            Some("function_item") => self.collect_function_item(&node, next_impl_context),
            Some("function_signature_item") => {
                self.collect_named_token(&node, RustNameKind::Method);
            }
            Some("let_declaration") => {
                if let Some(pattern) = node.child("pattern") {
                    self.collect_pattern_names(pattern, RustNameKind::Binding);
                } else {
                    self.collect_named_token(&node, RustNameKind::Binding);
                }
            }
            Some("parameter") => {
                if let Some(pattern) = node.child("pattern") {
                    self.collect_pattern_names(pattern, RustNameKind::Parameter);
                } else {
                    self.collect_named_token(&node, RustNameKind::Parameter);
                }
            }
            Some("match_arm") => {
                let branch = node.child("block").unwrap_or_else(|| node.clone());
                let span = branch.span();
                self.facts.match_arms.push(RustMatchArmFact {
                    span,
                    line: self.line_for(span),
                    lines: self.line_span(span),
                });
            }
            _ => {}
        }

        for child in node.children() {
            self.collect_node(child, next_impl_context);
        }
    }

    fn collect_function_item(&mut self, node: &RustNode, impl_context: ImplContext) {
        let Some(name_token) = node.child_tokens_any(NAME_TOKENS).next() else {
            return;
        };
        let name = name_token.text().to_string();
        if impl_context == ImplContext::Trait && self.trait_methods.contains(&name) {
            return;
        }
        self.push_name(
            match impl_context {
                ImplContext::None => RustNameKind::Function,
                ImplContext::Inherent | ImplContext::Trait => RustNameKind::Method,
            },
            &name_token,
        );
    }

    fn collect_named_token(&mut self, node: &RustNode, kind: RustNameKind) {
        let Some(name) = node.child_tokens_any(NAME_TOKENS).next() else {
            return;
        };
        self.push_name(kind, &name);
    }

    fn collect_pattern_names(&mut self, node: RustNode, kind: RustNameKind) {
        for name in node.tokens_any(NAME_TOKENS) {
            if name.text() != "self" {
                self.push_name(kind, &name);
            }
        }
    }

    fn collect_test_attributes(&mut self, root: RustNode) {
        for token in root.tokens_any(ATTRIBUTE_TOKENS) {
            if is_test_attribute(&token) {
                let span = token.span();
                self.facts.test_attributes.push(RustTestAttributeFact {
                    span,
                    line: self.line_for(span),
                });
            }
        }
    }

    fn push_name(&mut self, kind: RustNameKind, token: &RustToken) {
        let span = token.span();
        self.facts.names.push(RustNameFact {
            kind,
            name: token.text().to_string(),
            span,
            line: self.line_for(span),
        });
    }

    fn line_for(&self, span: Span) -> usize {
        self.line_index.line(span.start)
    }

    fn line_span(&self, span: Span) -> usize {
        self.line_index
            .line(span.end)
            .saturating_sub(self.line_index.line(span.start))
            + 1
    }
}

fn impl_item_is_trait(node: &RustNode) -> bool {
    node.has_token("KEYWORD_FOR")
}

fn is_test_attribute(token: &RustToken) -> bool {
    let trimmed = token.text().trim();
    trimmed.starts_with("#[test")
        || (trimmed.starts_with("#[cfg") && attr_words(trimmed).any(|word| word == "test"))
}

fn attr_words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|word| !word.is_empty())
}
