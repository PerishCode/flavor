use flavor_core::{Diagnostic, RawSyntaxKind, SourceText, Span, SyntaxNode, SyntaxToken};
use flavor_grammar::{
    find_balanced_brace_close as find_mustache_end, markup_char_at, RawAstSchema,
};
use flavor_plugin_typescript::{run as run_ts, TsPluginConfig};

use super::{kind, SvelteMarkupAst};

const PREFIX: &str = "const __flavor_svelte_expr = ";
const SUFFIX: &str = ";";

pub fn validate_expressions(ast: &SvelteMarkupAst) -> Vec<Diagnostic> {
    let schema = kind::schema();
    let mut diagnostics = Vec::new();
    for token in ast.syntax().descendants_with_tokens() {
        let Some(token) = token.into_token() else {
            continue;
        };
        for expression in token_expressions(schema, &token) {
            diagnostics.extend(validate_expression(expression));
        }
    }
    diagnostics
}

fn validate_expression(expression: MarkupExpression<'_>) -> Vec<Diagnostic> {
    let trimmed = expression.text.trim();
    if trimmed.is_empty() {
        return vec![Diagnostic::error_code(
            Span::new(expression.start, expression.start),
            "svelte/parse/error",
            "empty Svelte expression".to_string(),
        )];
    }

    let leading = expression.text.len() - expression.text.trim_start().len();
    let source_start = expression.start.saturating_add(to_u32(leading));
    let source = format!("{PREFIX}{trimmed}{SUFFIX}");
    let output = run_ts(
        SourceText::new("svelte-expression.ts", source),
        TsPluginConfig::default(),
    );
    output
        .diagnostics
        .into_iter()
        .map(|diagnostic| map_diagnostic(diagnostic, source_start, trimmed.len()))
        .collect()
}

fn token_expressions<'a>(
    schema: &RawAstSchema,
    token: &'a SyntaxToken,
) -> Vec<MarkupExpression<'a>> {
    if is_raw(schema, token.kind(), kind::EXPRESSION_TEXT) {
        return expression_text(schema, token);
    }

    if !is_raw(schema, token.kind(), kind::ATTRIBUTE_VALUE) {
        return Vec::new();
    }

    let Some(parent) = token.parent() else {
        return Vec::new();
    };
    let directive = is_raw(schema, parent.kind(), kind::DIRECTIVE_EXPRESSION);
    attribute_expressions(token.text(), token_start(token), directive)
}

fn expression_text<'a>(schema: &RawAstSchema, token: &'a SyntaxToken) -> Vec<MarkupExpression<'a>> {
    let Some(parent) = token.parent() else {
        return Vec::new();
    };
    match node_kind(schema, &parent) {
        kind::MUSTACHE => simple_expression(token),
        kind::SPREAD_ATTRIBUTE => spread_expression(token),
        kind::RENDER_TAG => simple_expression(token),
        kind::SPECIAL_TAG => simple_expression(token),
        kind::BLOCK_OPEN | kind::BLOCK_BRANCH => block_expressions(schema, &parent, token),
        _ => Vec::new(),
    }
}

fn block_expressions<'a>(
    schema: &RawAstSchema,
    node: &SyntaxNode,
    token: &'a SyntaxToken,
) -> Vec<MarkupExpression<'a>> {
    let Some(keyword) = block_keyword(schema, node) else {
        return Vec::new();
    };
    let text = token.text();
    let start = token_start(token);
    match (node_kind(schema, node), keyword.as_str()) {
        (kind::BLOCK_OPEN, "if" | "key" | "await") => {
            vec![MarkupExpression { text, start }]
        }
        (kind::BLOCK_OPEN, "each") => each_expressions(text, start),
        (kind::BLOCK_OPEN, "snippet") => Vec::new(),
        (kind::BLOCK_BRANCH, "else") => strip_else_if(text, start)
            .map(|expression| vec![expression])
            .unwrap_or_default(),
        (kind::BLOCK_BRANCH, "then" | "catch") => Vec::new(),
        _ => vec![MarkupExpression { text, start }],
    }
}

fn each_expressions(text: &str, start: u32) -> Vec<MarkupExpression<'_>> {
    let Some(as_offset) = find_top_keyword(text, "as") else {
        return vec![MarkupExpression { text, start }];
    };
    vec![MarkupExpression {
        text: &text[..as_offset],
        start,
    }]
}

fn strip_else_if(text: &str, start: u32) -> Option<MarkupExpression<'_>> {
    let trimmed = text.trim_start();
    let leading = text.len() - trimmed.len();
    let after_if = trimmed.strip_prefix("if")?;
    if after_if
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return None;
    }
    Some(MarkupExpression {
        text: after_if,
        start: start.saturating_add(to_u32(leading + 2)),
    })
}

