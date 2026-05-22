use crate::internal::grammar::{self as kind, Kind};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_expression(&mut self, stops: &[Kind]) {
        self.builder.start_node(kind::EXPRESSION);
        let shape = self.expression_shape(stops);
        if let Some(shape) = shape {
            self.builder.start_node(shape);
            self.parse_expression_tokens_until(stops);
            self.builder.finish_node();
        } else {
            self.parse_expression_tokens_until(stops);
        }
        self.builder.finish_node();
    }

    fn expression_shape(&self, stops: &[Kind]) -> Option<Kind> {
        if self.has_top_level_any(&[kind::ARROW], stops) {
            Some(kind::ARROW_FUNCTION)
        } else if self.has_top_level_any(&[kind::QUESTION], stops) {
            Some(kind::CONDITIONAL_EXPRESSION)
        } else if self.has_top_level_any(binary_operators(), stops) {
            Some(kind::BINARY_EXPRESSION)
        } else {
            None
        }
    }

    fn parse_expression_tokens_until(&mut self, stops: &[Kind]) {
        while !self.at(kind::END_OF_FILE) && !self.at_any(stops) {
            let start = self.cursor;
            match self.current() {
                kind::LESS_THAN if self.jsx_enabled() && self.starts_jsx_open() => {
                    self.parse_jsx_element();
                }
                kind::KEYWORD_AWAIT => self.parse_await_expression(),
                kind::KEYWORD_NEW => self.parse_new_expression(),
                kind if is_expr_base(kind) && self.next_is(kind::OPEN_PAREN) => {
                    self.parse_call_expression();
                }
                kind::IDENTIFIER if self.next_is(kind::LESS_THAN) => {
                    self.parse_typed_call();
                }
                kind if is_expr_base(kind)
                    && (self.next_is(kind::DOT) || self.next_is(kind::QUESTION_DOT)) =>
                {
                    self.parse_member_chain();
                }
                kind if is_unary_operator(kind) => self.parse_unary_expression(),
                kind::OPEN_BRACE => self.parse_balanced_node(
                    kind::OBJECT_EXPRESSION,
                    kind::OPEN_BRACE,
                    kind::CLOSE_BRACE,
                    "expected '}' to close object expression",
                ),
                kind::OPEN_BRACKET => self.parse_balanced_node(
                    kind::ARRAY_EXPRESSION,
                    kind::OPEN_BRACKET,
                    kind::CLOSE_BRACKET,
                    "expected ']' to close array expression",
                ),
                kind::OPEN_PAREN => self.parse_parenthesized_expression(),
                _ => self.bump(),
            }
            self.ensure_progress(start, "expression");
        }
    }

    fn parse_call_expression(&mut self) {
        self.builder.start_node(kind::CALL_EXPRESSION);
        self.bump();
        if self.at(kind::LESS_THAN) {
            self.parse_balanced_node(
                kind::TYPE_PARAMETERS,
                kind::LESS_THAN,
                kind::GREATER_THAN,
                "expected '>' to close type arguments",
            );
        }
        self.parse_balanced_node(
            kind::PARENTHESIZED_EXPRESSION,
            kind::OPEN_PAREN,
            kind::CLOSE_PAREN,
            "expected ')' to close call arguments",
        );
        self.builder.finish_node();
    }

    fn parse_typed_call(&mut self) {
        if self.is_typed_call() {
            self.parse_call_expression();
        } else {
            self.bump();
        }
    }

    fn parse_new_expression(&mut self) {
        self.builder.start_node(kind::NEW_EXPRESSION);
        self.bump();
        if self.at(kind::IDENTIFIER) && self.next_is(kind::DOT) {
            self.parse_member_expression();
        } else if self.at(kind::IDENTIFIER) {
            self.bump();
        } else {
            self.error_here("expected constructor name");
        }
        if self.at(kind::LESS_THAN) {
            self.parse_balanced_node(
                kind::TYPE_PARAMETERS,
                kind::LESS_THAN,
                kind::GREATER_THAN,
                "expected '>' to close type arguments",
            );
        }
        if self.at(kind::OPEN_PAREN) {
            self.parse_balanced_node(
                kind::PARENTHESIZED_EXPRESSION,
                kind::OPEN_PAREN,
                kind::CLOSE_PAREN,
                "expected ')' to close constructor arguments",
            );
        }
        self.builder.finish_node();
    }

    fn parse_unary_expression(&mut self) {
        self.builder.start_node(kind::UNARY_EXPRESSION);
        self.bump();
        self.parse_expression_operand();
        self.builder.finish_node();
    }

    fn parse_await_expression(&mut self) {
        self.builder.start_node(kind::AWAIT_EXPRESSION);
        self.bump();
        self.parse_expression_operand();
        self.builder.finish_node();
    }

    fn parse_expression_operand(&mut self) {
        match self.current() {
            kind::KEYWORD_NEW => self.parse_new_expression(),
            kind if is_expr_base(kind) && self.next_is(kind::OPEN_PAREN) => {
                self.parse_call_expression();
            }
            kind if is_expr_base(kind)
                && (self.next_is(kind::DOT) || self.next_is(kind::QUESTION_DOT)) =>
            {
                self.parse_member_chain();
            }
            kind::OPEN_PAREN => self.parse_parenthesized_expression(),
            kind::OPEN_BRACKET => self.parse_balanced_node(
                kind::ARRAY_EXPRESSION,
                kind::OPEN_BRACKET,
                kind::CLOSE_BRACKET,
                "expected ']' to close array expression",
            ),
            kind::OPEN_BRACE => self.parse_balanced_node(
                kind::OBJECT_EXPRESSION,
                kind::OPEN_BRACE,
                kind::CLOSE_BRACE,
                "expected '}' to close object expression",
            ),
            kind if kind != kind::END_OF_FILE => self.bump(),
            _ => {}
        }
    }

    fn parse_parenthesized_expression(&mut self) {
        self.builder.start_node(kind::PARENTHESIZED_EXPRESSION);
        if self.expect(kind::OPEN_PAREN, "expected '(' to start expression") {
            self.parse_expression_tokens_until(&[kind::CLOSE_PAREN, kind::END_OF_FILE]);
            self.expect(kind::CLOSE_PAREN, "expected ')' to close expression");
        }
        self.builder.finish_node();
    }

    fn parse_member_expression(&mut self) {
        self.builder.start_node(kind::MEMBER_EXPRESSION);
        self.bump();
        while self.at(kind::DOT) || self.at(kind::QUESTION_DOT) {
            let start = self.cursor;
            self.bump();
            if is_property_name(self.current()) {
                self.bump();
            } else if self.at(kind::OPEN_BRACKET) {
                self.parse_balanced_node(
                    kind::ELEMENT_ACCESS_EXPRESSION,
                    kind::OPEN_BRACKET,
                    kind::CLOSE_BRACKET,
                    "expected ']' to close element access",
                );
            } else if self.at(kind::OPEN_PAREN) {
                self.parse_balanced_node(
                    kind::PARENTHESIZED_EXPRESSION,
                    kind::OPEN_PAREN,
                    kind::CLOSE_PAREN,
                    "expected ')' to close call arguments",
                );
            } else {
                self.error_here("expected property name");
                break;
            }
            self.ensure_progress(start, "member expression");
        }
        self.builder.finish_node();
    }

    fn parse_member_chain(&mut self) {
        let has_call = self.has_member_call();
        if has_call {
            self.builder.start_node(kind::CALL_EXPRESSION);
        }
        self.parse_member_expression();
        if self.at(kind::OPEN_PAREN) {
            self.parse_balanced_node(
                kind::PARENTHESIZED_EXPRESSION,
                kind::OPEN_PAREN,
                kind::CLOSE_PAREN,
                "expected ')' to close call arguments",
            );
        }
        if has_call {
            self.builder.finish_node();
        }
    }

    fn is_typed_call(&self) -> bool {
        let mut cursor = self.cursor + 1;
        let mut depth = 0usize;
        while let Some(token) = self.token_at(cursor) {
            match token.kind {
                kind::LESS_THAN => depth += 1,
                kind::GREATER_THAN => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return self
                            .token_at(cursor + 1)
                            .is_some_and(|next| next.kind == kind::OPEN_PAREN);
                    }
                }
                kind::END_OF_FILE => return false,
                _ => {}
            }
            cursor += 1;
        }
        false
    }

    fn has_member_call(&self) -> bool {
        let mut cursor = self.cursor + 1;
        while let Some(token) = self.token_at(cursor) {
            if !matches!(token.kind, kind::DOT | kind::QUESTION_DOT) {
                return false;
            }
            cursor += 1;
            if self
                .token_at(cursor)
                .is_some_and(|next| next.kind == kind::OPEN_PAREN)
            {
                return true;
            }
            if self
                .token_at(cursor)
                .is_some_and(|next| next.kind == kind::OPEN_BRACKET)
            {
                cursor = self.skip_balanced(cursor, kind::OPEN_BRACKET, kind::CLOSE_BRACKET);
                if self
                    .token_at(cursor)
                    .is_some_and(|next| next.kind == kind::OPEN_PAREN)
                {
                    return true;
                }
                continue;
            }
            if self
                .token_at(cursor)
                .is_none_or(|next| !is_property_name(next.kind))
            {
                return false;
            }
            cursor += 1;
            if self
                .token_at(cursor)
                .is_some_and(|next| next.kind == kind::OPEN_PAREN)
            {
                return true;
            }
        }
        false
    }

    fn skip_balanced(&self, mut cursor: usize, open: Kind, close: Kind) -> usize {
        let mut depth = 0usize;
        while let Some(token) = self.token_at(cursor) {
            if token.kind == open {
                depth += 1;
            } else if token.kind == close {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return cursor + 1;
                }
            } else if token.kind == kind::END_OF_FILE {
                return cursor;
            }
            cursor += 1;
        }
        cursor
    }
}

