use crate::syntax_kind::TsSyntaxKind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_import_declaration(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::ImportDeclaration);
        self.bump();
        if self.at(TsSyntaxKind::StringLiteral) {
            self.bump();
        } else if self.is_import_equals_start() {
            self.parse_import_equals();
        } else {
            self.parse_import_clause();
            self.parse_from_clause("expected module source in import declaration");
        }
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    pub(super) fn parse_export_declaration(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::ExportDeclaration);
        self.bump();
        if self.at(TsSyntaxKind::KeywordDefault) {
            self.bump();
        }
        match self.current() {
            TsSyntaxKind::Equals => self.parse_export_assignment(),
            TsSyntaxKind::KeywordAs if self.next_is(TsSyntaxKind::KeywordNamespace) => {
                self.parse_namespace_export()
            }
            TsSyntaxKind::OpenBrace => self.parse_export_clause(),
            TsSyntaxKind::Star => self.parse_export_star_clause(),
            TsSyntaxKind::KeywordType if self.next_is(TsSyntaxKind::Star) => {
                self.bump();
                self.parse_export_star_clause();
            }
            TsSyntaxKind::KeywordType if self.next_is(TsSyntaxKind::OpenBrace) => {
                self.bump();
                self.parse_export_clause();
            }
            TsSyntaxKind::KeywordClass => self.parse_class_declaration(false),
            TsSyntaxKind::KeywordConst | TsSyntaxKind::KeywordLet => {
                self.parse_variable_statement()
            }
            TsSyntaxKind::KeywordEnum => self.parse_enum_declaration(),
            TsSyntaxKind::KeywordFunction => self.parse_function_declaration(false),
            TsSyntaxKind::KeywordInterface => self.parse_interface_declaration(),
            TsSyntaxKind::KeywordModule | TsSyntaxKind::KeywordNamespace => {
                self.parse_namespace_declaration()
            }
            TsSyntaxKind::KeywordType => self.parse_type_alias_declaration(),
            _ => self.parse_modified_export(),
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_enum_declaration(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::EnumDeclaration);
        self.parse_modifier_list();
        self.bump();
        if self.at(TsSyntaxKind::Identifier) {
            self.bump();
        } else {
            self.error_here("expected enum name");
        }
        self.parse_enum_body();
        self.builder.finish_node();
    }

    pub(super) fn parse_namespace_declaration(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::NamespaceDeclaration);
        self.parse_modifier_list();
        self.bump();
        self.parse_namespace_name();
        self.builder.start_schema_node(TsSyntaxKind::NamespaceBody);
        if self.expect(
            TsSyntaxKind::OpenBrace,
            "expected '{' to start namespace body",
        ) {
            while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
                self.parse_statement();
            }
            self.expect(
                TsSyntaxKind::CloseBrace,
                "expected '}' to close namespace body",
            );
        }
        self.builder.finish_node();
        self.builder.finish_node();
    }

    fn is_import_equals_start(&self) -> bool {
        self.at(TsSyntaxKind::Identifier) && self.next_is(TsSyntaxKind::Equals)
            || self.at(TsSyntaxKind::KeywordType)
                && self.token_kind_at(1) == TsSyntaxKind::Identifier
                && self.token_kind_at(2) == TsSyntaxKind::Equals
    }

    fn parse_import_equals(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::ImportEqualsDeclaration);
        if self.at(TsSyntaxKind::KeywordType) {
            self.bump();
        }
        self.bump();
        if self.expect(TsSyntaxKind::Equals, "expected '=' in import assignment") {
            if self.is_require_call() {
                self.parse_external_reference();
            } else {
                self.parse_expression(&[
                    TsSyntaxKind::Semicolon,
                    TsSyntaxKind::CloseBrace,
                    TsSyntaxKind::EndOfFile,
                ]);
            }
        }
        self.builder.finish_node();
    }

    fn parse_external_reference(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::ExternalModuleReference);
        self.bump();
        self.parse_balanced_node(
            TsSyntaxKind::ParenthesizedExpression,
            TsSyntaxKind::OpenParen,
            TsSyntaxKind::CloseParen,
            "expected ')' to close external module reference",
        );
        self.builder.finish_node();
    }

    fn parse_import_clause(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::ImportClause);
        if self.at(TsSyntaxKind::KeywordType) {
            self.bump();
        }
        if self.at(TsSyntaxKind::Identifier) {
            self.bump();
            if self.at(TsSyntaxKind::Comma) {
                self.bump();
            }
        }
        if self.at(TsSyntaxKind::Star) {
            self.parse_namespace_import();
        } else if self.at(TsSyntaxKind::OpenBrace) {
            self.parse_named_imports();
        }
        self.parse_balanced_tokens_until(&[
            TsSyntaxKind::KeywordFrom,
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::EndOfFile,
        ]);
        self.builder.finish_node();
    }

    fn parse_namespace_import(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::NamespaceImport);
        self.bump();
        if self.at(TsSyntaxKind::KeywordAs) {
            self.bump();
        } else {
            self.error_here("expected 'as' in namespace import");
        }
        if self.at(TsSyntaxKind::Identifier) {
            self.bump();
        } else {
            self.error_here("expected namespace import name");
        }
        self.builder.finish_node();
    }

    fn parse_named_imports(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::NamedImports);
        if self.expect(
            TsSyntaxKind::OpenBrace,
            "expected '{' to start named imports",
        ) {
            while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
                self.parse_import_specifier();
                if self.at(TsSyntaxKind::Comma) {
                    self.bump();
                } else if !self.at(TsSyntaxKind::CloseBrace) {
                    self.error_here("expected ',' or '}' in named imports");
                    break;
                }
            }
            self.expect(
                TsSyntaxKind::CloseBrace,
                "expected '}' to close named imports",
            );
        }
        self.builder.finish_node();
    }

    fn parse_import_specifier(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::ImportSpecifier);
        self.parse_binding_specifier("expected import specifier");
        self.builder.finish_node();
    }

    fn parse_export_clause(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::ExportClause);
        self.parse_named_exports();
        self.parse_from_clause("expected module source after export clause");
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn parse_export_star_clause(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::ExportClause);
        self.bump();
        if self.at(TsSyntaxKind::KeywordAs) {
            self.bump();
            if self.at(TsSyntaxKind::Identifier) {
                self.bump();
            } else {
                self.error_here("expected export namespace name");
            }
        }
        self.parse_from_clause("expected module source after export star");
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn parse_export_assignment(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::ExportAssignment);
        self.bump();
        self.parse_expression(&[
            TsSyntaxKind::Semicolon,
            TsSyntaxKind::CloseBrace,
            TsSyntaxKind::EndOfFile,
        ]);
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn parse_namespace_export(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::NamespaceExportDeclaration);
        self.bump();
        self.bump();
        if self.at(TsSyntaxKind::Identifier) {
            self.bump();
        } else {
            self.error_here("expected namespace export name");
        }
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn parse_named_exports(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::NamedExports);
        if self.expect(
            TsSyntaxKind::OpenBrace,
            "expected '{' to start named exports",
        ) {
            while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
                self.parse_export_specifier();
                if self.at(TsSyntaxKind::Comma) {
                    self.bump();
                } else if !self.at(TsSyntaxKind::CloseBrace) {
                    self.error_here("expected ',' or '}' in named exports");
                    break;
                }
            }
            self.expect(
                TsSyntaxKind::CloseBrace,
                "expected '}' to close named exports",
            );
        }
        self.builder.finish_node();
    }

    fn parse_export_specifier(&mut self) {
        self.builder
            .start_schema_node(TsSyntaxKind::ExportSpecifier);
        self.parse_binding_specifier("expected export specifier");
        self.builder.finish_node();
    }

    fn parse_binding_specifier(&mut self, message: &str) {
        if self.at(TsSyntaxKind::KeywordType) {
            self.bump();
        }
        if self.is_specifier_name() {
            self.bump();
            if self.at(TsSyntaxKind::KeywordAs) {
                self.bump();
                if self.is_specifier_name() {
                    self.bump();
                } else {
                    self.error_here("expected alias name");
                }
            }
        } else {
            self.error_here(message);
            if !self.at_any(&[
                TsSyntaxKind::Comma,
                TsSyntaxKind::CloseBrace,
                TsSyntaxKind::EndOfFile,
            ]) {
                self.bump();
            }
        }
    }

    fn is_specifier_name(&self) -> bool {
        matches!(
            self.current(),
            TsSyntaxKind::Identifier
                | TsSyntaxKind::KeywordDefault
                | TsSyntaxKind::KeywordSatisfies
                | TsSyntaxKind::KeywordKeyof
                | TsSyntaxKind::KeywordInfer
                | TsSyntaxKind::KeywordUnique
        )
    }

    fn is_require_call(&self) -> bool {
        self.at(TsSyntaxKind::Identifier)
            && self.source.slice(self.current_span()) == "require"
            && self.next_is(TsSyntaxKind::OpenParen)
    }

    fn parse_modified_export(&mut self) {
        match self.kind_after_modifiers() {
            Some(TsSyntaxKind::KeywordClass) => self.parse_class_declaration(false),
            Some(TsSyntaxKind::KeywordConst | TsSyntaxKind::KeywordLet) => {
                self.parse_variable_statement()
            }
            Some(TsSyntaxKind::KeywordEnum) => self.parse_enum_declaration(),
            Some(TsSyntaxKind::KeywordFunction) => self.parse_function_declaration(false),
            Some(TsSyntaxKind::KeywordInterface) => self.parse_interface_declaration(),
            Some(TsSyntaxKind::KeywordModule | TsSyntaxKind::KeywordNamespace) => {
                self.parse_namespace_declaration()
            }
            Some(TsSyntaxKind::KeywordType) => self.parse_type_alias_declaration(),
            _ => {
                self.parse_balanced_tokens_until(&[
                    TsSyntaxKind::Semicolon,
                    TsSyntaxKind::CloseBrace,
                    TsSyntaxKind::EndOfFile,
                ]);
                self.parse_optional_semicolon();
            }
        }
    }

    fn parse_from_clause(&mut self, message: &str) {
        if self.at(TsSyntaxKind::KeywordFrom) {
            self.bump();
            if self.at(TsSyntaxKind::StringLiteral) {
                self.bump();
            } else {
                self.error_here(message);
            }
        } else if !self.at_any(&[TsSyntaxKind::Semicolon, TsSyntaxKind::EndOfFile]) {
            self.error_here("expected 'from'");
        }
    }

    fn parse_enum_body(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::EnumBody);
        if self.expect(TsSyntaxKind::OpenBrace, "expected '{' to start enum body") {
            while !self.at_any(&[TsSyntaxKind::CloseBrace, TsSyntaxKind::EndOfFile]) {
                self.parse_enum_member();
                if self.at(TsSyntaxKind::Comma) {
                    self.bump();
                } else if !self.at(TsSyntaxKind::CloseBrace) {
                    self.error_here("expected ',' or '}' in enum body");
                    break;
                }
            }
            self.expect(TsSyntaxKind::CloseBrace, "expected '}' to close enum body");
        }
        self.builder.finish_node();
    }

    fn parse_enum_member(&mut self) {
        self.builder.start_schema_node(TsSyntaxKind::EnumMember);
        if self.at(TsSyntaxKind::Identifier) || self.at(TsSyntaxKind::StringLiteral) {
            self.bump();
        } else {
            self.error_here("expected enum member name");
        }
        if self.at(TsSyntaxKind::Equals) {
            self.parse_initializer(&[TsSyntaxKind::Comma, TsSyntaxKind::CloseBrace]);
        }
        self.builder.finish_node();
    }

    fn parse_namespace_name(&mut self) {
        if self.at(TsSyntaxKind::StringLiteral) {
            self.bump();
            return;
        }
        if self.at(TsSyntaxKind::Identifier) {
            self.bump();
            while self.at(TsSyntaxKind::Dot) {
                self.bump();
                if self.at(TsSyntaxKind::Identifier) {
                    self.bump();
                } else {
                    self.error_here("expected namespace segment");
                    break;
                }
            }
        } else {
            self.error_here("expected namespace name");
        }
    }
}
