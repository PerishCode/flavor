use flavor_shared::PluginState;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct RustPluginConfig {
    pub repeated_token_patterns: RustRepeatedTokenPatternConfig,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RustRepeatedTokenPatternConfig {
    pub min_occurrences: usize,
    pub min_total_lines: usize,
    pub min_lines: usize,
    pub max_lines: usize,
    pub min_tokens: usize,
    pub max_tokens: usize,
    pub min_nodes: usize,
    pub token_bucket_size: usize,
    pub max_reports: usize,
}

impl Default for RustRepeatedTokenPatternConfig {
    fn default() -> Self {
        Self {
            min_occurrences: 10,
            min_total_lines: 200,
            min_lines: 3,
            max_lines: 80,
            min_tokens: 8,
            max_tokens: 240,
            min_nodes: 4,
            token_bucket_size: 8,
            max_reports: 8,
        }
    }
}

pub type RustPluginState = PluginState<RustPluginConfig>;
