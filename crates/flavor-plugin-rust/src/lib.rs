mod internal;
pub mod model;
pub mod plugin;
pub mod state;

use flavor_core::SourceText;

pub use model::{
    RustAnalysisOutput, RustFacts, RustMatchArmFact, RustNameFact, RustNameKind,
    RustRepeatedTokenPatternFact, RustTestAttributeFact,
};
pub use state::{RustPluginConfig, RustPluginState, RustRepeatedTokenPatternConfig};

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
        let config = self.state.config();
        let frontend = internal::frontend::parse(source);
        let mut facts = internal::collect::collect(&frontend.syntax, &frontend.source);
        facts.repeated_token_patterns = internal::repeated_pattern::collect(
            &frontend.syntax,
            &frontend.source,
            &config.repeated_token_patterns,
        );
        RustAnalysisOutput {
            source: frontend.source,
            syntax: frontend.syntax,
            facts,
            diagnostics: frontend.diagnostics,
        }
    }
}

pub fn run(source: SourceText, config: RustPluginConfig) -> RustAnalysisOutput {
    RustPluginAnalyzer::new(config).run(source)
}
