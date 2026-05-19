use crate::internal::grammar::{self as kind, Kind};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_binding_name(&mut self, message: &str) -> bool {
        match self.current() {
            kind if is_binding_name(kind) => {
                self.bump();
                true
            }
            kind::OPEN_BRACE => {
                self.parse_object_binding_pattern();
                true
            }
            kind::OPEN_BRACKET => {
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
        self.builder.start_node(kind::REST_ELEMENT);
        self.bump();
        self.parse_binding_name(message);
        self.builder.finish_node();
    }

    fn parse_object_binding_pattern(&mut self) {
        self.builder.start_node(kind::OBJECT_BINDING_PATTERN);
        if self.expect(kind::OPEN_BRACE, "expected '{' to start binding pattern") {
            while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
                self.parse_binding_element(kind::CLOSE_BRACE);
                if self.at(kind::COMMA) {
                    self.bump();
                } else if !self.at(kind::CLOSE_BRACE) {
                    self.error_here("expected ',' or '}' in binding pattern");
                    break;
                }
            }
            self.expect(kind::CLOSE_BRACE, "expected '}' to close binding pattern");
        }
        self.builder.finish_node();
    }

    fn parse_array_binding_pattern(&mut self) {
        self.builder.start_node(kind::ARRAY_BINDING_PATTERN);
        if self.expect(kind::OPEN_BRACKET, "expected '[' to start binding pattern") {
            while !self.at_any(&[kind::CLOSE_BRACKET, kind::END_OF_FILE]) {
                if self.at(kind::COMMA) {
                    self.bump();
                    continue;
                }
                self.parse_binding_element(kind::CLOSE_BRACKET);
                if self.at(kind::COMMA) {
                    self.bump();
                } else if !self.at(kind::CLOSE_BRACKET) {
                    self.error_here("expected ',' or ']' in binding pattern");
                    break;
                }
            }
            self.expect(kind::CLOSE_BRACKET, "expected ']' to close binding pattern");
        }
        self.builder.finish_node();
    }

    fn parse_binding_element(&mut self, close: Kind) {
        self.builder.start_node(kind::BINDING_ELEMENT);
        if self.at(kind::DOT_DOT_DOT) {
            self.parse_rest_element("expected rest binding target");
        } else if self.at(kind::OPEN_BRACE) || self.at(kind::OPEN_BRACKET) {
            self.parse_binding_name("expected binding target");
        } else if is_binding_key(self.current()) {
            self.bump();
            if self.at(kind::COLON) {
                self.bump();
                self.parse_binding_name("expected binding target");
            }
        } else {
            self.error_here("expected binding element");
            if !self.at_any(&[kind::COMMA, close, kind::END_OF_FILE]) {
                self.bump();
            }
        }
        if self.at(kind::EQUALS) {
            self.parse_initializer(&[kind::COMMA, close]);
        }
        self.builder.finish_node();
    }
}

fn binding_stops() -> &'static [Kind] {
    &[
        kind::COMMA,
        kind::SEMICOLON,
        kind::CLOSE_PAREN,
        kind::CLOSE_BRACE,
        kind::CLOSE_BRACKET,
        kind::EQUALS,
        kind::END_OF_FILE,
    ]
}

fn is_binding_name(kind: Kind) -> bool {
    matches!(
        kind,
        kind::IDENTIFIER
            | kind::KEYWORD_SATISFIES
            | kind::KEYWORD_KEYOF
            | kind::KEYWORD_INFER
            | kind::KEYWORD_UNIQUE
    )
}

fn is_binding_key(kind: Kind) -> bool {
    matches!(kind, kind::STRING_LITERAL | kind::NUMERIC_LITERAL) || is_binding_name(kind)
}
