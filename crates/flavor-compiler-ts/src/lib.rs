pub mod ast;
pub mod facts;
pub mod lexer;
pub mod parser;
pub mod state;
pub mod syntax_kind;
pub mod visit;

use flavor_compiler_core::{Diagnostic, SourceText};

pub use ast::TsSourceFile;
pub use facts::TsFacts;
pub use state::{SourceMode, TsCompilerConfig, TsCompilerState};

#[derive(Debug, Clone)]
pub struct TsCompileOutput {
    pub source_file: TsSourceFile,
    pub facts: TsFacts,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct TsCompiler {
    state: TsCompilerState,
}

impl TsCompiler {
    pub fn new(config: TsCompilerConfig) -> Self {
        Self {
            state: TsCompilerState::new(config),
        }
    }

    pub fn run(&mut self, source: SourceText) -> TsCompileOutput {
        let tokens = lexer::scan(&source, self.state.config());
        let parse_output = parser::parse(source, tokens, self.state.config());
        let source_file = parse_output.source_file;
        let facts = facts::collect(&source_file);
        TsCompileOutput {
            source_file,
            facts,
            diagnostics: parse_output.diagnostics,
        }
    }
}

pub fn run(source: SourceText, config: TsCompilerConfig) -> TsCompileOutput {
    TsCompiler::new(config).run(source)
}
