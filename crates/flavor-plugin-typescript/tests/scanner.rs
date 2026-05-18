use flavor_plugin_core::{SourceText, TriviaKind};
use flavor_plugin_typescript::{lexer::scan, syntax_kind::TsSyntaxKind, TsPluginConfig};

fn kinds(source: &str) -> Vec<TsSyntaxKind> {
    scan(
        &SourceText::new("sample.ts", source),
        &TsPluginConfig::default(),
    )
    .into_iter()
    .map(|token| token.kind)
    .collect()
}

#[test]
fn scans_decorator_tokens() {
    assert_eq!(
        kinds("@sealed class Example {}"),
        vec![
            TsSyntaxKind::At,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::KeywordClass,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::OpenBrace,
            TsSyntaxKind::CloseBrace,
            TsSyntaxKind::EndOfFile,
        ]
    );
}

#[test]
fn scans_tsx_boundary() {
    assert_eq!(
        kinds("const node = <div />;"),
        vec![
            TsSyntaxKind::KeywordConst,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::Equals,
            TsSyntaxKind::LessThan,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::Slash,
            TsSyntaxKind::GreaterThan,
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::EndOfFile,
        ]
    );
}

#[test]
fn scans_rest_arrow() {
    assert_eq!(
        kinds("(...items) => items"),
        vec![
            TsSyntaxKind::OpenParen,
            TsSyntaxKind::DotDotDot,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::CloseParen,
            TsSyntaxKind::Arrow,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::EndOfFile,
        ]
    );
}

#[test]
fn scans_operator_modifiers() {
    assert_eq!(
        kinds("private readonly value = left?.id ?? right === next && ok;"),
        vec![
            TsSyntaxKind::KeywordPrivate,
            TsSyntaxKind::KeywordReadonly,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::Equals,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::QuestionDot,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::QuestionQuestion,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::EqualsEqualsEquals,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::AmpersandAmpersand,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::EndOfFile,
        ]
    );
}

#[test]
fn scans_module_keyword() {
    assert_eq!(
        kinds("declare module \"virtual:api\" {}"),
        vec![
            TsSyntaxKind::KeywordDeclare,
            TsSyntaxKind::KeywordModule,
            TsSyntaxKind::StringLiteral,
            TsSyntaxKind::OpenBrace,
            TsSyntaxKind::CloseBrace,
            TsSyntaxKind::EndOfFile,
        ]
    );
}

#[test]
fn scans_control_keywords() {
    assert_eq!(
        kinds("if else for of in while do switch case try catch finally throw break continue"),
        vec![
            TsSyntaxKind::KeywordIf,
            TsSyntaxKind::KeywordElse,
            TsSyntaxKind::KeywordFor,
            TsSyntaxKind::KeywordOf,
            TsSyntaxKind::KeywordIn,
            TsSyntaxKind::KeywordWhile,
            TsSyntaxKind::KeywordDo,
            TsSyntaxKind::KeywordSwitch,
            TsSyntaxKind::KeywordCase,
            TsSyntaxKind::KeywordTry,
            TsSyntaxKind::KeywordCatch,
            TsSyntaxKind::KeywordFinally,
            TsSyntaxKind::KeywordThrow,
            TsSyntaxKind::KeywordBreak,
            TsSyntaxKind::KeywordContinue,
            TsSyntaxKind::EndOfFile,
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
            TsSyntaxKind::KeywordThis,
            TsSyntaxKind::KeywordSuper,
            TsSyntaxKind::KeywordTrue,
            TsSyntaxKind::KeywordFalse,
            TsSyntaxKind::KeywordNull,
            TsSyntaxKind::KeywordInstanceof,
            TsSyntaxKind::KeywordTypeof,
            TsSyntaxKind::KeywordVoid,
            TsSyntaxKind::KeywordDelete,
            TsSyntaxKind::KeywordYield,
            TsSyntaxKind::KeywordSatisfies,
            TsSyntaxKind::KeywordKeyof,
            TsSyntaxKind::KeywordInfer,
            TsSyntaxKind::KeywordUnique,
            TsSyntaxKind::EndOfFile,
        ]
    );
}

#[test]
fn scans_number_literals() {
    assert_eq!(
        kinds("0x10 0b1010 0o77 1_000 12.5e-2 .5 123n 0xffn"),
        vec![
            TsSyntaxKind::NumericLiteral,
            TsSyntaxKind::NumericLiteral,
            TsSyntaxKind::NumericLiteral,
            TsSyntaxKind::NumericLiteral,
            TsSyntaxKind::NumericLiteral,
            TsSyntaxKind::NumericLiteral,
            TsSyntaxKind::BigIntLiteral,
            TsSyntaxKind::BigIntLiteral,
            TsSyntaxKind::EndOfFile,
        ]
    );
}

#[test]
fn scans_template_literal() {
    assert_eq!(
        kinds("const text = `hi ${items.map((item) => `${item.id}`).join(\",\")}`;"),
        vec![
            TsSyntaxKind::KeywordConst,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::Equals,
            TsSyntaxKind::TemplateLiteral,
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::EndOfFile,
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
            TsSyntaxKind::KeywordConst,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::Equals,
            TsSyntaxKind::RegexLiteral,
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::KeywordConst,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::Equals,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::Slash,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::KeywordReturn,
            TsSyntaxKind::RegexLiteral,
            TsSyntaxKind::Dot,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::OpenParen,
            TsSyntaxKind::Identifier,
            TsSyntaxKind::CloseParen,
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::EndOfFile,
        ]
    );
}

#[test]
fn keeps_leading_trivia() {
    let tokens = scan(
        &SourceText::new("sample.ts", "#!/usr/bin/env node\n// hi\nlet value = 1;"),
        &TsPluginConfig::default(),
    );

    assert_eq!(tokens[0].kind, TsSyntaxKind::KeywordLet);
    assert_eq!(tokens[0].leading[0].kind, TriviaKind::Shebang);
    assert_eq!(tokens[0].leading[1].kind, TriviaKind::LineComment);
}
