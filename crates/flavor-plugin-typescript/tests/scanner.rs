use flavor_core::{SourceText, TriviaKind};
use flavor_plugin_typescript::{run, TsPluginConfig};

#[path = "../src/internal/grammar.rs"]
mod kind;

use kind::Kind;

fn kinds(source: &str) -> Vec<Kind> {
    run(
        SourceText::new("sample.ts", source),
        TsPluginConfig::default(),
    )
    .tokens
    .into_iter()
    .map(|token| token.kind)
    .collect()
}

#[test]
fn scans_decorator_tokens() {
    assert_eq!(
        kinds("@sealed class Example {}"),
        vec![
            kind::AT,
            kind::IDENTIFIER,
            kind::KEYWORD_CLASS,
            kind::IDENTIFIER,
            kind::OPEN_BRACE,
            kind::CLOSE_BRACE,
            kind::END_OF_FILE,
        ]
    );
}

#[test]
fn scans_tsx_boundary() {
    assert_eq!(
        kinds("const node = <div />;"),
        vec![
            kind::KEYWORD_CONST,
            kind::IDENTIFIER,
            kind::EQUALS,
            kind::LESS_THAN,
            kind::IDENTIFIER,
            kind::SLASH,
            kind::GREATER_THAN,
            kind::SEMICOLON,
            kind::END_OF_FILE,
        ]
    );
}

#[test]
fn scans_rest_arrow() {
    assert_eq!(
        kinds("(...items) => items"),
        vec![
            kind::OPEN_PAREN,
            kind::DOT_DOT_DOT,
            kind::IDENTIFIER,
            kind::CLOSE_PAREN,
            kind::ARROW,
            kind::IDENTIFIER,
            kind::END_OF_FILE,
        ]
    );
}

#[test]
fn scans_operator_modifiers() {
    assert_eq!(
        kinds("private readonly value = left?.id ?? right === next && ok;"),
        vec![
            kind::KEYWORD_PRIVATE,
            kind::KEYWORD_READONLY,
            kind::IDENTIFIER,
            kind::EQUALS,
            kind::IDENTIFIER,
            kind::QUESTION_DOT,
            kind::IDENTIFIER,
            kind::QUESTION_QUESTION,
            kind::IDENTIFIER,
            kind::EQUALS_EQUALS_EQUALS,
            kind::IDENTIFIER,
            kind::AMPERSAND_AMPERSAND,
            kind::IDENTIFIER,
            kind::SEMICOLON,
            kind::END_OF_FILE,
        ]
    );
}

#[test]
fn scans_module_keyword() {
    assert_eq!(
        kinds("declare module \"virtual:api\" {}"),
        vec![
            kind::KEYWORD_DECLARE,
            kind::KEYWORD_MODULE,
            kind::STRING_LITERAL,
            kind::OPEN_BRACE,
            kind::CLOSE_BRACE,
            kind::END_OF_FILE,
        ]
    );
}

#[test]
fn scans_control_keywords() {
    assert_eq!(
        kinds("if else for of in while do switch case try catch finally throw break continue"),
        vec![
            kind::KEYWORD_IF,
            kind::KEYWORD_ELSE,
            kind::KEYWORD_FOR,
            kind::KEYWORD_OF,
            kind::KEYWORD_IN,
            kind::KEYWORD_WHILE,
            kind::KEYWORD_DO,
            kind::KEYWORD_SWITCH,
            kind::KEYWORD_CASE,
            kind::KEYWORD_TRY,
            kind::KEYWORD_CATCH,
            kind::KEYWORD_FINALLY,
            kind::KEYWORD_THROW,
            kind::KEYWORD_BREAK,
            kind::KEYWORD_CONTINUE,
            kind::END_OF_FILE,
        ]
    );
}

#[test]
fn scans_expression_keywords() {
    assert_eq!(
        kinds(
            "this super true false null instanceof typeof void delete yield satisfies keyof infer unique"
        ),
        vec![
            kind::KEYWORD_THIS,
            kind::KEYWORD_SUPER,
            kind::KEYWORD_TRUE,
            kind::KEYWORD_FALSE,
            kind::KEYWORD_NULL,
            kind::KEYWORD_INSTANCEOF,
            kind::KEYWORD_TYPEOF,
            kind::KEYWORD_VOID,
            kind::KEYWORD_DELETE,
            kind::KEYWORD_YIELD,
            kind::KEYWORD_SATISFIES,
            kind::KEYWORD_KEYOF,
            kind::KEYWORD_INFER,
            kind::KEYWORD_UNIQUE,
            kind::END_OF_FILE,
        ]
    );
}

#[test]
fn scans_number_literals() {
    assert_eq!(
        kinds("0x10 0b1010 0o77 1_000 12.5e-2 .5 123n 0xffn"),
        vec![
            kind::NUMERIC_LITERAL,
            kind::NUMERIC_LITERAL,
            kind::NUMERIC_LITERAL,
            kind::NUMERIC_LITERAL,
            kind::NUMERIC_LITERAL,
            kind::NUMERIC_LITERAL,
            kind::BIG_INT_LITERAL,
            kind::BIG_INT_LITERAL,
            kind::END_OF_FILE,
        ]
    );
}

#[test]
fn scans_template_literal() {
    assert_eq!(
        kinds("const text = `hi ${items.map((item) => `${item.id}`).join(\",\")}`;"),
        vec![
            kind::KEYWORD_CONST,
            kind::IDENTIFIER,
            kind::EQUALS,
            kind::TEMPLATE_LITERAL,
            kind::SEMICOLON,
            kind::END_OF_FILE,
        ]
    );
}

#[test]
fn scans_regex_literal() {
    assert_eq!(
        kinds(
            "const re = /foo\\/[a-z]+/gi; const ratio = left / right; return /done/.test(value);"
        ),
        vec![
            kind::KEYWORD_CONST,
            kind::IDENTIFIER,
            kind::EQUALS,
            kind::REGEX_LITERAL,
            kind::SEMICOLON,
            kind::KEYWORD_CONST,
            kind::IDENTIFIER,
            kind::EQUALS,
            kind::IDENTIFIER,
            kind::SLASH,
            kind::IDENTIFIER,
            kind::SEMICOLON,
            kind::KEYWORD_RETURN,
            kind::REGEX_LITERAL,
            kind::DOT,
            kind::IDENTIFIER,
            kind::OPEN_PAREN,
            kind::IDENTIFIER,
            kind::CLOSE_PAREN,
            kind::SEMICOLON,
            kind::END_OF_FILE,
        ]
    );
}

#[test]
fn keeps_leading_trivia() {
    let tokens = run(
        SourceText::new("sample.ts", "#!/usr/bin/env node\n// hi\nlet value = 1;"),
        TsPluginConfig::default(),
    )
    .tokens;

    assert_eq!(tokens[0].kind, kind::KEYWORD_LET);
    assert_eq!(tokens[0].leading[0].kind, TriviaKind::Shebang);
    assert_eq!(tokens[0].leading[1].kind, TriviaKind::LineComment);
}
