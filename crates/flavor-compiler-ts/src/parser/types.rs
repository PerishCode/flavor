use crate::syntax_kind::TsSyntaxKind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_type(&mut self, stops: &[TsSyntaxKind]) {
        let kind = if self.has_top_level_any(&[TsSyntaxKind::Question], stops) {
            Some(TsSyntaxKind::ConditionalType)
        } else if self.has_top_level_any(&[TsSyntaxKind::Pipe], stops) {
            Some(TsSyntaxKind::UnionType)
        } else if self.has_top_level_any(&[TsSyntaxKind::Ampersand], stops) {
            Some(TsSyntaxKind::IntersectionType)
        } else {
            None
        };

        if let Some(kind) = kind {
            self.builder.start_node(kind);
        }
        loop {
            let mut segment_stops = stops.to_vec();
            segment_stops.push(TsSyntaxKind::Pipe);
            segment_stops.push(TsSyntaxKind::Ampersand);
            self.parse_type_segment_until(&segment_stops);
            if self.at(TsSyntaxKind::Pipe) || self.at(TsSyntaxKind::Ampersand) {
                self.bump();
                continue;
            }
            break;
        }
        if kind.is_some() {
            self.builder.finish_node();
        }
    }

    fn parse_type_segment_until(&mut self, stops: &[TsSyntaxKind]) {
        while !self.at(TsSyntaxKind::EndOfFile) && !self.at_any(stops) {
            match self.current() {
                kind if is_type_operator(kind) => self.parse_type_operator(stops),
                TsSyntaxKind::Identifier if self.is_indexed_type_start() => {
                    self.parse_indexed_type()
                }
                TsSyntaxKind::Identifier if self.is_array_type_start() => self.parse_array_type(),
                TsSyntaxKind::Identifier => self.parse_type_reference(),
                TsSyntaxKind::OpenBrace if self.is_mapped_type_start() => self.parse_mapped_type(),
                TsSyntaxKind::OpenBrace => self.parse_object_type(),
                TsSyntaxKind::OpenBracket => self.parse_balanced_node(
                    TsSyntaxKind::TupleType,
                    TsSyntaxKind::OpenBracket,
                    TsSyntaxKind::CloseBracket,
                    "expected ']' to close tuple type",
                ),
                TsSyntaxKind::OpenParen => self.parse_balanced_node(
                    TsSyntaxKind::ParenthesizedType,
                    TsSyntaxKind::OpenParen,
                    TsSyntaxKind::CloseParen,
                    "expected ')' to close type",
                ),
                _ => self.bump(),
            }
        }
    }

    fn parse_type_reference(&mut self) {
        self.builder.start_node(TsSyntaxKind::TypeReference);
        self.bump();
        if self.at(TsSyntaxKind::LessThan) {
            self.parse_balanced_node(
                TsSyntaxKind::TypeParameters,
                TsSyntaxKind::LessThan,
                TsSyntaxKind::GreaterThan,
                "expected '>' to close type arguments",
            );
        }
        self.builder.finish_node();
    }

    fn parse_type_operator(&mut self, stops: &[TsSyntaxKind]) {
        self.builder.start_node(TsSyntaxKind::TypeOperator);
        self.bump();
        self.parse_type_segment_until(stops);
        self.builder.finish_node();
    }

    fn parse_indexed_type(&mut self) {
        self.builder.start_node(TsSyntaxKind::IndexedAccessType);
        self.parse_type_reference();
        while self.at(TsSyntaxKind::OpenBracket) && !self.next_is(TsSyntaxKind::CloseBracket) {
            self.bump();
            self.parse_type(&[TsSyntaxKind::CloseBracket, TsSyntaxKind::EndOfFile]);
            self.expect(
                TsSyntaxKind::CloseBracket,
                "expected ']' to close indexed access type",
            );
        }
        self.builder.finish_node();
    }

    fn parse_array_type(&mut self) {
        self.builder.start_node(TsSyntaxKind::ArrayType);
        self.parse_type_reference();
        while self.at(TsSyntaxKind::OpenBracket)
            && self.token_kind_at(1) == TsSyntaxKind::CloseBracket
        {
            self.bump();
            self.bump();
        }
        self.builder.finish_node();
    }

    fn is_array_type_start(&self) -> bool {
        self.token_kind_at(1) == TsSyntaxKind::OpenBracket
            && self.token_kind_at(2) == TsSyntaxKind::CloseBracket
    }

    fn is_indexed_type_start(&self) -> bool {
        self.token_kind_at(1) == TsSyntaxKind::OpenBracket
            && self.token_kind_at(2) != TsSyntaxKind::CloseBracket
    }

    fn is_mapped_type_start(&self) -> bool {
        if !self.at(TsSyntaxKind::OpenBrace) {
            return false;
        }
        let Some(open_bracket) = self.mapped_bracket_cursor() else {
            return false;
        };
        let mut cursor = open_bracket + 1;
        while let Some(token) = self.token_at(cursor) {
            match token.kind {
                TsSyntaxKind::KeywordIn => return true,
                TsSyntaxKind::CloseBracket | TsSyntaxKind::EndOfFile => return false,
                _ => cursor += 1,
            }
        }
        false
    }

    fn mapped_bracket_cursor(&self) -> Option<usize> {
        let mut cursor = self.cursor + 1;
        while let Some(token) = self.token_at(cursor) {
            match token.kind {
                TsSyntaxKind::Plus | TsSyntaxKind::Minus | TsSyntaxKind::KeywordReadonly => {
                    cursor += 1;
                }
                TsSyntaxKind::OpenBracket => return Some(cursor),
                _ => return None,
            }
        }
        None
    }

    fn parse_mapped_type(&mut self) {
        self.builder.start_node(TsSyntaxKind::MappedType);
        if self.expect(TsSyntaxKind::OpenBrace, "expected '{' to start mapped type") {
            while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
                self.parse_type_member();
            }
            self.expect(
                TsSyntaxKind::CloseBrace,
                "expected '}' to close mapped type",
            );
        }
        self.builder.finish_node();
    }

    fn parse_object_type(&mut self) {
        self.builder.start_node(TsSyntaxKind::ObjectType);
        if self.expect(TsSyntaxKind::OpenBrace, "expected '{' to start object type") {
            while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
                self.parse_type_member();
            }
            self.expect(
                TsSyntaxKind::CloseBrace,
                "expected '}' to close object type",
            );
        }
        self.builder.finish_node();
    }

    fn parse_type_member(&mut self) {
        self.builder.start_node(TsSyntaxKind::TypeMember);
        self.parse_modifier_list();
        if self.at(TsSyntaxKind::OpenParen) {
            self.parse_parameter_list();
            self.parse_type_member_return();
        } else if self.at(TsSyntaxKind::KeywordNew) && self.next_is(TsSyntaxKind::OpenParen) {
            self.bump();
            self.parse_parameter_list();
            self.parse_type_member_return();
        } else {
            self.parse_type_member_name();
            if self.at(TsSyntaxKind::Question) {
                self.bump();
            }
            if self.at(TsSyntaxKind::LessThan) {
                self.parse_balanced_node(
                    TsSyntaxKind::TypeParameters,
                    TsSyntaxKind::LessThan,
                    TsSyntaxKind::GreaterThan,
                    "expected '>' to close member type parameters",
                );
            }
            if self.at(TsSyntaxKind::OpenParen) {
                self.parse_parameter_list();
                self.parse_type_member_return();
            } else if self.at(TsSyntaxKind::Colon) {
                self.parse_type_annotation(
                    TsSyntaxKind::TypeAnnotation,
                    &[
                        TsSyntaxKind::Semicolon,
                        TsSyntaxKind::Comma,
                        TsSyntaxKind::CloseBrace,
                    ],
                );
            }
        }
        self.parse_type_member_separator();
        self.builder.finish_node();
    }

    fn parse_type_member_name(&mut self) {
        if self.at(TsSyntaxKind::OpenBracket) {
            self.bump();
            self.parse_balanced_tokens_until(&[
                TsSyntaxKind::CloseBracket,
                TsSyntaxKind::EndOfFile,
            ]);
            self.expect(
                TsSyntaxKind::CloseBracket,
                "expected ']' to close type member name",
            );
        } else if matches!(
            self.current(),
            TsSyntaxKind::Identifier
                | TsSyntaxKind::StringLiteral
                | TsSyntaxKind::NumericLiteral
                | TsSyntaxKind::KeywordGet
                | TsSyntaxKind::KeywordSet
                | TsSyntaxKind::KeywordNew
                | TsSyntaxKind::KeywordSatisfies
                | TsSyntaxKind::KeywordKeyof
                | TsSyntaxKind::KeywordInfer
                | TsSyntaxKind::KeywordTrue
                | TsSyntaxKind::KeywordFalse
                | TsSyntaxKind::KeywordNull
                | TsSyntaxKind::KeywordUnique
        ) {
            self.bump();
        } else {
            self.error_here("expected type member name");
            if !self.at_any(&[
                TsSyntaxKind::Semicolon,
                TsSyntaxKind::Comma,
                TsSyntaxKind::CloseBrace,
                TsSyntaxKind::EndOfFile,
            ]) {
                self.bump();
            }
        }
    }

    fn parse_type_member_return(&mut self) {
        if self.at(TsSyntaxKind::Colon) {
            self.parse_type_annotation(
                TsSyntaxKind::ReturnType,
                &[
                    TsSyntaxKind::Semicolon,
                    TsSyntaxKind::Comma,
                    TsSyntaxKind::CloseBrace,
                ],
            );
        }
    }

    fn parse_type_member_separator(&mut self) {
        if self.at(TsSyntaxKind::Semicolon) || self.at(TsSyntaxKind::Comma) {
            self.bump();
        }
    }
}

fn is_type_operator(kind: TsSyntaxKind) -> bool {
    matches!(
        kind,
        TsSyntaxKind::KeywordKeyof
            | TsSyntaxKind::KeywordInfer
            | TsSyntaxKind::KeywordReadonly
            | TsSyntaxKind::KeywordUnique
    )
}
