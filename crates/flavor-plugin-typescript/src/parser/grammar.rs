use crate::internal::grammar::{self as kind, Kind};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_block(&mut self, kind: Kind) {
        self.builder.start_node(kind);
        if !self.expect(kind::OPEN_BRACE, "expected '{' to start block") {
            self.builder.finish_node();
            return;
        }
        while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
            let start = self.cursor;
            self.parse_statement();
            self.ensure_progress(start, "block");
        }
        self.expect(kind::CLOSE_BRACE, "expected '}' to close block");
        self.builder.finish_node();
    }

    pub(super) fn parse_class_body(&mut self) {
        self.builder.start_node(kind::CLASS_BODY);
        if !self.expect(kind::OPEN_BRACE, "expected '{' to start class body") {
            self.builder.finish_node();
            return;
        }
        while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
            let start = self.cursor;
            self.parse_class_member();
            self.ensure_progress(start, "class body");
        }
        self.expect(kind::CLOSE_BRACE, "expected '}' to close class body");
        self.builder.finish_node();
    }

    pub(super) fn parse_type_annotation(&mut self, kind: Kind, stops: &[Kind]) {
        self.builder.start_node(kind);
        self.bump();
        self.parse_type(stops);
        self.builder.finish_node();
    }

    pub(super) fn parse_initializer(&mut self, stops: &[Kind]) {
        self.builder.start_node(kind::INITIALIZER);
        self.bump();
        self.parse_expression(stops);
        self.builder.finish_node();
    }

    pub(super) fn parse_optional_semicolon(&mut self) {
        if self.at(kind::SEMICOLON) {
            self.bump();
        }
    }

    pub(super) fn parse_expression_statement(&mut self) {
        self.builder.start_node(kind::EXPRESSION_STATEMENT);
        let start = self.cursor;
        self.parse_expression(&[kind::SEMICOLON, kind::CLOSE_BRACE, kind::END_OF_FILE]);
        if self.cursor == start && !self.at(kind::END_OF_FILE) {
            self.error_here("expected expression");
            self.bump();
        }
        if self.at(kind::SEMICOLON) {
            self.bump();
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_unknown_statement(&mut self) {
        self.builder.start_node(kind::UNKNOWN_STATEMENT);
        if self.at(kind::AT) {
            self.parse_decorator_list();
        }
        self.parse_balanced_tokens_until(&[kind::SEMICOLON, kind::CLOSE_BRACE, kind::END_OF_FILE]);
        if self.at(kind::SEMICOLON) {
            self.bump();
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_balanced_node(
        &mut self,
        kind: Kind,
        open: Kind,
        close: Kind,
        message: &str,
    ) {
        self.builder.start_node(kind);
        if self.expect(open, message) {
            let mut depth = 1usize;
            while depth > 0 && !self.at(kind::END_OF_FILE) {
                let start = self.cursor;
                if self.at(open) {
                    depth += 1;
                    self.bump();
                } else if self.at(close) {
                    depth -= 1;
                    self.bump();
                } else {
                    self.bump();
                }
                self.ensure_progress(start, "balanced node");
            }
            if depth > 0 {
                self.error_at(self.current_span(), message);
            }
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_balanced_tokens_until(&mut self, stops: &[Kind]) {
        let mut paren_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut angle_depth = 0usize;
        while !self.at(kind::END_OF_FILE) {
            let start = self.cursor;
            if paren_depth == 0
                && brace_depth == 0
                && bracket_depth == 0
                && angle_depth == 0
                && self.at_any(stops)
            {
                break;
            }
            match self.current() {
                kind::OPEN_PAREN => paren_depth += 1,
                kind::CLOSE_PAREN if paren_depth > 0 => paren_depth -= 1,
                kind::OPEN_BRACE => brace_depth += 1,
                kind::CLOSE_BRACE if brace_depth > 0 => brace_depth -= 1,
                kind::OPEN_BRACKET => bracket_depth += 1,
                kind::CLOSE_BRACKET if bracket_depth > 0 => bracket_depth -= 1,
                kind::LESS_THAN => angle_depth += 1,
                kind::GREATER_THAN if angle_depth > 0 => angle_depth -= 1,
                _ => {}
            }
            self.bump();
            self.ensure_progress(start, "balanced token range");
        }
    }
}
