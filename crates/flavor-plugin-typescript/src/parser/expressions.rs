use crate::syntax_kind::TsSyntaxKind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_expression(&mut self, stops: &[TsSyntaxKind]) {
        self.builder.start_schema_node(TsSyntaxKind::Expression);
        let shape = self.expression_shape(stops);
        if let Some(shape) = shape {
            self.builder.start_schema_node(shape);
            self.parse_expression_tokens_until(stops);
            self.builder.finish_node();
        } else {
            self.parse_expression_tokens_until(stops);
        }
        self.builder.finish_node();
    }

    fn expression_shape(&self, stops: &[TsSyntaxKind]) -> Option<TsSyntaxKind> {
        if self.has_top_level_any(&[TsSyntaxKind::Arrow], stops) {
            Some(TsSyntaxKind::ArrowFunction)
        } else if self.has_top_level_any(&[TsSyntaxKind::Question], stops) {
            Some(TsSyntaxKind::ConditionalExpression)
        } else if self.has_top_level_any(binary_operators(), stops) {
            Some(TsSyntaxKind::BinaryExpression)
        } else {
            None
        }
    }

    fn parse_expression_tokens_until(&mut self, stops: &[TsSyntaxKind]) {
        while !self.at(TsSyntaxKind::EndOfFile) && !self.at_any(stops) {
            match self.current() {
                TsSyntaxKind::LessThan if self.jsx_enabled() && self.starts_jsx_open() => {
                    self.parse_jsx_element();
                }
                TsSyntaxKind::KeywordAwait => self.parse_await_expression(),
                TsSyntaxKind::KeywordNew => self.parse_new_expression(),
                kind if is_expr_base(kind) && self.next_is(TsSyntaxKind::OpenParen) => {
                    self.parse_call_expression();
                }
                TsSyntaxKind::Identifier if self.next_is(TsSyntaxKind::LessThan) => {
                    self.parse_typed_call();
                }
                kind if is_expr_base(kind)
                    && (self.next_is(TsSyntaxKind::Dot)
                        || self.next_is(TsSyntaxKind::QuestionDot)) =>
                {
                    self.parse_member_chain();
                }
                kind if is_unary_operator(kind) => self.parse_unary_expression(),
                TsSyntaxKind::OpenBrace => self.parse_balanced_node(
                    TsSyntaxKind::ObjectExpression,
                    TsSyntaxKind::OpenBrace,
                    TsSyntaxKind::CloseBrace,
                    "expected '}' to close object expression",
                ),
                TsSyntaxKind::OpenBracket => self.parse_balanced_node(
                    TsSyntaxKind::ArrayExpression,
                    TsSyntaxKind::OpenBracket,
                    TsSyntaxKind::CloseBracket,
                    "expected ']' to close array expression",
                ),
                TsSyntaxKind::OpenParen => self.parse_parenthesized_expression(),
                _ => self.bump(),
            }
        }
    }

    fn parse_call_expression(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::CallExpression);
        self.bump();
        if self.at(TsSyntaxKind::LessThan) {
            self.parse_balanced_node(
                TsSyntaxKind::TypeParameters,
                TsSyntaxKind::LessThan,
                TsSyntaxKind::GreaterThan,
                "expected '>' to close type arguments",
            );
        }
        self.parse_balanced_node(
            TsSyntaxKind::ParenthesizedExpression,
            TsSyntaxKind::OpenParen,
            TsSyntaxKind::CloseParen,
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
        self.builder.start_schema_node(TsSyntaxKind::NewExpression);
        self.bump();
        if self.at(TsSyntaxKind::Identifier) && self.next_is(TsSyntaxKind::Dot) {
            self.parse_member_expression();
        } else if self.at(TsSyntaxKind::Identifier) {
            self.bump();
        } else {
            self.error_here("expected constructor name");
        }
        if self.at(TsSyntaxKind::LessThan) {
            self.parse_balanced_node(
                TsSyntaxKind::TypeParameters,
                TsSyntaxKind::LessThan,
                TsSyntaxKind::GreaterThan,
                "expected '>' to close type arguments",
            );
        }
        if self.at(TsSyntaxKind::OpenParen) {
            self.parse_balanced_node(
                TsSyntaxKind::ParenthesizedExpression,
                TsSyntaxKind::OpenParen,
                TsSyntaxKind::CloseParen,
                "expected ')' to close constructor arguments",
            );
        }
        self.builder.finish_node();
    }

    fn parse_unary_expression(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::UnaryExpression);
        self.bump();
        self.parse_expression_operand();
        self.builder.finish_node();
    }

    fn parse_await_expression(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::AwaitExpression);
        self.bump();
        self.parse_expression_operand();
        self.builder.finish_node();
    }

    fn parse_expression_operand(&mut self) {
        match self.current() {
            TsSyntaxKind::KeywordNew => self.parse_new_expression(),
            kind if is_expr_base(kind) && self.next_is(TsSyntaxKind::OpenParen) => {
                self.parse_call_expression();
            }
            kind if is_expr_base(kind)
                && (self.next_is(TsSyntaxKind::Dot) || self.next_is(TsSyntaxKind::QuestionDot)) =>
            {
                self.parse_member_chain();
            }
            TsSyntaxKind::OpenParen => self.parse_parenthesized_expression(),
            TsSyntaxKind::OpenBracket => self.parse_balanced_node(
                TsSyntaxKind::ArrayExpression,
                TsSyntaxKind::OpenBracket,
                TsSyntaxKind::CloseBracket,
                "expected ']' to close array expression",
            ),
            TsSyntaxKind::OpenBrace => self.parse_balanced_node(
                TsSyntaxKind::ObjectExpression,
                TsSyntaxKind::OpenBrace,
                TsSyntaxKind::CloseBrace,
                "expected '}' to close object expression",
            ),
            kind if kind != TsSyntaxKind::EndOfFile => self.bump(),
            _ => {}
        }
    }

    fn parse_parenthesized_expression(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::ParenthesizedExpression);
        if self.expect(TsSyntaxKind::OpenParen, "expected '(' to start expression") {
            self.parse_expression_tokens_until(&[
                TsSyntaxKind::CloseParen,
                TsSyntaxKind::EndOfFile,
            ]);
            self.expect(TsSyntaxKind::CloseParen, "expected ')' to close expression");
        }
        self.builder.finish_node();
    }

    fn parse_member_expression(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::MemberExpression);
        self.bump();
        while self.at(TsSyntaxKind::Dot) || self.at(TsSyntaxKind::QuestionDot) {
            self.bump();
            if is_property_name(self.current()) {
                self.bump();
            } else if self.at(TsSyntaxKind::OpenBracket) {
                self.parse_balanced_node(
                    TsSyntaxKind::ElementAccessExpression,
                    TsSyntaxKind::OpenBracket,
                    TsSyntaxKind::CloseBracket,
                    "expected ']' to close element access",
                );
            } else if self.at(TsSyntaxKind::OpenParen) {
                self.parse_balanced_node(
                    TsSyntaxKind::ParenthesizedExpression,
                    TsSyntaxKind::OpenParen,
                    TsSyntaxKind::CloseParen,
                    "expected ')' to close call arguments",
                );
            } else {
                self.error_here("expected property name");
                break;
            }
        }
        self.builder.finish_node();
    }

    fn parse_member_chain(&mut self) {
        let has_call = self.has_member_call();
        if has_call {
            self.builder.start_schema_node(TsSyntaxKind::CallExpression);
        }
        self.parse_member_expression();
        if self.at(TsSyntaxKind::OpenParen) {
            self.parse_balanced_node(
                TsSyntaxKind::ParenthesizedExpression,
                TsSyntaxKind::OpenParen,
                TsSyntaxKind::CloseParen,
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
                TsSyntaxKind::LessThan => depth += 1,
                TsSyntaxKind::GreaterThan => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return self
                            .token_at(cursor + 1)
                            .is_some_and(|next| next.kind == TsSyntaxKind::OpenParen);
                    }
                }
                TsSyntaxKind::EndOfFile => return false,
                _ => {}
            }
            cursor += 1;
        }
        false
    }

    fn has_member_call(&self) -> bool {
        let mut cursor = self.cursor + 1;
        while let Some(token) = self.token_at(cursor) {
            if !matches!(token.kind, TsSyntaxKind::Dot | TsSyntaxKind::QuestionDot) {
                return false;
            }
            cursor += 1;
            if self
                .token_at(cursor)
                .is_some_and(|next| next.kind == TsSyntaxKind::OpenParen)
            {
                return true;
            }
            if self
                .token_at(cursor)
                .is_some_and(|next| next.kind == TsSyntaxKind::OpenBracket)
            {
                cursor = self.skip_balanced(
                    cursor,
                    TsSyntaxKind::OpenBracket,
                    TsSyntaxKind::CloseBracket,
                );
                if self
                    .token_at(cursor)
                    .is_some_and(|next| next.kind == TsSyntaxKind::OpenParen)
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
                .is_some_and(|next| next.kind == TsSyntaxKind::OpenParen)
            {
                return true;
            }
        }
        false
    }

    fn skip_balanced(&self, mut cursor: usize, open: TsSyntaxKind, close: TsSyntaxKind) -> usize {
        let mut depth = 0usize;
        while let Some(token) = self.token_at(cursor) {
            if token.kind == open {
                depth += 1;
            } else if token.kind == close {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return cursor + 1;
                }
            } else if token.kind == TsSyntaxKind::EndOfFile {
                return cursor;
            }
            cursor += 1;
        }
        cursor
    }
}

