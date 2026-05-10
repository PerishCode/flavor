use std::path::Path;

use flavor_compiler_core::{Diagnostic, SourceText};
use flavor_compiler_svelte::{SvelteBlock, SvelteCompileOutput, SvelteCompilerConfig};

use crate::{
    config::{GuardConfig, NodeKind, RuleSettings},
    model::{issue, Issue},
    naming::ts::{check_ts_script_blocks, TsScriptBlock},
    rules::{
        PAYLOAD_MAX_BLOCKS, PAYLOAD_MAX_LINES, SVELTE_COMPONENT_TOO_LONG, SVELTE_SCRIPT_TOO_LONG,
        SVELTE_STYLE_TOO_LONG, SVELTE_TEMPLATE_TOO_COMPLEX,
    },
};

pub(crate) fn check_svelte_names(
    config: &GuardConfig,
    relative: &Path,
    path: &str,
    source: &str,
    issues: &mut Vec<Issue>,
    svelte_parse_rule: &RuleSettings,
    ts_parse_rule: &RuleSettings,
) {
    let source_text = SourceText::new(path, source);
    let line_index = source_text.line_index();
    let output = flavor_compiler_svelte::run(source_text, SvelteCompilerConfig::default());
    for diagnostic in &output.diagnostics {
        let line = diagnostic
            .span
            .map(|span| line_index.position(span.start).line as usize);
        push_svelte_parse_issue(issues, svelte_parse_rule, path, diagnostic, line);
    }
    check_svelte_shape(config, relative, path, &output, issues);

    let scripts = output
        .descriptor
        .module_script
        .into_iter()
        .chain(output.descriptor.script)
        .filter_map(svelte_script_block)
        .collect();
    check_ts_script_blocks(config, relative, path, scripts, issues, ts_parse_rule);
}

fn check_svelte_shape(
    config: &GuardConfig,
    relative: &Path,
    path: &str,
    output: &SvelteCompileOutput,
    issues: &mut Vec<Issue>,
) {
    let component_rule = config.rule(relative, NodeKind::File, SVELTE_COMPONENT_TOO_LONG);
    let script_rule = config.rule(relative, NodeKind::File, SVELTE_SCRIPT_TOO_LONG);
    let style_rule = config.rule(relative, NodeKind::File, SVELTE_STYLE_TOO_LONG);
    let template_rule = config.rule(relative, NodeKind::File, SVELTE_TEMPLATE_TOO_COMPLEX);
    let line_count = output.source.as_str().lines().count();
    let facts = &output.facts;

    if component_rule.enabled {
        let max_lines = component_rule.usize(PAYLOAD_MAX_LINES).unwrap_or(500);
        if line_count > max_lines {
            issues.push(issue(
                component_rule.severity,
                component_rule.id,
                path,
                None,
                format!(
                    "Svelte component has {line_count} lines; max is {max_lines}; breakdown: script {} lines, markup {} non-empty lines, style {} lines",
                    facts.script_lines, facts.markup_lines, facts.style_lines
                ),
            ));
        }
    }

    if script_rule.enabled {
        let max_lines = script_rule.usize(PAYLOAD_MAX_LINES).unwrap_or(180);
        if facts.script_lines > max_lines {
            issues.push(issue(
                script_rule.severity,
                script_rule.id,
                path,
                None,
                format!(
                    "Svelte script spans {} lines across {} block(s); max is {max_lines}",
                    facts.script_lines, facts.script_count
                ),
            ));
        }
    }

    if style_rule.enabled {
        let max_lines = style_rule.usize(PAYLOAD_MAX_LINES).unwrap_or(240);
        if facts.style_lines > max_lines {
            issues.push(issue(
                style_rule.severity,
                style_rule.id,
                path,
                None,
                format!(
                    "Svelte style spans {} lines across {} block(s); max is {max_lines}",
                    facts.style_lines, facts.style_count
                ),
            ));
        }
    }

    if template_rule.enabled {
        let max_blocks = template_rule.usize(PAYLOAD_MAX_BLOCKS).unwrap_or(18);
        if facts.markup_block_count > max_blocks {
            issues.push(issue(
                template_rule.severity,
                template_rule.id,
                path,
                None,
                format!(
                    "Svelte template has {} control block(s), {} branch tag(s), and {} render tag(s); max blocks is {max_blocks}",
                    facts.markup_block_count, facts.markup_branch_count, facts.markup_render_count
                ),
            ));
        }
    }
}

fn svelte_script_block(block: SvelteBlock) -> Option<TsScriptBlock> {
    if block.content.trim().is_empty() {
        return None;
    }
    let lang = block
        .attrs
        .get("lang")
        .and_then(|value| value.as_deref())
        .map(|value| value.to_ascii_lowercase());
    let tsx = match lang.as_deref() {
        None | Some("js" | "ts") => false,
        Some("jsx" | "tsx") => true,
        _ => return None,
    };
    Some(TsScriptBlock {
        content: block.content,
        start_line: block.start_line,
        tsx,
    })
}

fn push_svelte_parse_issue(
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
    path: &str,
    diagnostic: &Diagnostic,
    line: Option<usize>,
) {
    if !rule.enabled {
        return;
    }

    issues.push(issue(
        rule.severity,
        rule.id,
        path,
        line,
        format!("failed to parse Svelte source: {}", diagnostic.message),
    ));
}
