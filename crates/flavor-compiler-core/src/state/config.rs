#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct CompilerCoreConfig {
    pub recovery: RecoveryConfig,
    pub snapshot: SnapshotConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RecoveryConfig {
    pub keep_missing_nodes: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            keep_missing_nodes: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SnapshotConfig {
    pub include_trivia: bool,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            include_trivia: true,
        }
    }
}
