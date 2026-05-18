use crate::syntax_kind::TsSyntaxKind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_block(&mut self, kind: TsSyntaxKind) {
        self.builder.start_schema_node(kind);
        if !self.expect(TsSyntaxKind::OpenBrace, "expected '{' to start block") {
            self.builder.finish_node();
            return;
        }
        while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
            self.parse_statement();
        }
        self.expect(TsSyntaxKind::CloseBrace, "expected '}' to close block");
        self.builder.finish_node();
    }

    pub(super) fn parse_class_body(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::ClassBody);
        if !self.expect(TsSyntaxKind::OpenBrace, "expected '{' to start class body") {
            self.builder.finish_node();
            return;
        }
        while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
            self.parse_class_member();
        }
        self.expect(TsSyntaxKind::CloseBrace, "expected '}' to close class body");
        self.builder.finish_node();
    }

    pub(super) fn parse_type_annotation(&mut self, kind: TsSyntaxKind, stops: &[TsSyntaxKind]) {
        self.builder.start_schema_node(kind);
        self.bump();
        self.parse_type(stops);
        self.builder.finish_node();
    }

    pub(super) fn parse_initializer(&mut self, stops: &[TsSyntaxKind]) {
        self.builder.start_schema_node(TsSyntaxKind::Initializer);
        self.bump();
        self.parse_expression(stops);
        self.builder.finish_node();
    }

    pub(super) fn parse_optional_semicolon(&mut self) {
        if self.at(TsSyntaxKind::Semicolon) {
            self.bump();
        }
    }

    pub(super) fn parse_expression_statement(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::ExpressionStatement);
        let start = self.cursor;
        self.parse_expression(&[
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::CloseBrace,
            TsSyntaxKind::EndOfFile,
        ]);
        if self.cursor == start && !self.at(TsSyntaxKind::EndOfFile) {
            self.error_here("expected expression");
            self.bump();
        }
        if self.at(TsSyntaxKind::Semicolon) {
            self.bump();
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_unknown_statement(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::UnknownStatement);
        if self.at(TsSyntaxKind::At) {
            self.parse_decorator_list();
        }
        self.parse_balanced_tokens_until(&[
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::CloseBrace,
            TsSyntaxKind::EndOfFile,
        ]);
        if self.at(TsSyntaxKind::Semicolon) {
            self.bump();
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_balanced_node(
        &mut self,
        kind: TsSyntaxKind,
        open: TsSyntaxKind,
        close: TsSyntaxKind,
        message: &str,
    ) {
        self.builder.start_schema_node(kind);
        if self.expect(open, message) {
            let mut depth = 1usize;
            while depth > 0 && !self.at(TsSyntaxKind::EndOfFile) {
                if self.at(open) {
                    depth += 1;
                    self.bump();
                } else if self.at(close) {
                    depth -= 1;
                    self.bump();
                } else {
                    self.bump();
                }
            }
            if depth > 0 {
                self.error_at(self.current_span(), message);
            }
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_balanced_tokens_until(&mut self, stops: &[TsSyntaxKind]) {
        let mut paren_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut angle_depth = 0usize;
        while !self.at(TsSyntaxKind::EndOfFile) {
            if paren_depth == 0
                && brace_depth == 0
                && bracket_depth == 0
                && angle_depth == 0
                && self.at_any(stops)
            {
                break;
            }
            match self.current() {
                TsSyntaxKind::OpenParen => paren_depth += 1,
                TsSyntaxKind::CloseParen if paren_depth > 0 => paren_depth -= 1,
                TsSyntaxKind::OpenBrace => brace_depth += 1,
                TsSyntaxKind::CloseBrace if brace_depth > 0 => brace_depth -= 1,
                TsSyntaxKind::OpenBracket => bracket_depth += 1,
                TsSyntaxKind::CloseBracket if bracket_depth > 0 => bracket_depth -= 1,
                TsSyntaxKind::LessThan => angle_depth += 1,
                TsSyntaxKind::GreaterThan if angle_depth > 0 => angle_depth -= 1,
                _ => {}
            }
            self.bump();
        }
    }
}
