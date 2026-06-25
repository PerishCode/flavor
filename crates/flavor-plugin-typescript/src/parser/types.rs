use crate::internal::grammar::{self as kind, Kind};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_type(&mut self, stops: &[Kind]) {
        let kind = if self.has_top_level_any(&[kind::QUESTION], stops) {
            Some(kind::CONDITIONAL_TYPE)
        } else if self.has_top_level_any(&[kind::PIPE], stops) {
            Some(kind::UNION_TYPE)
        } else if self.has_top_level_any(&[kind::AMPERSAND], stops) {
            Some(kind::INTERSECTION_TYPE)
        } else {
            None
        };

        if let Some(kind) = kind {
            self.builder.start_node(kind);
        }
        loop {
            let start = self.cursor;
            let mut segment_stops = stops.to_vec();
            segment_stops.push(kind::PIPE);
            segment_stops.push(kind::AMPERSAND);
            if self.at(kind::OPEN_BRACE) && stops.contains(&kind::OPEN_BRACE) {
                if self.is_mapped_type_start() {
                    self.parse_mapped_type();
                } else {
                    self.parse_object_type();
                }
            } else if self.at_any(stops) {
                break;
            } else {
                self.parse_type_segment_until(&segment_stops);
            }
            if self.at(kind::PIPE) || self.at(kind::AMPERSAND) {
                self.bump();
                self.ensure_progress(start, "type");
                continue;
            }
            self.ensure_progress(start, "type");
            break;
        }
        if kind.is_some() {
            self.builder.finish_node();
        }
    }

    fn parse_type_segment_until(&mut self, stops: &[Kind]) {
        while !self.at(kind::END_OF_FILE) && !self.at_any(stops) {
            let start = self.cursor;
            match self.current() {
                kind if is_type_operator(kind) => self.parse_type_operator(stops),
                kind::IDENTIFIER if self.is_indexed_type_start() => self.parse_indexed_type(),
                kind::IDENTIFIER if self.is_array_type_start() => self.parse_array_type(),
                kind::IDENTIFIER => self.parse_type_reference(),
                kind::OPEN_BRACE if self.is_mapped_type_start() => self.parse_mapped_type(),
                kind::OPEN_BRACE => self.parse_object_type(),
                kind::OPEN_BRACKET => self.parse_balanced_node(
                    kind::TUPLE_TYPE,
                    kind::OPEN_BRACKET,
                    kind::CLOSE_BRACKET,
                    "expected ']' to close tuple type",
                ),
                kind::OPEN_PAREN => self.parse_balanced_node(
                    kind::PARENTHESIZED_TYPE,
                    kind::OPEN_PAREN,
                    kind::CLOSE_PAREN,
                    "expected ')' to close type",
                ),
                _ => self.bump(),
            }
            self.ensure_progress(start, "type segment");
        }
    }

    fn parse_type_reference(&mut self) {
        self.builder.start_node(kind::TYPE_REFERENCE);
        self.bump();
        if self.at(kind::LESS_THAN) {
            self.parse_balanced_node(
                kind::TYPE_PARAMETERS,
                kind::LESS_THAN,
                kind::GREATER_THAN,
                "expected '>' to close type arguments",
            );
        }
        self.builder.finish_node();
    }

    fn parse_type_operator(&mut self, stops: &[Kind]) {
        self.builder.start_node(kind::TYPE_OPERATOR);
        self.bump();
        self.parse_type_segment_until(stops);
        self.builder.finish_node();
    }

    fn parse_indexed_type(&mut self) {
        self.builder.start_node(kind::INDEXED_ACCESS_TYPE);
        self.parse_type_reference();
        while self.at(kind::OPEN_BRACKET) && !self.next_is(kind::CLOSE_BRACKET) {
            let start = self.cursor;
            self.bump();
            self.parse_type(&[kind::CLOSE_BRACKET, kind::END_OF_FILE]);
            self.expect(
                kind::CLOSE_BRACKET,
                "expected ']' to close indexed access type",
            );
            self.ensure_progress(start, "indexed access type");
        }
        self.builder.finish_node();
    }

    fn parse_array_type(&mut self) {
        self.builder.start_node(kind::ARRAY_TYPE);
        self.parse_type_reference();
        while self.at(kind::OPEN_BRACKET) && self.token_kind_at(1) == kind::CLOSE_BRACKET {
            let start = self.cursor;
            self.bump();
            self.bump();
            self.ensure_progress(start, "array type");
        }
        self.builder.finish_node();
    }

    fn is_array_type_start(&self) -> bool {
        self.token_kind_at(1) == kind::OPEN_BRACKET && self.token_kind_at(2) == kind::CLOSE_BRACKET
    }

    fn is_indexed_type_start(&self) -> bool {
        self.token_kind_at(1) == kind::OPEN_BRACKET && self.token_kind_at(2) != kind::CLOSE_BRACKET
    }

    fn is_mapped_type_start(&self) -> bool {
        if !self.at(kind::OPEN_BRACE) {
            return false;
        }
        let Some(open_bracket) = self.mapped_bracket_cursor() else {
            return false;
        };
        let mut cursor = open_bracket + 1;
        while let Some(token) = self.token_at(cursor) {
            match token.kind {
                kind::KEYWORD_IN => return true,
                kind::CLOSE_BRACKET | kind::END_OF_FILE => return false,
                _ => cursor += 1,
            }
        }
        false
    }

    fn mapped_bracket_cursor(&self) -> Option<usize> {
        let mut cursor = self.cursor + 1;
        while let Some(token) = self.token_at(cursor) {
            match token.kind {
                kind::PLUS | kind::MINUS | kind::KEYWORD_READONLY => {
                    cursor += 1;
                }
                kind::OPEN_BRACKET => return Some(cursor),
                _ => return None,
            }
        }
        None
    }

    fn parse_mapped_type(&mut self) {
        self.builder.start_node(kind::MAPPED_TYPE);
        if self.expect(kind::OPEN_BRACE, "expected '{' to start mapped type") {
            while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
                let start = self.cursor;
                self.parse_type_member();
                self.ensure_progress(start, "mapped type");
            }
            self.expect(kind::CLOSE_BRACE, "expected '}' to close mapped type");
        }
        self.builder.finish_node();
    }

    fn parse_object_type(&mut self) {
        self.builder.start_node(kind::OBJECT_TYPE);
        if self.expect(kind::OPEN_BRACE, "expected '{' to start object type") {
            while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
                let start = self.cursor;
                self.parse_type_member();
                self.ensure_progress(start, "object type");
            }
            self.expect(kind::CLOSE_BRACE, "expected '}' to close object type");
        }
        self.builder.finish_node();
    }

    fn parse_type_member(&mut self) {
        self.builder.start_node(kind::TYPE_MEMBER);
        self.parse_modifier_list();
        if self.at(kind::OPEN_PAREN) {
            self.parse_parameter_list();
            self.parse_type_member_return();
        } else if self.at(kind::KEYWORD_NEW) && self.next_is(kind::OPEN_PAREN) {
            self.bump();
            self.parse_parameter_list();
            self.parse_type_member_return();
        } else {
            self.parse_type_member_name();
            if self.at(kind::QUESTION) {
                self.bump();
            }
            if self.at(kind::LESS_THAN) {
                self.parse_balanced_node(
                    kind::TYPE_PARAMETERS,
                    kind::LESS_THAN,
                    kind::GREATER_THAN,
                    "expected '>' to close member type parameters",
                );
            }
            if self.at(kind::OPEN_PAREN) {
                self.parse_parameter_list();
                self.parse_type_member_return();
            } else if self.at(kind::COLON) {
                self.parse_type_annotation(
                    kind::TYPE_ANNOTATION,
                    &[kind::SEMICOLON, kind::COMMA, kind::CLOSE_BRACE],
                );
            }
        }
        self.parse_type_member_separator();
        self.builder.finish_node();
    }

    fn parse_type_member_name(&mut self) {
        if self.at(kind::OPEN_BRACKET) {
            self.bump();
            self.parse_balanced_tokens_until(&[kind::CLOSE_BRACKET, kind::END_OF_FILE]);
            self.expect(
                kind::CLOSE_BRACKET,
                "expected ']' to close type member name",
            );
        } else if matches!(
            self.current(),
            kind::IDENTIFIER
                | kind::STRING_LITERAL
                | kind::NUMERIC_LITERAL
                | kind::KEYWORD_GET
                | kind::KEYWORD_SET
                | kind::KEYWORD_MODULE
                | kind::KEYWORD_NEW
                | kind::KEYWORD_NAMESPACE
                | kind::KEYWORD_SATISFIES
                | kind::KEYWORD_KEYOF
                | kind::KEYWORD_INFER
                | kind::KEYWORD_TYPE
                | kind::KEYWORD_TRUE
                | kind::KEYWORD_FALSE
                | kind::KEYWORD_NULL
                | kind::KEYWORD_UNIQUE
        ) {
            self.bump();
        } else {
            self.error_here("expected type member name");
            if !self.at_any(&[
                kind::SEMICOLON,
                kind::COMMA,
                kind::CLOSE_BRACE,
                kind::END_OF_FILE,
            ]) {
                self.bump();
            }
        }
    }

    fn parse_type_member_return(&mut self) {
        if self.at(kind::COLON) {
            self.parse_type_annotation(
                kind::RETURN_TYPE,
                &[kind::SEMICOLON, kind::COMMA, kind::CLOSE_BRACE],
            );
        }
    }

    fn parse_type_member_separator(&mut self) {
        if self.at(kind::SEMICOLON) || self.at(kind::COMMA) {
            self.bump();
        }
    }
}

fn is_type_operator(kind: Kind) -> bool {
    matches!(
        kind,
        kind::KEYWORD_KEYOF | kind::KEYWORD_INFER | kind::KEYWORD_READONLY | kind::KEYWORD_UNIQUE
    )
}
