use crate::internal::grammar::{self as kind, Kind};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_jsx_element(&mut self) {
        self.builder.start_node(kind::JSX_ELEMENT);
        let Some(name) = self.parse_jsx_opening() else {
            self.builder.finish_node();
            return;
        };
        if self.was_jsx_self_closing() {
            self.builder.finish_node();
            return;
        }
        while !self.at(kind::END_OF_FILE) {
            let start = self.cursor;
            if self.starts_jsx_close() {
                self.parse_jsx_closing();
                self.builder.finish_node();
                return;
            }
            if self.starts_jsx_open() {
                self.parse_jsx_element();
            } else if self.at(kind::OPEN_BRACE) {
                self.parse_balanced_node(
                    kind::JSX_EXPRESSION,
                    kind::OPEN_BRACE,
                    kind::CLOSE_BRACE,
                    "expected '}' to close JSX expression",
                );
            } else {
                self.parse_jsx_text();
            }
            self.ensure_progress(start, "JSX element");
        }
        self.error_at(
            self.current_span(),
            &format!("expected closing JSX tag for {name}"),
        );
        self.builder.finish_node();
    }

    pub(super) fn starts_jsx_open(&self) -> bool {
        self.at(kind::LESS_THAN) && is_jsx_name_start(self.token_kind_at(1))
    }

    pub(super) fn starts_jsx_expression_open(&self) -> bool {
        self.starts_jsx_open() && can_start_jsx_after(self.token_kind_at_back(1))
    }

    fn parse_jsx_opening(&mut self) -> Option<String> {
        self.builder.start_node(kind::JSX_OPENING_ELEMENT);
        self.bump();
        let Some(name) = self.parse_jsx_name("expected JSX tag name") else {
            self.builder.finish_node();
            return None;
        };
        while !self.at_any(&[kind::GREATER_THAN, kind::END_OF_FILE]) {
            let start = self.cursor;
            if self.at(kind::SLASH) && self.next_is(kind::GREATER_THAN) {
                self.bump();
                self.bump();
                self.builder.finish_node();
                return Some(name);
            }
            self.parse_jsx_attribute();
            self.ensure_progress(start, "JSX opening element");
        }
        if self.at(kind::GREATER_THAN) {
            self.bump();
        } else {
            self.error_here("expected '>' to close JSX opening tag");
        }
        self.builder.finish_node();
        Some(name)
    }

    fn parse_jsx_closing(&mut self) {
        self.builder.start_node(kind::JSX_CLOSING_ELEMENT);
        self.bump();
        self.bump();
        self.parse_jsx_name("expected JSX closing tag name");
        if self.at(kind::GREATER_THAN) {
            self.bump();
        } else {
            self.error_here("expected '>' to close JSX closing tag");
        }
        self.builder.finish_node();
    }

    fn parse_jsx_attribute(&mut self) {
        if self.at(kind::OPEN_BRACE) {
            self.builder.start_node(kind::JSX_SPREAD_ATTRIBUTE);
            self.parse_balanced_node(
                kind::JSX_EXPRESSION,
                kind::OPEN_BRACE,
                kind::CLOSE_BRACE,
                "expected '}' to close JSX spread attribute",
            );
            self.builder.finish_node();
            return;
        }
        if is_jsx_name_start(self.current()) {
            self.builder.start_node(kind::JSX_ATTRIBUTE);
            self.parse_jsx_name("expected JSX attribute name");
            if self.at(kind::EQUALS) {
                self.bump();
                if self.at(kind::STRING_LITERAL) {
                    self.bump();
                } else if self.at(kind::OPEN_BRACE) {
                    self.parse_balanced_node(
                        kind::JSX_EXPRESSION,
                        kind::OPEN_BRACE,
                        kind::CLOSE_BRACE,
                        "expected '}' to close JSX attribute expression",
                    );
                }
            }
            self.builder.finish_node();
        } else {
            self.bump();
        }
    }

    fn parse_jsx_text(&mut self) {
        self.builder.start_node(kind::JSX_TEXT);
        while !self.at(kind::END_OF_FILE)
            && !self.at(kind::OPEN_BRACE)
            && !self.starts_jsx_open()
            && !self.starts_jsx_close()
        {
            let start = self.cursor;
            self.bump();
            self.ensure_progress(start, "JSX text");
        }
        self.builder.finish_node();
    }

    fn parse_jsx_name(&mut self, message: &str) -> Option<String> {
        if !is_jsx_name_start(self.current()) {
            self.error_here(message);
            return None;
        }
        let start = self.current_span().start;
        self.bump();
        while is_jsx_name_join(self.current()) && is_jsx_name_part(self.token_kind_at(1)) {
            let start = self.cursor;
            self.bump();
            self.bump();
            self.ensure_progress(start, "JSX name");
        }
        let end = self.token_kind_at_back(1);
        debug_assert!(end != kind::END_OF_FILE);
        let span_end = self
            .token_at(self.cursor.saturating_sub(1))
            .map_or(start, |token| token.span.end);
        Some(
            self.source
                .slice(flavor_core::Span::new(start, span_end))
                .to_string(),
        )
    }

    fn starts_jsx_close(&self) -> bool {
        self.at(kind::LESS_THAN) && self.next_is(kind::SLASH)
    }

    fn was_jsx_self_closing(&self) -> bool {
        self.token_kind_at_back(1) == kind::GREATER_THAN
            && self.token_kind_at_back(2) == kind::SLASH
    }
}

