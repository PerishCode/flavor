use crate::syntax_kind::TsSyntaxKind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_binding_name(&mut self, message: &str) -> bool {
        match self.current() {
            kind if is_binding_name(kind) => {
                self.bump();
                true
            }
            TsSyntaxKind::OpenBrace => {
                self.parse_object_binding_pattern();
                true
            }
            TsSyntaxKind::OpenBracket => {
                self.parse_array_binding_pattern();
                true
            }
            _ => {
                self.error_here(message);
                if !self.at_any(binding_stops()) {
                    self.bump();
                }
                false
            }
        }
    }

    pub(super) fn parse_rest_element(&mut self, message: &str) {
        self.builder.start_node(TsSyntaxKind::RestElement);
        self.bump();
        self.parse_binding_name(message);
        self.builder.finish_node();
    }

    fn parse_object_binding_pattern(&mut self) {
        self.builder.start_node(TsSyntaxKind::ObjectBindingPattern);
        if self.expect(
            TsSyntaxKind::OpenBrace,
            "expected '{' to start binding pattern",
        ) {
            while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
                self.parse_binding_element(TsSyntaxKind::CloseBrace);
                if self.at(TsSyntaxKind::Comma) {
                    self.bump();
                } else if !self.at(TsSyntaxKind::CloseBrace) {
                    self.error_here("expected ',' or '}' in binding pattern");
                    break;
                }
            }
            self.expect(
                TsSyntaxKind::CloseBrace,
                "expected '}' to close binding pattern",
            );
        }
        self.builder.finish_node();
    }

    fn parse_array_binding_pattern(&mut self) {
        self.builder.start_node(TsSyntaxKind::ArrayBindingPattern);
        if self.expect(
            TsSyntaxKind::OpenBracket,
            "expected '[' to start binding pattern",
        ) {
            while !self.at_any(&[TsSyntaxKind::CloseBracket, TsSyntaxKind::EndOfFile]) {
                if self.at(TsSyntaxKind::Comma) {
                    self.bump();
                    continue;
                }
                self.parse_binding_element(TsSyntaxKind::CloseBracket);
                if self.at(TsSyntaxKind::Comma) {
                    self.bump();
                } else if !self.at(TsSyntaxKind::CloseBracket) {
                    self.error_here("expected ',' or ']' in binding pattern");
                    break;
                }
            }
            self.expect(
                TsSyntaxKind::CloseBracket,
                "expected ']' to close binding pattern",
            );
        }
        self.builder.finish_node();
    }

    fn parse_binding_element(&mut self, close: TsSyntaxKind) {
        self.builder.start_node(TsSyntaxKind::BindingElement);
        if self.at(TsSyntaxKind::DotDotDot) {
            self.parse_rest_element("expected rest binding target");
        } else if self.at(TsSyntaxKind::OpenBrace) || self.at(TsSyntaxKind::OpenBracket) {
            self.parse_binding_name("expected binding target");
        } else if is_binding_key(self.current()) {
            self.bump();
            if self.at(TsSyntaxKind::Colon) {
                self.bump();
                self.parse_binding_name("expected binding target");
            }
        } else {
            self.error_here("expected binding element");
            if !self.at_any(&[TsSyntaxKind::Comma, close, TsSyntaxKind::EndOfFile]) {
                self.bump();
            }
        }
        if self.at(TsSyntaxKind::Equals) {
            self.parse_initializer(&[TsSyntaxKind::Comma, close]);
        }
        self.builder.finish_node();
    }
}

fn binding_stops() -> &'static [TsSyntaxKind] {
    &[
        TsSyntaxKind::Comma,
        TsSyntaxKind::Semicolon,
        TsSyntaxKind::CloseParen,
        TsSyntaxKind::CloseBrace,
        TsSyntaxKind::CloseBracket,
        TsSyntaxKind::Equals,
        TsSyntaxKind::EndOfFile,
    ]
}

fn is_binding_name(kind: TsSyntaxKind) -> bool {
    matches!(
        kind,
        TsSyntaxKind::Identifier
            | TsSyntaxKind::KeywordSatisfies
            | TsSyntaxKind::KeywordKeyof
            | TsSyntaxKind::KeywordInfer
            | TsSyntaxKind::KeywordUnique
    )
}

fn is_binding_key(kind: TsSyntaxKind) -> bool {
    matches!(
        kind,
        TsSyntaxKind::StringLiteral | TsSyntaxKind::NumericLiteral
    ) || is_binding_name(kind)
}
