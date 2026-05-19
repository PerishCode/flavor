use crate::internal::grammar::{self as kind, Kind};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_return_statement(&mut self) {
        self.parse_expression_tail_statement(kind::RETURN_STATEMENT, kind::KEYWORD_RETURN);
    }

    pub(super) fn parse_throw_statement(&mut self) {
        self.parse_expression_tail_statement(kind::THROW_STATEMENT, kind::KEYWORD_THROW);
    }

    pub(super) fn parse_break_statement(&mut self) {
        self.parse_label_tail_statement(kind::BREAK_STATEMENT);
    }

    pub(super) fn parse_continue_statement(&mut self) {
        self.parse_label_tail_statement(kind::CONTINUE_STATEMENT);
    }

    pub(super) fn parse_if_statement(&mut self) {
        self.builder.start_node(kind::IF_STATEMENT);
        self.bump();
        self.parse_parenthesized_condition("expected '(' to start if condition");
        self.parse_statement_or_block();
        if self.at(kind::KEYWORD_ELSE) {
            self.builder.start_node(kind::ELSE_CLAUSE);
            self.bump();
            self.parse_statement_or_block();
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_for_statement(&mut self) {
        self.builder.start_node(kind::FOR_STATEMENT);
        self.bump();
        self.parse_parenthesized_condition("expected '(' to start for header");
        self.parse_statement_or_block();
        self.builder.finish_node();
    }

    pub(super) fn parse_while_statement(&mut self) {
        self.builder.start_node(kind::WHILE_STATEMENT);
        self.bump();
        self.parse_parenthesized_condition("expected '(' to start while condition");
        self.parse_statement_or_block();
        self.builder.finish_node();
    }

    pub(super) fn parse_do_statement(&mut self) {
        self.builder.start_node(kind::DO_STATEMENT);
        self.bump();
        self.parse_statement_or_block();
        if self.at(kind::KEYWORD_WHILE) {
            self.bump();
            self.parse_parenthesized_condition("expected '(' to start do-while condition");
            self.parse_optional_semicolon();
        } else {
            self.error_here("expected while after do statement");
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_switch_statement(&mut self) {
        self.builder.start_node(kind::SWITCH_STATEMENT);
        self.bump();
        self.parse_parenthesized_condition("expected '(' to start switch condition");
        self.parse_switch_body();
        self.builder.finish_node();
    }

    pub(super) fn parse_try_statement(&mut self) {
        self.builder.start_node(kind::TRY_STATEMENT);
        self.bump();
        self.parse_block(kind::BLOCK);
        let mut has_handler = false;
        if self.at(kind::KEYWORD_CATCH) {
            has_handler = true;
            self.parse_catch_clause();
        }
        if self.at(kind::KEYWORD_FINALLY) {
            has_handler = true;
            self.parse_finally_clause();
        }
        if !has_handler {
            self.error_here("expected catch or finally after try block");
        }
        self.builder.finish_node();
    }

    fn parse_expression_tail_statement(&mut self, kind: Kind, keyword: Kind) {
        self.builder.start_node(kind);
        debug_assert!(self.at(keyword));
        self.bump();
        if !self.at_any(&[kind::SEMICOLON, kind::CLOSE_BRACE, kind::END_OF_FILE]) {
            self.parse_expression(&[kind::SEMICOLON, kind::CLOSE_BRACE, kind::END_OF_FILE]);
        }
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn parse_label_tail_statement(&mut self, kind: Kind) {
        self.builder.start_node(kind);
        self.bump();
        self.parse_balanced_tokens_until(&[kind::SEMICOLON, kind::CLOSE_BRACE, kind::END_OF_FILE]);
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn parse_parenthesized_condition(&mut self, message: &str) {
        self.parse_balanced_node(
            kind::PARENTHESIZED_EXPRESSION,
            kind::OPEN_PAREN,
            kind::CLOSE_PAREN,
            message,
        );
    }

    fn parse_statement_or_block(&mut self) {
        if self.at(kind::OPEN_BRACE) {
            self.parse_block(kind::BLOCK);
        } else if self.at(kind::END_OF_FILE) {
            self.error_here("expected statement");
        } else {
            self.parse_statement();
        }
    }

    fn parse_switch_body(&mut self) {
        self.builder.start_node(kind::SWITCH_BODY);
        if self.expect(kind::OPEN_BRACE, "expected '{' to start switch body") {
            while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
                self.parse_switch_case();
            }
            self.expect(kind::CLOSE_BRACE, "expected '}' to close switch body");
        }
        self.builder.finish_node();
    }

    fn parse_switch_case(&mut self) {
        self.builder.start_node(kind::SWITCH_CASE);
        if self.at(kind::KEYWORD_CASE) {
            self.bump();
            self.parse_expression(&[
                kind::COLON,
                kind::CLOSE_BRACE,
                kind::KEYWORD_CASE,
                kind::KEYWORD_DEFAULT,
                kind::END_OF_FILE,
            ]);
        } else if self.at(kind::KEYWORD_DEFAULT) {
            self.bump();
        } else {
            self.error_here("expected case or default in switch body");
            self.bump();
        }
        self.expect(kind::COLON, "expected ':' after switch label");
        while !self.at_any(&[
            kind::KEYWORD_CASE,
            kind::KEYWORD_DEFAULT,
            kind::CLOSE_BRACE,
            kind::END_OF_FILE,
        ]) {
            self.parse_statement();
        }
        self.builder.finish_node();
    }

    fn parse_catch_clause(&mut self) {
        self.builder.start_node(kind::CATCH_CLAUSE);
        self.bump();
        if self.at(kind::OPEN_PAREN) {
            self.parse_catch_binding();
        }
        self.parse_block(kind::BLOCK);
        self.builder.finish_node();
    }

    fn parse_catch_binding(&mut self) {
        self.builder.start_node(kind::CATCH_BINDING);
        self.bump();
        if !self.at_any(&[kind::CLOSE_PAREN, kind::END_OF_FILE]) {
            self.parse_binding_name("expected catch binding");
            if self.at(kind::COLON) {
                self.parse_type_annotation(
                    kind::TYPE_ANNOTATION,
                    &[kind::CLOSE_PAREN, kind::END_OF_FILE],
                );
            }
        }
        self.expect(kind::CLOSE_PAREN, "expected ')' to close catch binding");
        self.builder.finish_node();
    }

    fn parse_finally_clause(&mut self) {
        self.builder.start_node(kind::FINALLY_CLAUSE);
        self.bump();
        self.parse_block(kind::BLOCK);
        self.builder.finish_node();
    }
}
