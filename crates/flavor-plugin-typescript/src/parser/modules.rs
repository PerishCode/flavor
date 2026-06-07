use crate::internal::grammar as kind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_import_declaration(&mut self) {
        self.builder.start_node(kind::IMPORT_DECLARATION);
        self.bump();
        if self.at(kind::STRING_LITERAL) {
            self.bump();
        } else if self.is_import_equals_start() {
            self.import_equals();
        } else {
            self.import_clause();
            self.module_source_clause("expected module source in import declaration");
        }
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    pub(super) fn parse_export_declaration(&mut self) {
        self.builder.start_node(kind::EXPORT_DECLARATION);
        self.bump();
        if self.at(kind::KEYWORD_DEFAULT) {
            self.bump();
        }
        match self.current() {
            kind::EQUALS => self.export_assignment(),
            kind::KEYWORD_AS if self.next_is(kind::KEYWORD_NAMESPACE) => self.namespace_export(),
            kind::OPEN_BRACE => self.export_clause(),
            kind::STAR => self.export_star_clause(),
            kind::KEYWORD_TYPE if self.next_is(kind::STAR) => {
                self.bump();
                self.export_star_clause();
            }
            kind::KEYWORD_TYPE if self.next_is(kind::OPEN_BRACE) => {
                self.bump();
                self.export_clause();
            }
            kind::KEYWORD_CLASS => self.parse_class_declaration(false),
            kind::KEYWORD_CONST | kind::KEYWORD_LET => self.parse_variable_statement(),
            kind::KEYWORD_ENUM => self.parse_enum_declaration(),
            kind::KEYWORD_FUNCTION => self.parse_function_declaration(false),
            kind::KEYWORD_INTERFACE => self.parse_interface_declaration(),
            kind::KEYWORD_MODULE | kind::KEYWORD_NAMESPACE => self.parse_namespace_declaration(),
            kind::KEYWORD_TYPE => self.parse_type_alias_declaration(),
            _ => self.modified_export(),
        }
        self.builder.finish_node();
    }

    pub(super) fn parse_enum_declaration(&mut self) {
        self.builder.start_node(kind::ENUM_DECLARATION);
        self.parse_modifier_list();
        self.bump();
        if self.at(kind::IDENTIFIER) {
            self.bump();
        } else {
            self.error_here("expected enum name");
        }
        self.enum_body();
        self.builder.finish_node();
    }

    pub(super) fn parse_namespace_declaration(&mut self) {
        self.builder.start_node(kind::NAMESPACE_DECLARATION);
        self.parse_modifier_list();
        self.bump();
        self.namespace_name();
        self.builder.start_node(kind::NAMESPACE_BODY);
        if self.expect(kind::OPEN_BRACE, "expected '{' to start namespace body") {
            while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
                let start = self.cursor;
                self.parse_statement();
                self.ensure_progress(start, "namespace body");
            }
            self.expect(kind::CLOSE_BRACE, "expected '}' to close namespace body");
        }
        self.builder.finish_node();
        self.builder.finish_node();
    }

    fn is_import_equals_start(&self) -> bool {
        self.at(kind::IDENTIFIER) && self.next_is(kind::EQUALS)
            || self.at(kind::KEYWORD_TYPE)
                && self.token_kind_at(1) == kind::IDENTIFIER
                && self.token_kind_at(2) == kind::EQUALS
    }

    fn import_equals(&mut self) {
        self.builder.start_node(kind::IMPORT_EQUALS_DECLARATION);
        if self.at(kind::KEYWORD_TYPE) {
            self.bump();
        }
        self.bump();
        if self.expect(kind::EQUALS, "expected '=' in import assignment") {
            if self.is_require_call() {
                self.external_reference();
            } else {
                self.parse_expression(&[kind::SEMICOLON, kind::CLOSE_BRACE, kind::END_OF_FILE]);
            }
        }
        self.builder.finish_node();
    }

    fn external_reference(&mut self) {
        self.builder.start_node(kind::EXTERNAL_MODULE_REFERENCE);
        self.bump();
        self.parse_balanced_node(
            kind::PARENTHESIZED_EXPRESSION,
            kind::OPEN_PAREN,
            kind::CLOSE_PAREN,
            "expected ')' to close external module reference",
        );
        self.builder.finish_node();
    }

    fn import_clause(&mut self) {
        self.builder.start_node(kind::IMPORT_CLAUSE);
        if self.at(kind::KEYWORD_TYPE) {
            self.bump();
        }
        if self.at(kind::IDENTIFIER) {
            self.bump();
            if self.at(kind::COMMA) {
                self.bump();
            }
        }
        if self.at(kind::STAR) {
            self.namespace_import();
        } else if self.at(kind::OPEN_BRACE) {
            self.named_imports();
        }
        self.parse_balanced_tokens_until(&[kind::KEYWORD_FROM, kind::SEMICOLON, kind::END_OF_FILE]);
        self.builder.finish_node();
    }

    fn namespace_import(&mut self) {
        self.builder.start_node(kind::NAMESPACE_IMPORT);
        self.bump();
        if self.at(kind::KEYWORD_AS) {
            self.bump();
        } else {
            self.error_here("expected 'as' in namespace import");
        }
        if self.at(kind::IDENTIFIER) {
            self.bump();
        } else {
            self.error_here("expected namespace import name");
        }
        self.builder.finish_node();
    }

    fn named_imports(&mut self) {
        self.builder.start_node(kind::NAMED_IMPORTS);
        if self.expect(kind::OPEN_BRACE, "expected '{' to start named imports") {
            while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
                let start = self.cursor;
                self.import_specifier();
                if self.at(kind::COMMA) {
                    self.bump();
                } else if !self.at(kind::CLOSE_BRACE) {
                    self.error_here("expected ',' or '}' in named imports");
                    break;
                }
                self.ensure_progress(start, "named imports");
            }
            self.expect(kind::CLOSE_BRACE, "expected '}' to close named imports");
        }
        self.builder.finish_node();
    }

    fn import_specifier(&mut self) {
        self.builder.start_node(kind::IMPORT_SPECIFIER);
        self.binding_specifier("expected import specifier");
        self.builder.finish_node();
    }

    fn export_clause(&mut self) {
        self.builder.start_node(kind::EXPORT_CLAUSE);
        self.named_exports();
        self.module_source_clause("expected module source after export clause");
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn export_star_clause(&mut self) {
        self.builder.start_node(kind::EXPORT_CLAUSE);
        self.bump();
        if self.at(kind::KEYWORD_AS) {
            self.bump();
            if self.at(kind::IDENTIFIER) {
                self.bump();
            } else {
                self.error_here("expected export namespace name");
            }
        }
        self.module_source_clause("expected module source after export star");
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn export_assignment(&mut self) {
        self.builder.start_node(kind::EXPORT_ASSIGNMENT);
        self.bump();
        self.parse_expression(&[kind::SEMICOLON, kind::CLOSE_BRACE, kind::END_OF_FILE]);
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn namespace_export(&mut self) {
        self.builder.start_node(kind::NAMESPACE_EXPORT_DECLARATION);
        self.bump();
        self.bump();
        if self.at(kind::IDENTIFIER) {
            self.bump();
        } else {
            self.error_here("expected namespace export name");
        }
        self.parse_optional_semicolon();
        self.builder.finish_node();
    }

    fn named_exports(&mut self) {
        self.builder.start_node(kind::NAMED_EXPORTS);
        if self.expect(kind::OPEN_BRACE, "expected '{' to start named exports") {
            while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
                let start = self.cursor;
                self.export_specifier();
                if self.at(kind::COMMA) {
                    self.bump();
                } else if !self.at(kind::CLOSE_BRACE) {
                    self.error_here("expected ',' or '}' in named exports");
                    break;
                }
                self.ensure_progress(start, "named exports");
            }
            self.expect(kind::CLOSE_BRACE, "expected '}' to close named exports");
        }
        self.builder.finish_node();
    }

    fn export_specifier(&mut self) {
        self.builder.start_node(kind::EXPORT_SPECIFIER);
        self.binding_specifier("expected export specifier");
        self.builder.finish_node();
    }

    fn binding_specifier(&mut self, message: &str) {
        if self.at(kind::KEYWORD_TYPE) {
            self.bump();
        }
        if self.is_specifier_name() {
            self.bump();
            if self.at(kind::KEYWORD_AS) {
                self.bump();
                if self.is_specifier_name() {
                    self.bump();
                } else {
                    self.error_here("expected alias name");
                }
            }
        } else {
            self.error_here(message);
            if !self.at_any(&[kind::COMMA, kind::CLOSE_BRACE, kind::END_OF_FILE]) {
                self.bump();
            }
        }
    }

    fn is_specifier_name(&self) -> bool {
        matches!(
            self.current(),
            kind::IDENTIFIER
                | kind::KEYWORD_DEFAULT
                | kind::KEYWORD_SATISFIES
                | kind::KEYWORD_KEYOF
                | kind::KEYWORD_INFER
                | kind::KEYWORD_UNIQUE
        )
    }

    fn is_require_call(&self) -> bool {
        self.at(kind::IDENTIFIER)
            && self.source.slice(self.current_span()) == "require"
            && self.next_is(kind::OPEN_PAREN)
    }

    fn modified_export(&mut self) {
        match self.kind_after_modifiers() {
            Some(kind::KEYWORD_CLASS) => self.parse_class_declaration(false),
            Some(kind::KEYWORD_CONST | kind::KEYWORD_LET) => self.parse_variable_statement(),
            Some(kind::KEYWORD_ENUM) => self.parse_enum_declaration(),
            Some(kind::KEYWORD_FUNCTION) => self.parse_function_declaration(false),
            Some(kind::KEYWORD_INTERFACE) => self.parse_interface_declaration(),
            Some(kind::KEYWORD_MODULE | kind::KEYWORD_NAMESPACE) => {
                self.parse_namespace_declaration()
            }
            Some(kind::KEYWORD_TYPE) => self.parse_type_alias_declaration(),
            _ => {
                self.parse_balanced_tokens_until(&[
                    kind::SEMICOLON,
                    kind::CLOSE_BRACE,
                    kind::END_OF_FILE,
                ]);
                self.parse_optional_semicolon();
            }
        }
    }

    fn module_source_clause(&mut self, message: &str) {
        if self.at(kind::KEYWORD_FROM) {
            self.bump();
            if self.at(kind::STRING_LITERAL) {
                self.bump();
            } else {
                self.error_here(message);
            }
        } else if !self.at_any(&[kind::SEMICOLON, kind::END_OF_FILE]) {
            self.error_here("expected 'from'");
        }
    }

    fn enum_body(&mut self) {
        self.builder.start_node(kind::ENUM_BODY);
        if self.expect(kind::OPEN_BRACE, "expected '{' to start enum body") {
            while !self.at_any(&[kind::CLOSE_BRACE, kind::END_OF_FILE]) {
                let start = self.cursor;
                self.enum_member();
                if self.at(kind::COMMA) {
                    self.bump();
                } else if !self.at(kind::CLOSE_BRACE) {
                    self.error_here("expected ',' or '}' in enum body");
                    break;
                }
                self.ensure_progress(start, "enum body");
            }
            self.expect(kind::CLOSE_BRACE, "expected '}' to close enum body");
        }
        self.builder.finish_node();
    }

    fn enum_member(&mut self) {
        self.builder.start_node(kind::ENUM_MEMBER);
        if self.at(kind::IDENTIFIER) || self.at(kind::STRING_LITERAL) {
            self.bump();
        } else {
            self.error_here("expected enum member name");
        }
        if self.at(kind::EQUALS) {
            self.parse_initializer(&[kind::COMMA, kind::CLOSE_BRACE]);
        }
        self.builder.finish_node();
    }

    fn namespace_name(&mut self) {
        if self.at(kind::STRING_LITERAL) {
            self.bump();
            return;
        }
        if self.at(kind::IDENTIFIER) {
            self.bump();
            while self.at(kind::DOT) {
                let start = self.cursor;
                self.bump();
                if self.at(kind::IDENTIFIER) {
                    self.bump();
                } else {
                    self.error_here("expected namespace segment");
                    break;
                }
                self.ensure_progress(start, "namespace name");
            }
        } else {
            self.error_here("expected namespace name");
        }
    }
}
