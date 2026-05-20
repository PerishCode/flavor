use flavor_grammar::{GrammarNode, TokenTextRun};

use crate::{
    internal::grammar::{self as kind},
    model::TsStructuredFailureKind,
    state::TsFailureSurfaceConfig,
};

type TsNode = GrammarNode;

pub(super) const RAW_FAILURE_LITERAL_TOKENS: &[&str] = &["STRING_LITERAL", "TEMPLATE_LITERAL"];

const BUILTIN_ERROR_CONSTRUCTORS: &[&str] = &[
    "Error",
    "EvalError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "TypeError",
    "URIError",
];
const CALLEE_JOIN_TOKENS: &[&str] = &["DOT", "QUESTION_DOT"];
const CALLEE_PART_TOKENS: &[&str] = &["IDENTIFIER", "KEYWORD_SUPER", "KEYWORD_THIS"];

pub(super) fn constructor_name(node: &TsNode) -> Option<String> {
    if let Some(member) = node.child(kind::MEMBER_EXPRESSION) {
        return member.token_run_text(TokenTextRun::new(CALLEE_PART_TOKENS, CALLEE_JOIN_TOKENS));
    }
    node.child_token_text_any(&["IDENTIFIER"])
}

pub(super) fn throw_new_expression(expression: &TsNode) -> Option<TsNode> {
    expression.child(kind::NEW_EXPRESSION).or_else(|| {
        expression
            .child(kind::PARENTHESIZED_EXPRESSION)
            .and_then(|node| node.child(kind::NEW_EXPRESSION))
    })
}

pub(super) fn callee_name(node: &TsNode) -> Option<String> {
    if let Some(member) = node.child(kind::MEMBER_EXPRESSION) {
        return member.token_run_text(TokenTextRun::new(CALLEE_PART_TOKENS, CALLEE_JOIN_TOKENS));
    }
    node.head_token_text_any(CALLEE_PART_TOKENS)
}

pub(super) fn raw_error_argument_constructor(node: &TsNode) -> Option<String> {
    let arguments = node.child(kind::PARENTHESIZED_EXPRESSION)?;
    let tokens = token_views(&arguments);
    let mut after_outer_open = false;
    let mut depth = 0usize;
    let mut argument_start = false;

    for (index, token) in tokens.iter().enumerate() {
        if !after_outer_open {
            if token.kind == kind::OPEN_PAREN {
                after_outer_open = true;
                argument_start = true;
            }
            continue;
        }

        if depth == 0 && token.kind == kind::CLOSE_PAREN {
            break;
        }
        if depth == 0 && token.kind == kind::COMMA {
            argument_start = true;
            continue;
        }
        if depth == 0 && argument_start && token.kind == kind::KEYWORD_NEW {
            if let Some(constructor) = constructor_from_tokens(&tokens[index + 1..]) {
                if is_builtin_error_constructor(&constructor) {
                    return Some(constructor);
                }
            }
        }

        update_depth(&token.kind, &mut depth);
        if depth == 0 && token.kind != kind::COMMA {
            argument_start = false;
        }
    }

    None
}

pub(super) fn structured_failure_kind(
    callee: &str,
    config: &TsFailureSurfaceConfig,
) -> Option<TsStructuredFailureKind> {
    if matches_configured_callee(callee, &config.structured_guards) {
        Some(TsStructuredFailureKind::Guard)
    } else if matches_configured_callee(callee, &config.structured_factories) {
        Some(TsStructuredFailureKind::Factory)
    } else {
        None
    }
}

pub(super) fn matches_configured_callee(callee: &str, configured: &[String]) -> bool {
    configured
        .iter()
        .map(|value| value.trim())
        .any(|value| !value.is_empty() && callee_matches(callee, value))
}

pub(super) fn is_builtin_error_constructor(name: &str) -> bool {
    let leaf = name.rsplit_once('.').map_or(name, |(_, leaf)| leaf);
    BUILTIN_ERROR_CONSTRUCTORS.contains(&leaf)
}

#[derive(Debug, Clone)]
struct TokenView {
    kind: String,
    text: String,
}

fn token_views(node: &TsNode) -> Vec<TokenView> {
    node.tokens()
        .filter_map(|token| {
            Some(TokenView {
                kind: token.kind_name()?.to_string(),
                text: token.text().to_string(),
            })
        })
        .collect()
}

fn constructor_from_tokens(tokens: &[TokenView]) -> Option<String> {
    let first = tokens.first()?;
    if first.kind != kind::IDENTIFIER {
        return None;
    }
    let mut name = first.text.clone();
    let mut index = 1usize;
    while tokens
        .get(index)
        .is_some_and(|token| token.kind == kind::DOT)
    {
        let Some(part) = tokens.get(index + 1) else {
            break;
        };
        if part.kind != kind::IDENTIFIER {
            break;
        }
        name.push('.');
        name.push_str(&part.text);
        index += 2;
    }
    Some(name)
}

fn callee_matches(callee: &str, configured: &str) -> bool {
    callee == configured
        || callee
            .strip_prefix(configured)
            .is_some_and(|tail| tail.starts_with('.') || tail.starts_with("?."))
}

fn update_depth(kind: &str, depth: &mut usize) {
    match kind {
        kind::OPEN_PAREN | kind::OPEN_BRACKET | kind::OPEN_BRACE => *depth += 1,
        kind::CLOSE_PAREN | kind::CLOSE_BRACKET | kind::CLOSE_BRACE => {
            *depth = depth.saturating_sub(1);
        }
        _ => {}
    }
}