fn attribute_expressions(text: &str, start: u32, directive: bool) -> Vec<MarkupExpression<'_>> {
    if let Some((inner, offset)) = strip_braces(text) {
        return vec![MarkupExpression {
            text: inner,
            start: start.saturating_add(to_u32(offset)),
        }];
    }

    if let Some((inner, offset)) = strip_quotes(text) {
        let quoted_start = start.saturating_add(to_u32(offset));
        let expressions = braced_segments(inner, quoted_start);
        if !expressions.is_empty() {
            return expressions;
        }
        if directive {
            return vec![MarkupExpression {
                text: inner,
                start: quoted_start,
            }];
        }
        return Vec::new();
    }

    directive
        .then_some(MarkupExpression { text, start })
        .into_iter()
        .collect()
}

fn simple_expression(token: &SyntaxToken) -> Vec<MarkupExpression<'_>> {
    vec![MarkupExpression {
        text: token.text(),
        start: token_start(token),
    }]
}

fn spread_expression(token: &SyntaxToken) -> Vec<MarkupExpression<'_>> {
    let text = token.text();
    let Some(inner) = text
        .strip_prefix("{...")
        .and_then(|value| value.strip_suffix('}'))
    else {
        return Vec::new();
    };
    vec![MarkupExpression {
        text: inner,
        start: token_start(token).saturating_add(4),
    }]
}

fn braced_segments(text: &str, start: u32) -> Vec<MarkupExpression<'_>> {
    let mut expressions = Vec::new();
    let mut cursor = 0;
    while let Some(offset) = text[cursor..].find('{') {
        let open = cursor + offset;
        let Some(close) = find_mustache_end(text, open + 1) else {
            break;
        };
        expressions.push(MarkupExpression {
            text: &text[open + 1..close],
            start: start.saturating_add(to_u32(open + 1)),
        });
        cursor = close + 1;
    }
    expressions
}

fn block_keyword(schema: &RawAstSchema, node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| is_raw(schema, token.kind(), kind::BLOCK_KEYWORD))
        .map(|token| token.text().to_string())
}

fn find_top_keyword(text: &str, keyword: &str) -> Option<usize> {
    let mut cursor = 0;
    let mut quote = None;
    let mut depth = 0usize;
    while let Some((ch, width)) = markup_char_at(text, cursor) {
        if let Some(quote_char) = quote {
            if ch == '\\' {
                cursor += width;
                if let Some((_, escaped_width)) = markup_char_at(text, cursor) {
                    cursor += escaped_width;
                }
                continue;
            }
            if ch == quote_char {
                quote = None;
            }
            cursor += width;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 && keyword_at(text, cursor, keyword) => return Some(cursor),
            _ => {}
        }
        cursor += width;
    }
    None
}

fn keyword_at(text: &str, cursor: usize, keyword: &str) -> bool {
    if !text[cursor..].starts_with(keyword) {
        return false;
    }
    let before = text[..cursor].chars().next_back();
    let after = text[cursor + keyword.len()..].chars().next();
    before.is_some_and(char::is_whitespace) && after.is_some_and(char::is_whitespace)
}

fn strip_braces(text: &str) -> Option<(&str, usize)> {
    if !text.starts_with('{') || !text.ends_with('}') {
        return None;
    }
    Some((&text[1..text.len().saturating_sub(1)], 1))
}

fn strip_quotes(text: &str) -> Option<(&str, usize)> {
    let quote = text.as_bytes().first().copied()?;
    if !matches!(quote, b'\'' | b'"') || text.as_bytes().last().copied() != Some(quote) {
        return None;
    }
    Some((&text[1..text.len().saturating_sub(1)], 1))
}

fn map_diagnostic(
    diagnostic: Diagnostic,
    expression_start: u32,
    expression_len: usize,
) -> Diagnostic {
    let expression_len = to_u32(expression_len);
    Diagnostic {
        severity: diagnostic.severity,
        code: Some("svelte/parse/error".to_string()),
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

fn node_kind<'a>(schema: &'a RawAstSchema, node: &SyntaxNode) -> &'a str {
    schema.raw_kind_name(node.kind()).unwrap_or(kind::ERROR)
}

fn is_raw(schema: &RawAstSchema, raw: RawSyntaxKind, kind: kind::Kind) -> bool {
    schema.raw_kind_name(raw) == Some(kind)
}

fn token_start(token: &SyntaxToken) -> u32 {
    token.text_range().start().into()
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

struct MarkupExpression<'a> {
    text: &'a str,
    start: u32,
}
