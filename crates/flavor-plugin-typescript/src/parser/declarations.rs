use crate::syntax_kind::TsSyntaxKind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_type_alias_declaration(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::TypeAliasDeclaration);
        self.parse_modifier_list();
        self.bump();
        if self.at(TsSyntaxKind::Identifier) {
            self.bump();
        } else {
            self.error_here("expected type alias name");
        }
        if self.at(TsSyntaxKind::LessThan) {
            self.parse_balanced_node(
                TsSyntaxKind::TypeParameters,
                TsSyntaxKind::LessThan,
                TsSyntaxKind::GreaterThan,
                "expected '>' to close type parameters",
            );
        }
        if self.expect(TsSyntaxKind::Equals, "expected '=' in type alias") {
            self.parse_type(&[TsSyntaxKind::Semicolon, TsSyntaxKind::EndOfFile]);
        }
        if self.at(TsSyntaxKind::Semicolon) {
            self.bump();
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_interface_declaration(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::InterfaceDeclaration);
        self.parse_modifier_list();
        self.bump();
        if self.at(TsSyntaxKind::Identifier) {
            self.bump();
        } else {
            self.error_here("expected interface name");
        }
        if self.at(TsSyntaxKind::LessThan) {
            self.parse_balanced_node(
                TsSyntaxKind::TypeParameters,
                TsSyntaxKind::LessThan,
                TsSyntaxKind::GreaterThan,
                "expected '>' to close type parameters",
            );
        }
        if self.at(TsSyntaxKind::KeywordExtends) {
            self.parse_heritage_clause();
        }
        self.parse_interface_body();
        self.builder.finish_node();
    }

    pub(super) fn parse_variable_statement(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::VariableStatement);
        self.parse_modifier_list();
        self.builder
            .start_schema_node(TsSyntaxKind::VariableDeclarationList);
        self.bump();

        loop {
            if self.at_any(&[
                TsSyntaxKind::Semicolon,
                TsSyntaxKind::CloseBrace,
                TsSyntaxKind::EndOfFile,
            ]) {
                break;
            }
            self.parse_variable_declaration();
            if self.at(TsSyntaxKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }

        self.builder.finish_node();
        if self.at(TsSyntaxKind::Semicolon) {
            self.bump();
        }
        self.builder.finish_node();
    }

    fn parse_variable_declaration(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::VariableDeclaration);
        self.parse_binding_name("expected variable name");
        if self.at(TsSyntaxKind::Question) || self.at(TsSyntaxKind::Bang) {
            self.bump();
        }
        if self.at(TsSyntaxKind::Colon) {
            self.parse_type_annotation(
                TsSyntaxKind::TypeAnnotation,
                &[
                    TsSyntaxKind::Equals,
                    TsSyntaxKind::Comma,
                    TsSyntaxKind::Semicolon,
                    TsSyntaxKind::CloseParen,
                    TsSyntaxKind::CloseBrace,
                ],
            );
        }
        if self.at(TsSyntaxKind::Equals) {
            self.parse_initializer(&[
                TsSyntaxKind::Comma,
                TsSyntaxKind::Semicolon,
                TsSyntaxKind::CloseBrace,
            ]);
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_function_declaration(&mut self, has_decorators: bool) {
        self.builder
            .start_schema_node(TsSyntaxKind::FunctionDeclaration);
        if has_decorators {
            self.parse_decorator_list();
        }
        self.parse_modifier_list();
        self.bump();
        if self.at(TsSyntaxKind::Identifier) {
            self.bump();
        } else {
            self.error_here("expected function name");
        }
        if self.at(TsSyntaxKind::LessThan) {
            self.parse_balanced_node(
                TsSyntaxKind::TypeParameters,
                TsSyntaxKind::LessThan,
                TsSyntaxKind::GreaterThan,
                "expected '>' to close type parameters",
            );
        }
        self.parse_parameter_list();
        if self.at(TsSyntaxKind::Colon) {
            self.parse_type_annotation(
                TsSyntaxKind::ReturnType,
                &[
                    TsSyntaxKind::OpenBrace,
                    TsSyntaxKind::Semicolon,
                    TsSyntaxKind::EndOfFile,
                ],
            );
        }
        if self.at(TsSyntaxKind::OpenBrace) {
            self.parse_block(TsSyntaxKind::Block);
        } else if self.at(TsSyntaxKind::Semicolon) {
            self.bump();
        } else {
            self.error_here("expected function body");
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_parameter_list(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::ParameterList);
        if !self.expect(
            TsSyntaxKind::OpenParen,
            "expected '(' to start parameter list",
        ) {
            self.builder.finish_node();
            return;
        }
        while !self.at_any(&[TsSyntaxKind::CloseParen, TsSyntaxKind::EndOfFile]) {
            self.parse_parameter();
            if self.at(TsSyntaxKind::Comma) {
                self.bump();
            } else if !self.at(TsSyntaxKind::CloseParen) {
                self.error_here("expected ',' or ')' in parameter list");
                break;
            }
        }
        self.expect(
            TsSyntaxKind::CloseParen,
            "expected ')' to close parameter list",
        );
        self.builder.finish_node();
    }

    fn parse_parameter(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::Parameter);
        if self.at(TsSyntaxKind::DotDotDot) {
            self.parse_rest_element("expected parameter name");
        } else {
            self.parse_binding_name("expected parameter name");
        }
        if self.at(TsSyntaxKind::Question) {
            self.bump();
        }
        if self.at(TsSyntaxKind::Colon) {
            self.parse_type_annotation(
                TsSyntaxKind::TypeAnnotation,
                &[
                    TsSyntaxKind::Equals,
                    TsSyntaxKind::Comma,
                    TsSyntaxKind::CloseParen,
                ],
            );
        }
        if self.at(TsSyntaxKind::Equals) {
            self.parse_initializer(&[TsSyntaxKind::Comma, TsSyntaxKind::CloseParen]);
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_class_declaration(&mut self, has_decorators: bool) {
        self.builder
            .start_schema_node(TsSyntaxKind::ClassDeclaration);
        if has_decorators {
            self.parse_decorator_list();
        }
        self.parse_modifier_list();
        self.bump();
        if self.at(TsSyntaxKind::Identifier) {
            self.bump();
        } else {
            self.error_here("expected class name");
        }
        if self.at(TsSyntaxKind::LessThan) {
            self.parse_balanced_node(
                TsSyntaxKind::TypeParameters,
                TsSyntaxKind::LessThan,
                TsSyntaxKind::GreaterThan,
                "expected '>' to close type parameters",
            );
        }
        if self.at(TsSyntaxKind::KeywordExtends) || self.at(TsSyntaxKind::KeywordImplements) {
            self.parse_heritage_clause();
        }
        if self.at(TsSyntaxKind::OpenBrace) {
            self.parse_class_body();
        } else {
            self.error_here("expected class body");
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_decorator_list(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::DecoratorList);
        while self.at(TsSyntaxKind::At) {
            self.builder.start_schema_node(TsSyntaxKind::Decorator);
            self.bump();
            if self.at(TsSyntaxKind::Identifier) {
                self.bump();
                while self.at(TsSyntaxKind::Dot) {
                    self.bump();
                    if self.at(TsSyntaxKind::Identifier) {
                        self.bump();
                    } else {
                        self.error_here("expected decorator property name");
                        break;
                    }
                }
            } else {
                self.error_here("expected decorator name");
            }
            if self.at(TsSyntaxKind::OpenParen) {
                self.parse_balanced_node(
                    TsSyntaxKind::Expression,
                    TsSyntaxKind::OpenParen,
                    TsSyntaxKind::CloseParen,
                    "expected ')' to close decorator arguments",
                );
            }
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn parse_heritage_clause(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::HeritageClause);
        while !self.at_any(&[TsSyntaxKind::OpenBrace, TsSyntaxKind::EndOfFile]) {
            self.bump();
        }
        self.builder.finish_node();
    }
}
