mod bindings;
mod declarations;
mod expressions;
mod grammar;
mod jsx;
mod members;
mod modules;
mod statements;
mod types;

use flavor_core::{Diagnostic, SourceText, Span, SyntaxNode, Token};
use flavor_grammar::RawAstBuilder;

use crate::{
    ast::TsSourceFile,
    internal::grammar::{self as kind, Kind},
    state::{SourceMode, TsPluginConfig},
};

#[derive(Debug, Clone)]
pub struct TsParseOutput {
    pub source_file: TsSourceFile,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(
    source: SourceText,
    tokens: Vec<Token<Kind>>,
    config: &TsPluginConfig,
) -> TsParseOutput {
    let (syntax, diagnostics) = Parser::new(&source, &tokens, config).parse();
    TsParseOutput {
        source_file: TsSourceFile::new(source, tokens, syntax),
        diagnostics,
    }
}

struct Parser<'a> {
    source: &'a SourceText,
    tokens: &'a [Token<Kind>],
    cursor: usize,
    builder: RawAstBuilder<'static>,
    diagnostics: Vec<Diagnostic>,
    jsx: bool,
}

impl<'a> Parser<'a> {
    fn new(source: &'a SourceText, tokens: &'a [Token<Kind>], config: &TsPluginConfig) -> Self {
        Self {
            source,
            tokens,
            cursor: 0,
            builder: RawAstBuilder::new(kind::schema()),
            diagnostics: Vec::new(),
            jsx: matches!(config.source_mode, SourceMode::Jsx | SourceMode::Tsx)
                && config.jsx.enabled,
        }
    }

    fn parse(mut self) -> (SyntaxNode, Vec<Diagnostic>) {
        self.builder.start_node(kind::SOURCE_FILE);
        while !self.at(kind::END_OF_FILE) {
            self.parse_statement();
        }
        self.builder.finish_node();
        (self.builder.finish(), self.diagnostics)
    }

    fn parse_statement(&mut self) {
        match self.current() {
            kind::AT => match self.kind_after_decorators() {
                Some(kind::KEYWORD_CLASS) => self.parse_class_declaration(true),
                Some(kind::KEYWORD_FUNCTION) => self.parse_function_declaration(true),
                _ => self.parse_unknown_statement(),
            },
            kind::KEYWORD_CLASS => self.parse_class_declaration(false),
            kind::KEYWORD_CONST | kind::KEYWORD_LET => self.parse_variable_statement(),
            kind::KEYWORD_BREAK => self.parse_break_statement(),
            kind::KEYWORD_CONTINUE => self.parse_continue_statement(),
            kind::KEYWORD_DO => self.parse_do_statement(),
            kind::KEYWORD_IMPORT => self.parse_import_declaration(),
            kind::KEYWORD_FOR => self.parse_for_statement(),
            kind::KEYWORD_EXPORT => self.parse_export_declaration(),
            kind::KEYWORD_ENUM => self.parse_enum_declaration(),
            kind::KEYWORD_FUNCTION => self.parse_function_declaration(false),
            kind::KEYWORD_IF => self.parse_if_statement(),
            kind::KEYWORD_INTERFACE => self.parse_interface_declaration(),
            kind::KEYWORD_MODULE | kind::KEYWORD_NAMESPACE => self.parse_namespace_declaration(),
            kind::KEYWORD_RETURN => self.parse_return_statement(),
            kind::KEYWORD_SWITCH => self.parse_switch_statement(),
            kind::KEYWORD_THROW => self.parse_throw_statement(),
            kind::KEYWORD_TRY => self.parse_try_statement(),
            kind::KEYWORD_TYPE => self.parse_type_alias_declaration(),
            kind::KEYWORD_WHILE => self.parse_while_statement(),
            kind if is_modifier_kind(kind) => self.parse_modified_statement(),
            kind::OPEN_BRACE => self.parse_block(kind::BLOCK),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_modified_statement(&mut self) {
        match self.kind_after_modifiers() {
            Some(kind::KEYWORD_CLASS) => self.parse_class_declaration(false),
            Some(kind::KEYWORD_CONST | kind::KEYWORD_LET) => self.parse_variable_statement(),
            Some(kind::KEYWORD_FUNCTION) => self.parse_function_declaration(false),
            Some(kind::KEYWORD_INTERFACE) => self.parse_interface_declaration(),
            Some(kind::KEYWORD_ENUM) => self.parse_enum_declaration(),
            Some(kind::KEYWORD_MODULE | kind::KEYWORD_NAMESPACE) => {
                self.parse_namespace_declaration()
            }
            Some(kind::KEYWORD_TYPE) => self.parse_type_alias_declaration(),
            _ => self.parse_expression_statement(),
        }
    }

    fn kind_after_decorators(&self) -> Option<Kind> {
        let mut cursor = self.cursor;
        while self
            .token_at(cursor)
            .is_some_and(|token| token.kind == kind::AT)
        {
            cursor += 1;
            while let Some(token) = self.token_at(cursor) {
                if matches!(
                    token.kind,
                    kind::AT | kind::KEYWORD_CLASS | kind::KEYWORD_FUNCTION | kind::END_OF_FILE
                ) {
                    break;
                }
                cursor += 1;
            }
        }
        self.token_at(cursor).map(|token| token.kind)
    }

    fn kind_after_modifiers(&self) -> Option<Kind> {
        let mut cursor = self.cursor;
        while self
            .token_at(cursor)
            .is_some_and(|token| is_modifier_kind(token.kind))
        {
            cursor += 1;
        }
        self.token_at(cursor).map(|token| token.kind)
    }

    fn parse_modifier_list(&mut self) {
        if !is_modifier_kind(self.current()) {
            return;
        }
        self.builder.start_node(kind::MODIFIER_LIST);
        while is_modifier_kind(self.current()) {
            self.bump();
        }
        self.builder.finish_node();
    }

    fn expect(&mut self, kind: Kind, message: &str) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            self.error_here(message);
            false
        }
    }

