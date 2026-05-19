use flavor_core::{Diagnostic, RawSyntaxKind, SourceText, Span, SyntaxToken};
use flavor_grammar::RawAstSchema;
use flavor_plugin_typescript::{run as run_ts, TsPluginConfig};

use super::{kind, TemplateAst};

const PREFIX: &str = "const __flavor_template_expr = ";
const SUFFIX: &str = ";";

pub fn validate_expressions(ast: &TemplateAst) -> Vec<Diagnostic> {
    let schema = kind::schema();
    let mut diagnostics = Vec::new();
    for token in ast.syntax().descendants_with_tokens() {
        let Some(token) = token.into_token() else {
            continue;
        };
        if let Some(expression) = template_expression(&schema, &token) {
            diagnostics.extend(validate_expression(expression));
        }
    }
    diagnostics
}

fn validate_expression(expression: TemplateExpression<'_>) -> Vec<Diagnostic> {
    let trimmed = expression.text.trim();
    if trimmed.is_empty() {
        return vec![Diagnostic::error_code(
            Span::new(expression.start, expression.start),
            "vue/parse/error",
            "empty template expression".to_string(),
        )];
    }

    let leading = expression.text.len() - expression.text.trim_start().len();
    let source_start = expression.start.saturating_add(to_u32(leading));
    let source = format!("{PREFIX}{}{SUFFIX}", expression.text.trim());
    let output = run_ts(
        SourceText::new("template-expression.ts", source),
        TsPluginConfig::default(),
    );
    output
        .diagnostics
        .into_iter()
        .map(|diagnostic| map_diagnostic(diagnostic, source_start, trimmed.len()))
        .collect()
}

fn template_expression<'a>(
    schema: &RawAstSchema,
    token: &'a SyntaxToken,
) -> Option<TemplateExpression<'a>> {
    if is_raw(schema, token.kind(), kind::EXPRESSION_TEXT) {
        return Some(TemplateExpression {
            text: token.text(),
            start: token_start(token),
        });
    }

    if is_raw(schema, token.kind(), kind::DIRECTIVE_ARGUMENT) {
        let text = token.text();
        let (inner, offset) = strip_dynamic_arg(text)?;
        return Some(TemplateExpression {
            text: inner,
            start: token_start(token).saturating_add(to_u32(offset)),
        });
    }

    if !is_raw(schema, token.kind(), kind::ATTRIBUTE_VALUE) {
        return None;
    }
    let parent = token.parent()?;
    if !is_raw(schema, parent.kind(), kind::DIRECTIVE_EXPRESSION) {
        return None;
    }
    let text = token.text();
    let start = token_start(token);
    let Some((inner, offset)) = strip_attribute_quotes(text) else {
        return Some(TemplateExpression { text, start });
    };
    Some(TemplateExpression {
        text: inner,
        start: start.saturating_add(to_u32(offset)),
    })
}

fn is_raw(schema: &RawAstSchema, raw: RawSyntaxKind, kind: kind::Kind) -> bool {
    schema.raw_kind_name(raw) == Some(kind)
}

fn map_diagnostic(
    diagnostic: Diagnostic,
    expression_start: u32,
    expression_len: usize,
) -> Diagnostic {
    let expression_len = to_u32(expression_len);
    Diagnostic {
        severity: diagnostic.severity,
        code: Some("vue/parse/error".to_string()),
        span: diagnostic.span.map(|span| {
            let start = span.start.saturating_sub(to_u32(PREFIX.len()));
            let end = span.end.saturating_sub(to_u32(PREFIX.len()));
            Span::new(
                expression_start.saturating_add(start.min(expression_len)),
                expression_start.saturating_add(end.min(expression_len)),
            )
        }),
        message: diagnostic.message,
    }
}

fn strip_attribute_quotes(text: &str) -> Option<(&str, usize)> {
    let quote = text.as_bytes().first().copied()?;
    if !matches!(quote, b'\'' | b'"') || text.as_bytes().last().copied() != Some(quote) {
        return None;
    }
    Some((&text[1..text.len().saturating_sub(1)], 1))
}

fn strip_dynamic_arg(text: &str) -> Option<(&str, usize)> {
    let open = text.find('[')?;
    let close = text.rfind(']')?;
    if close <= open {
        return None;
    }
    Some((&text[open + 1..close], open + 1))
}

fn token_start(token: &SyntaxToken) -> u32 {
    token.text_range().start().into()
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

struct TemplateExpression<'a> {
    text: &'a str,
    start: u32,
}
