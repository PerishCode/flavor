use flavor_core::GrammarProduct;
use flavor_plugin_rust::{RustPluginConfig, RustRepeatedTokenPatternConfig};
use flavor_plugin_typescript::{TsFailureSurfaceConfig, TsPluginConfig};

use crate::{
    config::{GuardConfig, NodeKind, SourceKind},
    plugins::{PluginManifest, Scope},
    rules::{
        ERROR_FAILURE_SURFACE_MATURITY, PAYLOAD_MAX_LINES, PAYLOAD_MAX_REPORTS, PAYLOAD_MAX_TOKENS,
        PAYLOAD_MIN_LINES, PAYLOAD_MIN_NODES, PAYLOAD_MIN_OCCURRENCES, PAYLOAD_MIN_TOKENS,
        PAYLOAD_MIN_TOTAL_LINES, PAYLOAD_RAW_REJECT_CALLEES, PAYLOAD_STRUCTURED_FACTORIES,
        PAYLOAD_STRUCTURED_GUARDS, PAYLOAD_TOKEN_BUCKET_SIZE, SHAPE_REPEATED_TOKEN_PATTERN,
    },
};

pub(super) fn satisfy(
    config: &GuardConfig,
    manifest: PluginManifest,
    scope: Scope<'_>,
) -> Vec<GrammarProduct> {
    let mut products = Vec::new();
    let Some(source) = scope.source_file_data() else {
        return products;
    };
    if manifest.grammars.is_empty() {
        return products;
    }

    match source.kind {
        SourceKind::G4 => flavor_plugin_g4::plugin::satisfy(
            &|grammar_id| entrypoint(manifest, grammar_id),
            source.path,
            source.source,
            &mut products,
        ),
        SourceKind::Python => flavor_plugin_python::plugin::satisfy(
            &|grammar_id| entrypoint(manifest, grammar_id),
            source.path,
            source.source,
            &mut products,
        ),
        SourceKind::Rust => flavor_plugin_rust::plugin::satisfy_with_config(
            &|grammar_id| entrypoint(manifest, grammar_id),
            source.path,
            source.source,
            rust_config(config, &source),
            &mut products,
        ),
        SourceKind::TypeScript => flavor_plugin_typescript::plugin::satisfy_source_with_config(
            &|grammar_id| entrypoint(manifest, grammar_id),
            source.path,
            source.source,
            typescript_config(config, &source),
            &mut products,
        ),
        SourceKind::Vue => flavor_plugin_vue::plugin::satisfy_with_failure_surface(
            &|grammar_id| entrypoint(manifest, grammar_id),
            source.path,
            source.source,
            typescript_config(config, &source).failure_surface,
            &mut products,
        ),
        SourceKind::Svelte => flavor_plugin_svelte::plugin::satisfy_with_failure_surface(
            &|grammar_id| entrypoint(manifest, grammar_id),
            source.path,
            source.source,
            typescript_config(config, &source).failure_surface,
            &mut products,
        ),
    }

    products
}

fn typescript_config(
    config: &GuardConfig,
    source: &crate::plugins::SourceFileScope<'_>,
) -> TsPluginConfig {
    let rule = config.rule(
        source.relative,
        NodeKind::File,
        ERROR_FAILURE_SURFACE_MATURITY,
    );
    TsPluginConfig {
        failure_surface: TsFailureSurfaceConfig {
            structured_guards: rule
                .string_list(PAYLOAD_STRUCTURED_GUARDS)
                .unwrap_or_default(),
            structured_factories: rule
                .string_list(PAYLOAD_STRUCTURED_FACTORIES)
                .unwrap_or_default(),
            raw_reject_callees: rule
                .string_list(PAYLOAD_RAW_REJECT_CALLEES)
                .unwrap_or_else(|| TsFailureSurfaceConfig::default().raw_reject_callees),
        },
        ..Default::default()
    }
}

fn rust_config(
    config: &GuardConfig,
    source: &crate::plugins::SourceFileScope<'_>,
) -> RustPluginConfig {
    let rule = config.rule(
        source.relative,
        NodeKind::File,
        SHAPE_REPEATED_TOKEN_PATTERN,
    );
    let defaults = RustRepeatedTokenPatternConfig::default();
    let repeated_token_patterns = RustRepeatedTokenPatternConfig {
        min_occurrences: rule
            .usize(PAYLOAD_MIN_OCCURRENCES)
            .unwrap_or(defaults.min_occurrences),
        min_total_lines: rule
            .usize(PAYLOAD_MIN_TOTAL_LINES)
            .unwrap_or(defaults.min_total_lines),
        min_lines: rule.usize(PAYLOAD_MIN_LINES).unwrap_or(defaults.min_lines),
        max_lines: rule.usize(PAYLOAD_MAX_LINES).unwrap_or(defaults.max_lines),
        min_tokens: rule
            .usize(PAYLOAD_MIN_TOKENS)
            .unwrap_or(defaults.min_tokens),
        max_tokens: rule
            .usize(PAYLOAD_MAX_TOKENS)
            .unwrap_or(defaults.max_tokens),
        min_nodes: rule.usize(PAYLOAD_MIN_NODES).unwrap_or(defaults.min_nodes),
        token_bucket_size: rule
            .usize(PAYLOAD_TOKEN_BUCKET_SIZE)
            .unwrap_or(defaults.token_bucket_size),
        max_reports: rule
            .usize(PAYLOAD_MAX_REPORTS)
            .unwrap_or(defaults.max_reports),
    };
    tracing::debug!(
        path = source.path,
        min_occurrences = repeated_token_patterns.min_occurrences,
        min_total_lines = repeated_token_patterns.min_total_lines,
        min_lines = repeated_token_patterns.min_lines,
        max_lines = repeated_token_patterns.max_lines,
        min_tokens = repeated_token_patterns.min_tokens,
        max_tokens = repeated_token_patterns.max_tokens,
        min_nodes = repeated_token_patterns.min_nodes,
        token_bucket_size = repeated_token_patterns.token_bucket_size,
        max_reports = repeated_token_patterns.max_reports,
        "configured Rust repeated token pattern collector",
    );
    RustPluginConfig {
        repeated_token_patterns,
    }
}

fn entrypoint(manifest: PluginManifest, grammar_id: &str) -> Option<&'static str> {
    manifest
        .grammars
        .iter()
        .find(|grammar| grammar.grammar_id == grammar_id)
        .map(|grammar| grammar.entrypoint)
}
