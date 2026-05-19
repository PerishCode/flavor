pub mod plugin;

use flavor_core::SourceText;
use flavor_grammar::{parse_g4_source_validated, G4Source, GrammarError};
use flavor_shared::PluginState;

pub const PLUGIN_ID: &str = "flavor-plugin-g4";
pub const G4_PARSE_ERROR: &str = "g4/parse/error";
pub const RULES: &[&str] = &[G4_PARSE_ERROR];

#[derive(Debug, Clone, Default)]
pub struct G4PluginConfig;

pub type G4PluginState = PluginState<G4PluginConfig>;

#[derive(Debug, Clone)]
pub struct G4AnalysisOutput {
    pub source: SourceText,
    pub grammar: Option<G4Source>,
    pub diagnostics: Vec<GrammarError>,
}

#[derive(Debug, Clone)]
pub struct G4PluginAnalyzer {
    state: G4PluginState,
}

impl G4PluginAnalyzer {
    pub fn new(config: G4PluginConfig) -> Self {
        Self {
            state: G4PluginState::new(config),
        }
    }

    pub fn run(&mut self, source: SourceText) -> G4AnalysisOutput {
        let _config = self.state.config();
        match parse_g4_source_validated(source.as_str()) {
            Ok(grammar) => G4AnalysisOutput {
                source,
                grammar: Some(grammar),
                diagnostics: Vec::new(),
            },
            Err(diagnostics) => G4AnalysisOutput {
                source,
                grammar: None,
                diagnostics,
            },
        }
    }
}

pub fn run(source: SourceText, config: G4PluginConfig) -> G4AnalysisOutput {
    G4PluginAnalyzer::new(config).run(source)
}