fn is_unary_operator(kind: TsSyntaxKind) -> bool {
    matches!(
        kind,
        TsSyntaxKind::Bang
            | TsSyntaxKind::Plus
            | TsSyntaxKind::Minus
            | TsSyntaxKind::PlusPlus
            | TsSyntaxKind::MinusMinus
            | TsSyntaxKind::KeywordDelete
            | TsSyntaxKind::KeywordTypeof
            | TsSyntaxKind::KeywordVoid
            | TsSyntaxKind::KeywordYield
    )
}

fn is_expr_base(kind: TsSyntaxKind) -> bool {
    matches!(
        kind,
        TsSyntaxKind::Identifier | TsSyntaxKind::KeywordSuper | TsSyntaxKind::KeywordThis
    )
}

fn is_property_name(kind: TsSyntaxKind) -> bool {
    matches!(
        kind,
        TsSyntaxKind::Identifier
            | TsSyntaxKind::KeywordAs
            | TsSyntaxKind::KeywordAwait
            | TsSyntaxKind::KeywordClass
            | TsSyntaxKind::KeywordDefault
            | TsSyntaxKind::KeywordEnum
            | TsSyntaxKind::KeywordFrom
            | TsSyntaxKind::KeywordFunction
            | TsSyntaxKind::KeywordGet
            | TsSyntaxKind::KeywordImport
            | TsSyntaxKind::KeywordInfer
            | TsSyntaxKind::KeywordInstanceof
            | TsSyntaxKind::KeywordInterface
            | TsSyntaxKind::KeywordKeyof
            | TsSyntaxKind::KeywordLet
            | TsSyntaxKind::KeywordModule
            | TsSyntaxKind::KeywordNamespace
            | TsSyntaxKind::KeywordNew
            | TsSyntaxKind::KeywordNull
            | TsSyntaxKind::KeywordSatisfies
            | TsSyntaxKind::KeywordSet
            | TsSyntaxKind::KeywordStatic
            | TsSyntaxKind::KeywordSuper
            | TsSyntaxKind::KeywordThis
            | TsSyntaxKind::KeywordTrue
            | TsSyntaxKind::KeywordFalse
            | TsSyntaxKind::KeywordType
            | TsSyntaxKind::KeywordTypeof
            | TsSyntaxKind::KeywordUnique
            | TsSyntaxKind::KeywordVoid
            | TsSyntaxKind::KeywordDelete
            | TsSyntaxKind::KeywordYield
    )
}

