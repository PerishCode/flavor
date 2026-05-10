use crate::syntax_kind::TsSyntaxKind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_return_statement(&mut self) {
        self.parse_expression_tail_statement(
            TsSyntaxKind::ReturnStatement,
            TsSyntaxKind::KeywordReturn,
        );
    }

    pub(super) fn parse_throw_statement(&mut self) {
        self.parse_expression_tail_statement(
            TsSyntaxKind::ThrowStatement,
            TsSyntaxKind::KeywordThrow,
        );
    }

    pub(super) fn parse_break_statement(&mut self) {
        self.parse_label_tail_statement(TsSyntaxKind::BreakStatement);
    }

    pub(super) fn parse_continue_statement(&mut self) {
        self.parse_label_tail_statement(TsSyntaxKind::ContinueStatement);
    }

    pub(super) fn parse_if_statement(&mut self) {
        self.builder.start_node(TsSyntaxKind::IfStatement);
        self.bump();
        self.parse_parenthesized_condition("expected '(' to start if condition");
        self.parse_statement_or_block();
        if self.at(TsSyntaxKind::KeywordElse) {
            self.builder.start_node(TsSyntaxKind::ElseClause);
            self.bump();
            self.parse_statement_or_block();
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_for_statement(&mut self) {
        self.builder.start_node(TsSyntaxKind::ForStatement);
        self.bump();
        self.parse_parenthesized_condition("expected '(' to start for header");
        self.parse_statement_or_block();
        self.builder.finish_node();
    }

    pub(super) fn parse_while_statement(&mut self) {
        self.builder.start_node(TsSyntaxKind::WhileStatement);
        self.bump();
        self.parse_parenthesized_condition("expected '(' to start while condition");
        self.parse_statement_or_block();
        self.builder.finish_node();
    }

    pub(super) fn parse_do_statement(&mut self) {
        self.builder.start_node(TsSyntaxKind::DoStatement);
        self.bump();
        self.parse_statement_or_block();
        if self.at(TsSyntaxKind::KeywordWhile) {
            self.bump();
            self.parse_parenthesized_condition("expected '(' to start do-while condition");
            self.parse_optional_semicolon();
        } else {
            self.error_here("expected while after do statement");
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_switch_statement(&mut self) {
        self.builder.start_node(TsSyntaxKind::SwitchStatement);
        self.bump();
        self.parse_parenthesized_condition("expected '(' to start switch condition");
        self.parse_switch_body();
        self.builder.finish_node();
    }

    pub(super) fn parse_try_statement(&mut self) {
        self.builder.start_node(TsSyntaxKind::TryStatement);
        self.bump();
        self.parse_block(TsSyntaxKind::Block);
        let mut has_handler = false;
        if self.at(TsSyntaxKind::KeywordCatch) {
            has_handler = true;
            self.parse_catch_clause();
        }
        if self.at(TsSyntaxKind::KeywordFinally) {
            has_handler = true;
            self.parse_finally_clause();
        }
        if !has_handler {
            self.error_here("expected catch or finally after try block");
        }
        self.builder.finish_node();
    }

    fn parse_expression_tail_statement(&mut self, kind: TsSyntaxKind, keyword: TsSyntaxKind) {
        self.builder.start_node(kind);
        debug_assert!(self.at(keyword));
        self.bump();
        if !self.at_any(&[
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::CloseBrace,
            TsSyntaxKind::EndOfFile,
        ]) {
            self.parse_expression(&[
                TsSyntaxKind::Semicolon,
                TsSyntaxKind::CloseBrace,
                TsSyntaxKind::EndOfFile,
            ]);
        }
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn parse_label_tail_statement(&mut self, kind: TsSyntaxKind) {
        self.builder.start_node(kind);
        self.bump();
        self.parse_balanced_tokens_until(&[
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::CloseBrace,
            TsSyntaxKind::EndOfFile,
        ]);
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn parse_parenthesized_condition(&mut self, message: &str) {
        self.parse_balanced_node(
            TsSyntaxKind::ParenthesizedExpression,
            TsSyntaxKind::OpenParen,
            TsSyntaxKind::CloseParen,
            message,
        );
    }

    fn parse_statement_or_block(&mut self) {
        if self.at(TsSyntaxKind::OpenBrace) {
            self.parse_block(TsSyntaxKind::Block);
        } else if self.at(TsSyntaxKind::EndOfFile) {
            self.error_here("expected statement");
        } else {
            self.parse_statement();
        }
    }

    fn parse_switch_body(&mut self) {
        self.builder.start_node(TsSyntaxKind::SwitchBody);
        if self.expect(TsSyntaxKind::OpenBrace, "expected '{' to start switch body") {
            while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
                self.parse_switch_case();
            }
            self.expect(
                TsSyntaxKind::CloseBrace,
                "expected '}' to close switch body",
            );
        }
        self.builder.finish_node();
    }

    fn parse_switch_case(&mut self) {
        self.builder.start_node(TsSyntaxKind::SwitchCase);
        if self.at(TsSyntaxKind::KeywordCase) {
            self.bump();
            self.parse_expression(&[
                TsSyntaxKind::Colon,
                TsSyntaxKind::CloseBrace,
                TsSyntaxKind::KeywordCase,
                TsSyntaxKind::KeywordDefault,
                TsSyntaxKind::EndOfFile,
            ]);
        } else if self.at(TsSyntaxKind::KeywordDefault) {
            self.bump();
        } else {
            self.error_here("expected case or default in switch body");
            self.bump();
        }
        self.expect(TsSyntaxKind::Colon, "expected ':' after switch label");
        while !self.at_any(&[
            TsSyntaxKind::KeywordCase,
            TsSyntaxKind::KeywordDefault,
            TsSyntaxKind::CloseBrace,
            TsSyntaxKind::EndOfFile,
        ]) {
            self.parse_statement();
        }
        self.builder.finish_node();
    }

    fn parse_catch_clause(&mut self) {
        self.builder.start_node(TsSyntaxKind::CatchClause);
        self.bump();
        if self.at(TsSyntaxKind::OpenParen) {
            self.parse_catch_binding();
        }
        self.parse_block(TsSyntaxKind::Block);
        self.builder.finish_node();
    }

    fn parse_catch_binding(&mut self) {
        self.builder.start_node(TsSyntaxKind::CatchBinding);
        self.bump();
        if !self.at_any(&[TsSyntaxKind::CloseParen, TsSyntaxKind::EndOfFile]) {
            self.parse_binding_name("expected catch binding");
            if self.at(TsSyntaxKind::Colon) {
                self.parse_type_annotation(
                    TsSyntaxKind::TypeAnnotation,
                    &[TsSyntaxKind::CloseParen, TsSyntaxKind::EndOfFile],
                );
            }
        }
        self.expect(
            TsSyntaxKind::CloseParen,
            "expected ')' to close catch binding",
        );
        self.builder.finish_node();
    }

    fn parse_finally_clause(&mut self) {
        self.builder.start_node(TsSyntaxKind::FinallyClause);
        self.bump();
        self.parse_block(TsSyntaxKind::Block);
        self.builder.finish_node();
    }
}