    fn bump(&mut self) {
        if self.at(kind::END_OF_FILE) {
            return;
        }
        let token = &self.tokens[self.cursor];
        self.builder.source_token(self.source, token);
        self.cursor += 1;
    }

    fn at(&self, kind: Kind) -> bool {
        self.current() == kind
    }

    fn at_any(&self, kinds: &[Kind]) -> bool {
        kinds.contains(&self.current())
    }

    fn next_is(&self, kind: Kind) -> bool {
        self.token_kind_at(1) == kind
    }

    fn jsx_enabled(&self) -> bool {
        self.jsx
    }

    fn current(&self) -> Kind {
        self.token_at(self.cursor)
            .map_or(kind::END_OF_FILE, |token| token.kind)
    }

    fn token_kind_at(&self, offset: usize) -> Kind {
        self.token_at(self.cursor + offset)
            .map_or(kind::END_OF_FILE, |token| token.kind)
    }

    fn token_kind_at_back(&self, offset: usize) -> Kind {
        self.cursor
            .checked_sub(offset)
            .and_then(|cursor| self.token_at(cursor))
            .map_or(kind::END_OF_FILE, |token| token.kind)
    }

    fn token_at(&self, cursor: usize) -> Option<&'a Token<Kind>> {
        self.tokens.get(cursor)
    }

    fn has_top_level_any(&self, kinds: &[Kind], stops: &[Kind]) -> bool {
        let mut cursor = self.cursor;
        let mut paren_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut angle_depth = 0usize;
        while let Some(token) = self.token_at(cursor) {
            if token.kind == kind::END_OF_FILE {
                return false;
            }
            if paren_depth == 0
                && brace_depth == 0
                && bracket_depth == 0
                && angle_depth == 0
                && stops.contains(&token.kind)
            {
                return false;
            }
            if paren_depth == 0
                && brace_depth == 0
                && bracket_depth == 0
                && angle_depth == 0
                && kinds.contains(&token.kind)
            {
                return true;
            }
            match token.kind {
                kind::OPEN_PAREN => paren_depth += 1,
                kind::CLOSE_PAREN if paren_depth > 0 => paren_depth -= 1,
                kind::OPEN_BRACE => brace_depth += 1,
                kind::CLOSE_BRACE if brace_depth > 0 => brace_depth -= 1,
                kind::OPEN_BRACKET => bracket_depth += 1,
                kind::CLOSE_BRACKET if bracket_depth > 0 => bracket_depth -= 1,
                kind::LESS_THAN => angle_depth += 1,
                kind::GREATER_THAN if angle_depth > 0 => angle_depth -= 1,
                _ => {}
            }
            cursor += 1;
        }
        false
    }

    fn current_span(&self) -> Span {
        self.token_at(self.cursor)
            .or_else(|| self.tokens.last())
            .map_or_else(|| Span::new(0, 0), |token| token.span)
    }

    fn error_here(&mut self, message: &str) {
        self.error_at(self.current_span(), message);
    }

    fn error_at(&mut self, span: Span, message: &str) {
        self.diagnostics.push(Diagnostic::error_code(
            span,
            "ts/parse/error",
            message.to_string(),
        ));
    }
}

fn is_modifier_kind(kind: Kind) -> bool {
    matches!(
        kind,
        kind::KEYWORD_ABSTRACT
            | kind::KEYWORD_ASYNC
            | kind::KEYWORD_DECLARE
            | kind::KEYWORD_OVERRIDE
            | kind::KEYWORD_PRIVATE
            | kind::KEYWORD_PROTECTED
            | kind::KEYWORD_PUBLIC
            | kind::KEYWORD_READONLY
            | kind::KEYWORD_STATIC
    )
}