fn is_unary_operator(kind: Kind) -> bool {
    matches!(
        kind,
        kind::BANG
            | kind::PLUS
            | kind::MINUS
            | kind::PLUS_PLUS
            | kind::MINUS_MINUS
            | kind::KEYWORD_DELETE
            | kind::KEYWORD_TYPEOF
            | kind::KEYWORD_VOID
            | kind::KEYWORD_YIELD
    )
}

fn is_expr_base(kind: Kind) -> bool {
    matches!(
        kind,
        kind::IDENTIFIER | kind::KEYWORD_SUPER | kind::KEYWORD_THIS
    )
}

fn is_property_name(kind: Kind) -> bool {
    matches!(
        kind,
        kind::IDENTIFIER
            | kind::KEYWORD_AS
            | kind::KEYWORD_AWAIT
            | kind::KEYWORD_CLASS
            | kind::KEYWORD_DEFAULT
            | kind::KEYWORD_ENUM
            | kind::KEYWORD_FROM
            | kind::KEYWORD_FUNCTION
            | kind::KEYWORD_GET
            | kind::KEYWORD_IMPORT
            | kind::KEYWORD_INFER
            | kind::KEYWORD_INSTANCEOF
            | kind::KEYWORD_INTERFACE
            | kind::KEYWORD_KEYOF
            | kind::KEYWORD_LET
            | kind::KEYWORD_MODULE
            | kind::KEYWORD_NAMESPACE
            | kind::KEYWORD_NEW
            | kind::KEYWORD_NULL
            | kind::KEYWORD_SATISFIES
            | kind::KEYWORD_SET
            | kind::KEYWORD_STATIC
            | kind::KEYWORD_SUPER
            | kind::KEYWORD_THIS
            | kind::KEYWORD_TRUE
            | kind::KEYWORD_FALSE
            | kind::KEYWORD_TYPE
            | kind::KEYWORD_TYPEOF
            | kind::KEYWORD_UNIQUE
            | kind::KEYWORD_VOID
            | kind::KEYWORD_DELETE
            | kind::KEYWORD_YIELD
    )
}

fn binary_operators() -> &'static [Kind] {
    &[
        kind::PLUS,
        kind::MINUS,
        kind::STAR,
        kind::SLASH,
        kind::EQUALS,
        kind::EQUALS_EQUALS,
        kind::EQUALS_EQUALS_EQUALS,
        kind::BANG_EQUALS,
        kind::BANG_EQUALS_EQUALS,
        kind::LESS_THAN,
        kind::LESS_THAN_EQUALS,
        kind::GREATER_THAN,
        kind::GREATER_THAN_EQUALS,
        kind::PLUS_EQUALS,
        kind::MINUS_EQUALS,
        kind::STAR_EQUALS,
        kind::SLASH_EQUALS,
        kind::PERCENT,
        kind::PERCENT_EQUALS,
        kind::AMPERSAND_AMPERSAND,
        kind::PIPE,
        kind::PIPE_PIPE,
        kind::AMPERSAND,
        kind::QUESTION_QUESTION,
        kind::KEYWORD_IN,
        kind::KEYWORD_INSTANCEOF,
        kind::KEYWORD_OF,
        kind::KEYWORD_AS,
        kind::KEYWORD_SATISFIES,
    ]
}
