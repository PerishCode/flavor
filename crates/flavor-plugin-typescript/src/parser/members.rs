use crate::syntax_kind::TsSyntaxKind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_class_member(&mut self) {
        self.builder.start_node(TsSyntaxKind::ClassMember);
        if self.at(TsSyntaxKind::At) {
            self.parse_decorator_list();
        }
        self.parse_modifier_list();
        let member_kind = if self.is_method_member() {
            TsSyntaxKind::MethodDeclaration
        } else {
            TsSyntaxKind::PropertyDeclaration
        };
        self.builder.start_node(member_kind);
        self.parse_member_name("expected class member name");
        if member_kind == TsSyntaxKind::MethodDeclaration {
            self.parse_member_type_params();
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
            }
        } else {
            self.parse_property_tail();
        }
        self.builder.finish_node();
        self.builder.finish_node();
    }

    pub(super) fn parse_interface_body(&mut self) {
        self.builder.start_node(TsSyntaxKind::InterfaceBody);
        if !self.expect(TsSyntaxKind::OpenBrace, "expected interface body") {
            self.builder.finish_node();
            return;
        }
        while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
            self.parse_interface_member();
        }
        self.expect(
            TsSyntaxKind::CloseBrace,
            "expected '}' to close interface body",
        );
        self.builder.finish_node();
    }

    fn parse_interface_member(&mut self) {
        self.parse_modifier_list();
        let member_kind = if self.is_method_member() {
            TsSyntaxKind::MethodSignature
        } else {
            TsSyntaxKind::PropertySignature
        };
        self.builder.start_node(member_kind);
        if !self.parse_member_name("expected interface member name") {
            self.recover_member();
            self.builder.finish_node();
            return;
        }
        if member_kind == TsSyntaxKind::MethodSignature {
            self.parse_member_type_params();
            self.parse_parameter_list();
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
        } else {
            if self.at(TsSyntaxKind::Question) {
                self.bump();
            }
            if self.at(TsSyntaxKind::Colon) {
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
        if self.at(TsSyntaxKind::Semicolon) || self.at(TsSyntaxKind::Comma) {
            self.bump();
        }
        self.builder.finish_node();
    }

    fn parse_property_tail(&mut self) {
        if self.at(TsSyntaxKind::Question) || self.at(TsSyntaxKind::Bang) {
            self.bump();
        }
        if self.at(TsSyntaxKind::Colon) {
            self.parse_type_annotation(
                TsSyntaxKind::TypeAnnotation,
                &[
                    TsSyntaxKind::Equals,
                    TsSyntaxKind::Semicolon,
                    TsSyntaxKind::CloseBrace,
                ],
            );
        }
        if self.at(TsSyntaxKind::Equals) {
            self.parse_initializer(&[TsSyntaxKind::Semicolon, TsSyntaxKind::CloseBrace]);
        }
        if self.at(TsSyntaxKind::Semicolon) {
            self.bump();
        } else if !self.at(TsSyntaxKind::CloseBrace) {
            self.recover_member();
        }
    }

    fn recover_member(&mut self) {
        while !self.at_any(&[
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::Comma,
            TsSyntaxKind::CloseBrace,
            TsSyntaxKind::EndOfFile,
        ]) {
            self.bump();
        }
        if self.at(TsSyntaxKind::Semicolon) || self.at(TsSyntaxKind::Comma) {
            self.bump();
        }
    }

    fn is_method_member(&self) -> bool {
        self.next_is(TsSyntaxKind::OpenParen)
            || self.current() == TsSyntaxKind::KeywordNew
                && self.token_kind_at(1) == TsSyntaxKind::OpenParen
            || self.token_kind_at(1) == TsSyntaxKind::LessThan
            || matches!(
                self.current(),
                TsSyntaxKind::KeywordGet | TsSyntaxKind::KeywordSet
            ) && self.token_kind_at(1) == TsSyntaxKind::Identifier
                && self.token_kind_at(2) == TsSyntaxKind::OpenParen
    }

    fn parse_member_name(&mut self, message: &str) -> bool {
        if matches!(
            self.current(),
            TsSyntaxKind::KeywordGet | TsSyntaxKind::KeywordSet
        ) && self.token_kind_at(1) == TsSyntaxKind::Identifier
            && self.token_kind_at(2) == TsSyntaxKind::OpenParen
        {
            self.bump();
            self.bump();
            return true;
        }
        if matches!(
            self.current(),
            TsSyntaxKind::Identifier
                | TsSyntaxKind::KeywordGet
                | TsSyntaxKind::KeywordSet
                | TsSyntaxKind::KeywordNew
                | TsSyntaxKind::KeywordSatisfies
                | TsSyntaxKind::KeywordKeyof
                | TsSyntaxKind::KeywordInfer
                | TsSyntaxKind::KeywordUnique
                | TsSyntaxKind::KeywordThis
                | TsSyntaxKind::KeywordSuper
                | TsSyntaxKind::KeywordTrue
                | TsSyntaxKind::KeywordFalse
                | TsSyntaxKind::KeywordNull
        ) {
            self.bump();
            return true;
        }
        self.error_here(message);
        false
    }

    fn parse_member_type_params(&mut self) {
        if self.at(TsSyntaxKind::LessThan) {
            self.parse_balanced_node(
                TsSyntaxKind::TypeParameters,
                TsSyntaxKind::LessThan,
                TsSyntaxKind::GreaterThan,
                "expected '>' to close member type parameters",
            );
        }
    }
}
