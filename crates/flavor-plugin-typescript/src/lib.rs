pub mod ast;
pub mod facts;
pub mod lexer;
pub mod parser;
pub mod product;
pub mod state;
pub mod syntax_kind;
pub mod visit;

use flavor_plugin_core::{Diagnostic, SourceText};

pub use ast::TsSourceFile;
pub use facts::{
    TsDispatchBranchFact, TsFacts, TsImportFact, TsImportSpecifier, TsNameFact, TsNameKind,
    TsxElementFact,
};
pub use state::{SourceMode, TsPluginConfig, TsPluginState};

#[derive(Debug, Clone)]
pub struct TsAnalysisOutput {
    pub source_file: TsSourceFile,
    pub facts: TsFacts,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct TsPluginAnalyzer {
    state: TsPluginState,
}

impl TsPluginAnalyzer {
    pub fn new(config: TsPluginConfig) -> Self {
        Self {
            state: TsPluginState::new(config),
        }
    }

    pub fn run(&mut self, source: SourceText) -> TsAnalysisOutput {
        let tokens = lexer::scan(&source, self.state.config());
        let parse_output = parser::parse(source, tokens, self.state.config());
        let source_file = parse_output.source_file;
        let facts = facts::collect(&source_file, self.state.config());
        TsAnalysisOutput {
            source_file,
            facts,
            diagnostics: parse_output.diagnostics,
        }
    }
}

pub fn run(source: SourceText, config: TsPluginConfig) -> TsAnalysisOutput {
    TsPluginAnalyzer::new(config).run(source)
}