fn is_jsx_name_start(kind: Kind) -> bool {
    kind == kind::IDENTIFIER || is_jsx_keyword(kind)
}

fn is_jsx_keyword(kind: Kind) -> bool {
    matches!(
        kind,
        kind::KEYWORD_ABSTRACT
            | kind::KEYWORD_AS
            | kind::KEYWORD_ASYNC
            | kind::KEYWORD_AWAIT
            | kind::KEYWORD_CLASS
            | kind::KEYWORD_CONST
            | kind::KEYWORD_DEFAULT
            | kind::KEYWORD_EXPORT
            | kind::KEYWORD_FOR
            | kind::KEYWORD_FROM
            | kind::KEYWORD_FUNCTION
            | kind::KEYWORD_GET
            | kind::KEYWORD_IF
            | kind::KEYWORD_IMPORT
            | kind::KEYWORD_IN
            | kind::KEYWORD_INTERFACE
            | kind::KEYWORD_LET
            | kind::KEYWORD_NEW
            | kind::KEYWORD_OF
            | kind::KEYWORD_READONLY
            | kind::KEYWORD_SET
            | kind::KEYWORD_STATIC
            | kind::KEYWORD_THIS
            | kind::KEYWORD_TYPE
    )
}

fn is_jsx_name_part(kind: Kind) -> bool {
    is_jsx_name_start(kind) || matches!(kind, kind::NUMERIC_LITERAL)
}

fn is_jsx_name_join(kind: Kind) -> bool {
    matches!(kind, kind::DOT | kind::COLON | kind::MINUS)
}

fn can_start_jsx_after(kind: Kind) -> bool {
    matches!(
        kind,
        kind::END_OF_FILE
            | kind::OPEN_PAREN
            | kind::OPEN_BRACKET
            | kind::OPEN_BRACE
            | kind::COMMA
            | kind::SEMICOLON
            | kind::COLON
            | kind::QUESTION
            | kind::EQUALS
            | kind::ARROW
            | kind::BANG
            | kind::PLUS
            | kind::MINUS
            | kind::STAR
            | kind::SLASH
            | kind::PERCENT
            | kind::AMPERSAND_AMPERSAND
            | kind::PIPE_PIPE
            | kind::QUESTION_QUESTION
            | kind::KEYWORD_RETURN
            | kind::KEYWORD_THROW
            | kind::KEYWORD_CASE
            | kind::KEYWORD_YIELD
    )
}
