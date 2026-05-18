use crate::syntax_kind::TsSyntaxKind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_jsx_element(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::JsxElement);
        let Some(name) = self.parse_jsx_opening() else {
            self.builder.finish_node();
            return;
        };
        if self.was_jsx_self_closing() {
            self.builder.finish_node();
            return;
        }
        while !self.at(TsSyntaxKind::EndOfFile) {
            if self.starts_jsx_close() {
                self.parse_jsx_closing();
                self.builder.finish_node();
                return;
            }
            if self.starts_jsx_open() {
                self.parse_jsx_element();
            } else if self.at(TsSyntaxKind::OpenBrace) {
                self.parse_balanced_node(
                    TsSyntaxKind::JsxExpression,
                    TsSyntaxKind::OpenBrace,
                    TsSyntaxKind::CloseBrace,
                    "expected '}' to close JSX expression",
                );
            } else {
                self.parse_jsx_text();
            }
        }
        self.error_at(
            self.current_span(),
            &format!("expected closing JSX tag for {name}"),
        );
        self.builder.finish_node();
    }

    pub(super) fn starts_jsx_open(&self) -> bool {
        self.at(TsSyntaxKind::LessThan) && is_jsx_name_start(self.token_kind_at(1))
    }

    fn parse_jsx_opening(&mut self) -> Option<String> {
        self.builder
            .start_schema_node(TsSyntaxKind::JsxOpeningElement);
        self.bump();
        let Some(name) = self.parse_jsx_name("expected JSX tag name") else {
            self.builder.finish_node();
            return None;
        };
        while !self.at_any(&[TsSyntaxKind::GreaterThan, TsSyntaxKind::EndOfFile]) {
            if self.at(TsSyntaxKind::Slash) && self.next_is(TsSyntaxKind::GreaterThan) {
                self.bump();
                self.bump();
                self.builder.finish_node();
                return Some(name);
            }
            self.parse_jsx_attribute();
        }
        if self.at(TsSyntaxKind::GreaterThan) {
            self.bump();
        } else {
            self.error_here("expected '>' to close JSX opening tag");
        }
        self.builder.finish_node();
        Some(name)
    }

    fn parse_jsx_closing(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::JsxClosingElement);
        self.bump();
        self.bump();
        self.parse_jsx_name("expected JSX closing tag name");
        if self.at(TsSyntaxKind::GreaterThan) {
            self.bump();
        } else {
            self.error_here("expected '>' to close JSX closing tag");
        }
        self.builder.finish_node();
    }

    fn parse_jsx_attribute(&mut self) {
        if self.at(TsSyntaxKind::OpenBrace) {
            self.builder
                .start_schema_node(TsSyntaxKind::JsxSpreadAttribute);
            self.parse_balanced_node(
                TsSyntaxKind::JsxExpression,
                TsSyntaxKind::OpenBrace,
                TsSyntaxKind::CloseBrace,
                "expected '}' to close JSX spread attribute",
            );
            self.builder.finish_node();
            return;
        }
        if is_jsx_name_start(self.current()) {
            self.builder.start_schema_node(TsSyntaxKind::JsxAttribute);
            self.parse_jsx_name("expected JSX attribute name");
            if self.at(TsSyntaxKind::Equals) {
                self.bump();
                if self.at(TsSyntaxKind::StringLiteral) {
                    self.bump();
                } else if self.at(TsSyntaxKind::OpenBrace) {
                    self.parse_balanced_node(
                        TsSyntaxKind::JsxExpression,
                        TsSyntaxKind::OpenBrace,
                        TsSyntaxKind::CloseBrace,
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
        self.builder.start_schema_node(TsSyntaxKind::JsxText);
        while !self.at(TsSyntaxKind::EndOfFile)
            && !self.at(TsSyntaxKind::OpenBrace)
            && !self.starts_jsx_open()
            && !self.starts_jsx_close()
        {
            self.bump();
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
            self.bump();
            self.bump();
        }
        let end = self.token_kind_at_back(1);
        debug_assert!(end != TsSyntaxKind::EndOfFile);
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
        self.at(TsSyntaxKind::LessThan) && self.next_is(TsSyntaxKind::Slash)
    }

    fn was_jsx_self_closing(&self) -> bool {
        self.token_kind_at_back(1) == TsSyntaxKind::GreaterThan
            && self.token_kind_at_back(2) == TsSyntaxKind::Slash
    }
}

fn is_jsx_name_start(kind: TsSyntaxKind) -> bool {
    kind == TsSyntaxKind::Identifier || is_jsx_keyword(kind)
}

fn is_jsx_keyword(kind: TsSyntaxKind) -> bool {
    matches!(
        kind,
        TsSyntaxKind::KeywordAbstract
            | TsSyntaxKind::KeywordAs
            | TsSyntaxKind::KeywordAsync
            | TsSyntaxKind::KeywordAwait
            | TsSyntaxKind::KeywordClass
            | TsSyntaxKind::KeywordConst
            | TsSyntaxKind::KeywordDefault
            | TsSyntaxKind::KeywordExport
            | TsSyntaxKind::KeywordFor
            | TsSyntaxKind::KeywordFrom
            | TsSyntaxKind::KeywordFunction
            | TsSyntaxKind::KeywordGet
            | TsSyntaxKind::KeywordIf
            | TsSyntaxKind::KeywordImport
            | TsSyntaxKind::KeywordIn
            | TsSyntaxKind::KeywordInterface
            | TsSyntaxKind::KeywordLet
            | TsSyntaxKind::KeywordNew
            | TsSyntaxKind::KeywordOf
            | TsSyntaxKind::KeywordReadonly
            | TsSyntaxKind::KeywordSet
            | TsSyntaxKind::KeywordStatic
            | TsSyntaxKind::KeywordThis
            | TsSyntaxKind::KeywordType
    )
}

fn is_jsx_name_part(kind: TsSyntaxKind) -> bool {
    is_jsx_name_start(kind) || matches!(kind, TsSyntaxKind::NumericLiteral)
}

fn is_jsx_name_join(kind: TsSyntaxKind) -> bool {
    matches!(
        kind,
        TsSyntaxKind::Dot | TsSyntaxKind::Colon | TsSyntaxKind::Minus
    )
}
