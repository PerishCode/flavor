pub mod facts;
pub mod product;
pub mod state;

use flavor_plugin_core::{Diagnostic, SourceText, Span};
use tree_sitter::Parser;

pub use facts::{RustFacts, RustMatchArmFact, RustNameFact, RustNameKind, RustTestAttributeFact};
pub use state::{RustPluginConfig, RustPluginState};

#[derive(Debug, Clone)]
pub struct RustAnalysisOutput {
    pub source: SourceText,
    pub facts: RustFacts,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct RustPluginAnalyzer {
    state: RustPluginState,
}

impl RustPluginAnalyzer {
    pub fn new(config: RustPluginConfig) -> Self {
        Self {
            state: RustPluginState::new(config),
        }
    }

    pub fn run(&mut self, source: SourceText) -> RustAnalysisOutput {
        let _config = self.state.config();
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("tree-sitter Rust grammar must load");
        let Some(tree) = parser.parse(source.as_str(), None) else {
            return RustAnalysisOutput {
                source,
                facts: RustFacts::default(),
                diagnostics: vec![Diagnostic::error_code(
                    None,
                    "rust/parse/error",
                    "failed to parse Rust source",
                )],
            };
        };
        let root = tree.root_node();
        let facts = facts::collect(root, source.as_str().as_bytes());
        let diagnostics = if root.has_error() {
            vec![Diagnostic::error_code(
                first_error_span(root),
                "rust/parse/error",
                "invalid Rust syntax",
            )]
        } else {
            Vec::new()
        };
        RustAnalysisOutput {
            source,
            facts,
            diagnostics,
        }
    }
}

pub fn run(source: SourceText, config: RustPluginConfig) -> RustAnalysisOutput {
    RustPluginAnalyzer::new(config).run(source)
}

fn first_error_span(node: tree_sitter::Node<'_>) -> Option<Span> {
    if node.is_error() || node.is_missing() {
        return Some(Span::from_usize(node.start_byte(), node.end_byte()));
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Some(span) = first_error_span(child) {
            return Some(span);
        }
    }
    None
}
