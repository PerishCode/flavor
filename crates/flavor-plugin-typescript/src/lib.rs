mod ast;
mod facts;
mod internal;
mod lexer;
pub mod model;
mod parser;
pub mod plugin;
pub mod state;
mod visit;

use flavor_core::SourceText;

pub use model::{
    TsAnalysisOutput, TsDispatchBranchFact, TsFacts, TsImportFact, TsImportSpecifier, TsNameFact,
    TsNameKind, TsTokenKind, TsxElementFact,
};
pub use state::{SourceMode, TsPluginConfig, TsPluginState};

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
        let facts = facts::collect(&parse_output.source_file);
        let (source, tokens, syntax) = parse_output.source_file.into_parts();
        TsAnalysisOutput {
            source,
            syntax,
            tokens,
            facts,
            diagnostics: parse_output.diagnostics,
        }
    }
}

pub fn run(source: SourceText, config: TsPluginConfig) -> TsAnalysisOutput {
    TsPluginAnalyzer::new(config).run(source)
}