fn binary_operators() -> &'static [TsSyntaxKind] {
    &[
        TsSyntaxKind::Plus,
        TsSyntaxKind::Minus,
        TsSyntaxKind::Star,
        TsSyntaxKind::Slash,
        TsSyntaxKind::Equals,
        TsSyntaxKind::EqualsEquals,
        TsSyntaxKind::EqualsEqualsEquals,
        TsSyntaxKind::BangEquals,
        TsSyntaxKind::BangEqualsEquals,
        TsSyntaxKind::LessThan,
        TsSyntaxKind::LessThanEquals,
        TsSyntaxKind::GreaterThan,
        TsSyntaxKind::GreaterThanEquals,
        TsSyntaxKind::PlusEquals,
        TsSyntaxKind::MinusEquals,
        TsSyntaxKind::StarEquals,
        TsSyntaxKind::SlashEquals,
        TsSyntaxKind::Percent,
        TsSyntaxKind::PercentEquals,
        TsSyntaxKind::AmpersandAmpersand,
        TsSyntaxKind::Pipe,
        TsSyntaxKind::PipePipe,
        TsSyntaxKind::Ampersand,
        TsSyntaxKind::QuestionQuestion,
        TsSyntaxKind::KeywordIn,
        TsSyntaxKind::KeywordInstanceof,
        TsSyntaxKind::KeywordOf,
        TsSyntaxKind::KeywordAs,
        TsSyntaxKind::KeywordSatisfies,
    ]
}
