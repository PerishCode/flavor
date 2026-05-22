use crate::internal::grammar as kind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_type_alias_declaration(&mut self) {
        self.builder.start_node(kind::TYPE_ALIAS_DECLARATION);
        self.parse_modifier_list();
        self.bump();
        if self.at(kind::IDENTIFIER) {
            self.bump();
        } else {
            self.error_here("expected type alias name");
        }
        if self.at(kind::LESS_THAN) {
            self.parse_balanced_node(
                kind::TYPE_PARAMETERS,
                kind::LESS_THAN,
                kind::GREATER_THAN,
                "expected '>' to close type parameters",
            );
        }
        if self.expect(kind::EQUALS, "expected '=' in type alias") {
            self.parse_type(&[kind::SEMICOLON, kind::END_OF_FILE]);
        }
        if self.at(kind::SEMICOLON) {
            self.bump();
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_interface_declaration(&mut self) {
        self.builder.start_node(kind::INTERFACE_DECLARATION);
        self.parse_modifier_list();
        self.bump();
        if self.at(kind::IDENTIFIER) {
            self.bump();
        } else {
            self.error_here("expected interface name");
        }
        if self.at(kind::LESS_THAN) {
            self.parse_balanced_node(
                kind::TYPE_PARAMETERS,
                kind::LESS_THAN,
                kind::GREATER_THAN,
                "expected '>' to close type parameters",
            );
        }
        if self.at(kind::KEYWORD_EXTENDS) {
            self.parse_heritage_clause();
        }
        self.parse_interface_body();
        self.builder.finish_node();
    }

    pub(super) fn parse_variable_statement(&mut self) {
        self.builder.start_node(kind::VARIABLE_STATEMENT);
        self.parse_modifier_list();
        self.builder.start_node(kind::VARIABLE_DECLARATION_LIST);
        self.bump();

        loop {
            let start = self.cursor;
            if self.at_any(&[kind::SEMICOLON, kind::CLOSE_BRACE, kind::END_OF_FILE]) {
                break;
            }
            self.parse_variable_declaration();
            if self.at(kind::COMMA) {
                self.bump();
                self.ensure_progress(start, "variable declaration list");
                continue;
            }
            self.ensure_progress(start, "variable declaration list");
            break;
        }

        self.builder.finish_node();
        if self.at(kind::SEMICOLON) {
            self.bump();
        }
        self.builder.finish_node();
    }

    fn parse_variable_declaration(&mut self) {
        self.builder.start_node(kind::VARIABLE_DECLARATION);
        self.parse_binding_name("expected variable name");
        if self.at(kind::QUESTION) || self.at(kind::BANG) {
            self.bump();
        }
        if self.at(kind::COLON) {
            self.parse_type_annotation(
                kind::TYPE_ANNOTATION,
                &[
                    kind::EQUALS,
                    kind::COMMA,
                    kind::SEMICOLON,
                    kind::CLOSE_PAREN,
                    kind::CLOSE_BRACE,
                ],
            );
        }
        if self.at(kind::EQUALS) {
            self.parse_initializer(&[kind::COMMA, kind::SEMICOLON, kind::CLOSE_BRACE]);
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_function_declaration(&mut self, has_decorators: bool) {
        self.builder.start_node(kind::FUNCTION_DECLARATION);
        if has_decorators {
            self.parse_decorator_list();
        }
        self.parse_modifier_list();
        self.bump();
        if self.at(kind::IDENTIFIER) {
            self.bump();
        } else {
            self.error_here("expected function name");
        }
        if self.at(kind::LESS_THAN) {
            self.parse_balanced_node(
                kind::TYPE_PARAMETERS,
                kind::LESS_THAN,
                kind::GREATER_THAN,
                "expected '>' to close type parameters",
            );
        }
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
        } else {
            self.error_here("expected function body");
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_parameter_list(&mut self) {
        self.builder.start_node(kind::PARAMETER_LIST);
        if !self.expect(kind::OPEN_PAREN, "expected '(' to start parameter list") {
            self.builder.finish_node();
            return;
        }
        while !self.at_any(&[kind::CLOSE_PAREN, kind::END_OF_FILE]) {
            let start = self.cursor;
            self.parse_parameter();
            if self.at(kind::COMMA) {
                self.bump();
            } else if !self.at(kind::CLOSE_PAREN) {
                self.error_here("expected ',' or ')' in parameter list");
                break;
            }
            self.ensure_progress(start, "parameter list");
        }
        self.expect(kind::CLOSE_PAREN, "expected ')' to close parameter list");
        self.builder.finish_node();
    }

    fn parse_parameter(&mut self) {
        self.builder.start_node(kind::PARAMETER);
        if self.at(kind::DOT_DOT_DOT) {
            self.parse_rest_element("expected parameter name");
        } else {
            self.parse_binding_name("expected parameter name");
        }
        if self.at(kind::QUESTION) {
            self.bump();
        }
        if self.at(kind::COLON) {
            self.parse_type_annotation(
                kind::TYPE_ANNOTATION,
                &[kind::EQUALS, kind::COMMA, kind::CLOSE_PAREN],
            );
        }
        if self.at(kind::EQUALS) {
            self.parse_initializer(&[kind::COMMA, kind::CLOSE_PAREN]);
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_class_declaration(&mut self, has_decorators: bool) {
        self.builder.start_node(kind::CLASS_DECLARATION);
        if has_decorators {
            self.parse_decorator_list();
        }
        self.parse_modifier_list();
        self.bump();
        if self.at(kind::IDENTIFIER) {
            self.bump();
        } else {
            self.error_here("expected class name");
        }
        if self.at(kind::LESS_THAN) {
            self.parse_balanced_node(
                kind::TYPE_PARAMETERS,
                kind::LESS_THAN,
                kind::GREATER_THAN,
                "expected '>' to close type parameters",
            );
        }
        if self.at(kind::KEYWORD_EXTENDS) || self.at(kind::KEYWORD_IMPLEMENTS) {
            self.parse_heritage_clause();
        }
        if self.at(kind::OPEN_BRACE) {
            self.parse_class_body();
        } else {
            self.error_here("expected class body");
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_decorator_list(&mut self) {
        self.builder.start_node(kind::DECORATOR_LIST);
        while self.at(kind::AT) {
            let start = self.cursor;
            self.builder.start_node(kind::DECORATOR);
            self.bump();
            if self.at(kind::IDENTIFIER) {
                self.bump();
                while self.at(kind::DOT) {
                    let start = self.cursor;
                    self.bump();
                    if self.at(kind::IDENTIFIER) {
                        self.bump();
                    } else {
                        self.error_here("expected decorator property name");
                        break;
                    }
                    self.ensure_progress(start, "decorator path");
                }
            } else {
                self.error_here("expected decorator name");
            }
            if self.at(kind::OPEN_PAREN) {
                self.parse_balanced_node(
                    kind::EXPRESSION,
                    kind::OPEN_PAREN,
                    kind::CLOSE_PAREN,
                    "expected ')' to close decorator arguments",
                );
            }
            self.builder.finish_node();
            self.ensure_progress(start, "decorator list");
        }
        self.builder.finish_node();
    }

    fn parse_heritage_clause(&mut self) {
        self.builder.start_node(kind::HERITAGE_CLAUSE);
        while !self.at_any(&[kind::OPEN_BRACE, kind::END_OF_FILE]) {
            let start = self.cursor;
            self.bump();
            self.ensure_progress(start, "heritage clause");
        }
        self.builder.finish_node();
    }
}
