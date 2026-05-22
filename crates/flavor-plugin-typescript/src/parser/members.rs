use crate::internal::grammar as kind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_class_member(&mut self) {
        self.builder.start_node(kind::CLASS_MEMBER);
        if self.at(kind::AT) {
            self.parse_decorator_list();
        }
        self.parse_modifier_list();
        let member_kind = if self.is_method_member() {
            kind::METHOD_DECLARATION
        } else {
            kind::PROPERTY_DECLARATION
        };
        self.builder.start_node(member_kind);
        if !self.parse_member_name("expected class member name") {
            self.recover_member();
            self.builder.finish_node();
            self.builder.finish_node();
            return;
        }
        if member_kind == kind::METHOD_DECLARATION {
            self.parse_member_type_params();
            self.parse_parameter_list();
            if self.at(kind::COLON) {
                self.parse_type_annotation(
                    kind::RETURN_TYPE,
                    &[kind::OPEN_BRACE, kind::SEMICOLON, kind::END_OF_FILE],
                );
            }
            if self.at(kind::OPEN_BRACE) {
                self.parse_block(kind::BLOCK);
            } else if self.at(kind::SEMICOLON) {
                self.bump();
            }
        } else {
            self.parse_property_tail();
        }
        self.builder.finish_node();
        self.builder.finish_node();
    }

    pub(super) fn parse_interface_body(&mut self) {
        self.builder.start_node(kind::INTERFACE_BODY);
        if !self.expect(kind::OPEN_BRACE, "expected interface body") {
            self.builder.finish_node();
            return;
        }
        while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
            let start = self.cursor;
            self.parse_interface_member();
            self.ensure_progress(start, "interface body");
        }
        self.expect(kind::CLOSE_BRACE, "expected '}' to close interface body");
        self.builder.finish_node();
    }

    fn parse_interface_member(&mut self) {
        self.parse_modifier_list();
        let member_kind = if self.is_method_member() {
            kind::METHOD_SIGNATURE
        } else {
            kind::PROPERTY_SIGNATURE
        };
        self.builder.start_node(member_kind);
        if !self.parse_member_name("expected interface member name") {
            self.recover_member();
            self.builder.finish_node();
            return;
        }
        if member_kind == kind::METHOD_SIGNATURE {
            self.parse_member_type_params();
            self.parse_parameter_list();
            if self.at(kind::COLON) {
                self.parse_type_annotation(
                    kind::RETURN_TYPE,
                    &[kind::SEMICOLON, kind::COMMA, kind::CLOSE_BRACE],
                );
            }
        } else {
            if self.at(kind::QUESTION) {
                self.bump();
            }
            if self.at(kind::COLON) {
                self.parse_type_annotation(
                    kind::TYPE_ANNOTATION,
                    &[kind::SEMICOLON, kind::COMMA, kind::CLOSE_BRACE],
                );
            }
        }
        if self.at(kind::SEMICOLON) || self.at(kind::COMMA) {
            self.bump();
        }
        self.builder.finish_node();
    }

    fn parse_property_tail(&mut self) {
        if self.at(kind::QUESTION) || self.at(kind::BANG) {
            self.bump();
        }
        if self.at(kind::COLON) {
            self.parse_type_annotation(
                kind::TYPE_ANNOTATION,
                &[kind::EQUALS, kind::SEMICOLON, kind::CLOSE_BRACE],
            );
        }
        if self.at(kind::EQUALS) {
            self.parse_initializer(&[kind::SEMICOLON, kind::CLOSE_BRACE]);
        }
        if self.at(kind::SEMICOLON) {
            self.bump();
        } else if !self.at(kind::CLOSE_BRACE) {
            self.recover_member();
        }
    }

    fn recover_member(&mut self) {
        while !self.at_any(&[
            kind::SEMICOLON,
            kind::COMMA,
            kind::CLOSE_BRACE,
            kind::END_OF_FILE,
        ]) {
            let start = self.cursor;
            self.bump();
            self.ensure_progress(start, "class member recovery");
        }
        if self.at(kind::SEMICOLON) || self.at(kind::COMMA) {
            self.bump();
        }
    }

    fn is_method_member(&self) -> bool {
        self.next_is(kind::OPEN_PAREN)
            || self.current() == kind::KEYWORD_NEW && self.token_kind_at(1) == kind::OPEN_PAREN
            || self.token_kind_at(1) == kind::LESS_THAN
            || matches!(self.current(), kind::KEYWORD_GET | kind::KEYWORD_SET)
                && self.token_kind_at(1) == kind::IDENTIFIER
                && self.token_kind_at(2) == kind::OPEN_PAREN
    }

    fn parse_member_name(&mut self, message: &str) -> bool {
        if matches!(self.current(), kind::KEYWORD_GET | kind::KEYWORD_SET)
            && self.token_kind_at(1) == kind::IDENTIFIER
            && self.token_kind_at(2) == kind::OPEN_PAREN
        {
            self.bump();
            self.bump();
            return true;
        }
        if matches!(
            self.current(),
            kind::IDENTIFIER
                | kind::STRING_LITERAL
                | kind::NUMERIC_LITERAL
                | kind::KEYWORD_AS
                | kind::KEYWORD_AWAIT
                | kind::KEYWORD_CLASS
                | kind::KEYWORD_DEFAULT
                | kind::KEYWORD_DELETE
                | kind::KEYWORD_ENUM
                | kind::KEYWORD_FROM
                | kind::KEYWORD_FUNCTION
                | kind::KEYWORD_GET
                | kind::KEYWORD_IMPORT
                | kind::KEYWORD_INSTANCEOF
                | kind::KEYWORD_INTERFACE
                | kind::KEYWORD_SET
                | kind::KEYWORD_LET
                | kind::KEYWORD_MODULE
                | kind::KEYWORD_NAMESPACE
                | kind::KEYWORD_NEW
                | kind::KEYWORD_SATISFIES
                | kind::KEYWORD_KEYOF
                | kind::KEYWORD_INFER
                | kind::KEYWORD_STATIC
                | kind::KEYWORD_TYPE
                | kind::KEYWORD_TYPEOF
                | kind::KEYWORD_UNIQUE
                | kind::KEYWORD_VOID
                | kind::KEYWORD_THIS
                | kind::KEYWORD_SUPER
                | kind::KEYWORD_TRUE
                | kind::KEYWORD_FALSE
                | kind::KEYWORD_NULL
        ) {
            self.bump();
            return true;
        }
        self.error_here(message);
        false
    }

    fn parse_member_type_params(&mut self) {
        if self.at(kind::LESS_THAN) {
            self.parse_balanced_node(
                kind::TYPE_PARAMETERS,
                kind::LESS_THAN,
                kind::GREATER_THAN,
                "expected '>' to close member type parameters",
            );
        }
    }
}
