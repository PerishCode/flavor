mod bindings;
mod declarations;
mod expressions;
mod grammar;
mod jsx;
mod members;
mod modules;
mod statements;
mod types;

use flavor_core::{Diagnostic, SourceText, Span, SyntaxBuilder, SyntaxNode, Token};

use crate::{
    ast::TsSourceFile,
    state::{SourceMode, TsPluginConfig},
    syntax_kind::TsSyntaxKind,
};

#[derive(Debug, Clone)]
pub struct TsParseOutput {
    pub source_file: TsSourceFile,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(
    source: SourceText,
    tokens: Vec<Token<TsSyntaxKind>>,
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
    tokens: &'a [Token<TsSyntaxKind>],
    cursor: usize,
    builder: SyntaxBuilder,
    diagnostics: Vec<Diagnostic>,
    jsx: bool,
}

impl<'a> Parser<'a> {
    fn new(
        source: &'a SourceText,
        tokens: &'a [Token<TsSyntaxKind>],
        config: &TsPluginConfig,
    ) -> Self {
        Self {
            source,
            tokens,
            cursor: 0,
            builder: SyntaxBuilder::new(),
            diagnostics: Vec::new(),
            jsx: matches!(config.source_mode, SourceMode::Jsx | SourceMode::Tsx)
                && config.jsx.enabled,
        }
    }

    fn parse(mut self) -> (SyntaxNode, Vec<Diagnostic>) {
        self.builder.start_schema_node(TsSyntaxKind::SourceFile);
        while !self.at(TsSyntaxKind::EndOfFile) {
            self.parse_statement();
        }
        self.builder.finish_node();
        (self.builder.finish(), self.diagnostics)
    }

    fn parse_statement(&mut self) {
        match self.current() {
            TsSyntaxKind::At => match self.kind_after_decorators() {
                Some(TsSyntaxKind::KeywordClass) => self.parse_class_declaration(true),
                Some(TsSyntaxKind::KeywordFunction) => self.parse_function_declaration(true),
                _ => self.parse_unknown_statement(),
            },
            TsSyntaxKind::KeywordClass => self.parse_class_declaration(false),
            TsSyntaxKind::KeywordConst | TsSyntaxKind::KeywordLet => {
                self.parse_variable_statement()
            }
            TsSyntaxKind::KeywordBreak => self.parse_break_statement(),
            TsSyntaxKind::KeywordContinue => self.parse_continue_statement(),
            TsSyntaxKind::KeywordDo => self.parse_do_statement(),
            TsSyntaxKind::KeywordImport => self.parse_import_declaration(),
            TsSyntaxKind::KeywordFor => self.parse_for_statement(),
            TsSyntaxKind::KeywordExport => self.parse_export_declaration(),
            TsSyntaxKind::KeywordEnum => self.parse_enum_declaration(),
            TsSyntaxKind::KeywordFunction => self.parse_function_declaration(false),
            TsSyntaxKind::KeywordIf => self.parse_if_statement(),
            TsSyntaxKind::KeywordInterface => self.parse_interface_declaration(),
            TsSyntaxKind::KeywordModule | TsSyntaxKind::KeywordNamespace => {
                self.parse_namespace_declaration()
            }
            TsSyntaxKind::KeywordReturn => self.parse_return_statement(),
            TsSyntaxKind::KeywordSwitch => self.parse_switch_statement(),
            TsSyntaxKind::KeywordThrow => self.parse_throw_statement(),
            TsSyntaxKind::KeywordTry => self.parse_try_statement(),
            TsSyntaxKind::KeywordType => self.parse_type_alias_declaration(),
            TsSyntaxKind::KeywordWhile => self.parse_while_statement(),
            kind if is_modifier_kind(kind) => self.parse_modified_statement(),
            TsSyntaxKind::OpenBrace => self.parse_block(TsSyntaxKind::Block),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_modified_statement(&mut self) {
        match self.kind_after_modifiers() {
            Some(TsSyntaxKind::KeywordClass) => self.parse_class_declaration(false),
            Some(TsSyntaxKind::KeywordConst | TsSyntaxKind::KeywordLet) => {
                self.parse_variable_statement()
            }
            Some(TsSyntaxKind::KeywordFunction) => self.parse_function_declaration(false),
            Some(TsSyntaxKind::KeywordInterface) => self.parse_interface_declaration(),
            Some(TsSyntaxKind::KeywordEnum) => self.parse_enum_declaration(),
            Some(TsSyntaxKind::KeywordModule | TsSyntaxKind::KeywordNamespace) => {
                self.parse_namespace_declaration()
            }
            Some(TsSyntaxKind::KeywordType) => self.parse_type_alias_declaration(),
            _ => self.parse_expression_statement(),
        }
    }

    fn kind_after_decorators(&self) -> Option<TsSyntaxKind> {
        let mut cursor = self.cursor;
        while self
            .token_at(cursor)
            .is_some_and(|token| token.kind == TsSyntaxKind::At)
        {
            cursor += 1;
            while let Some(token) = self.token_at(cursor) {
                if matches!(
                    token.kind,
                    TsSyntaxKind::At
                        | TsSyntaxKind::KeywordClass
                        | TsSyntaxKind::KeywordFunction
                        | TsSyntaxKind::EndOfFile
                ) {
                    break;
                }
                cursor += 1;
            }
        }
        self.token_at(cursor).map(|token| token.kind)
    }

    fn kind_after_modifiers(&self) -> Option<TsSyntaxKind> {
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
        self.builder.start_schema_node(TsSyntaxKind::ModifierList);
        while is_modifier_kind(self.current()) {
            self.bump();
        }
        self.builder.finish_node();
    }

    fn expect(&mut self, kind: TsSyntaxKind, message: &str) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            self.error_here(message);
            false
        }
    }

    fn bump(&mut self) {
        if self.at(TsSyntaxKind::EndOfFile) {
            return;
        }
        let token = &self.tokens[self.cursor];
        for trivia in &token.leading {
            self.builder
                .token(trivia.kind, self.source.slice(trivia.span));
        }
        self.builder
            .schema_token(token.kind, self.source.slice(token.span));
        self.cursor += 1;
    }

    fn at(&self, kind: TsSyntaxKind) -> bool {
        self.current() == kind
    }

    fn at_any(&self, kinds: &[TsSyntaxKind]) -> bool {
        kinds.contains(&self.current())
    }

    fn next_is(&self, kind: TsSyntaxKind) -> bool {
        self.token_kind_at(1) == kind
    }

    fn jsx_enabled(&self) -> bool {
        self.jsx
    }

    fn current(&self) -> TsSyntaxKind {
        self.token_at(self.cursor)
            .map_or(TsSyntaxKind::EndOfFile, |token| token.kind)
    }

    fn token_kind_at(&self, offset: usize) -> TsSyntaxKind {
        self.token_at(self.cursor + offset)
            .map_or(TsSyntaxKind::EndOfFile, |token| token.kind)
    }

    fn token_kind_at_back(&self, offset: usize) -> TsSyntaxKind {
        self.cursor
            .checked_sub(offset)
            .and_then(|cursor| self.token_at(cursor))
            .map_or(TsSyntaxKind::EndOfFile, |token| token.kind)
    }

    fn token_at(&self, cursor: usize) -> Option<&'a Token<TsSyntaxKind>> {
        self.tokens.get(cursor)
    }

    fn has_top_level_any(&self, kinds: &[TsSyntaxKind], stops: &[TsSyntaxKind]) -> bool {
        let mut cursor = self.cursor;
        let mut paren_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut angle_depth = 0usize;
        while let Some(token) = self.token_at(cursor) {
            if token.kind == TsSyntaxKind::EndOfFile {
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
                TsSyntaxKind::OpenParen => paren_depth += 1,
                TsSyntaxKind::CloseParen if paren_depth > 0 => paren_depth -= 1,
                TsSyntaxKind::OpenBrace => brace_depth += 1,
                TsSyntaxKind::CloseBrace if brace_depth > 0 => brace_depth -= 1,
                TsSyntaxKind::OpenBracket => bracket_depth += 1,
                TsSyntaxKind::CloseBracket if bracket_depth > 0 => bracket_depth -= 1,
                TsSyntaxKind::LessThan => angle_depth += 1,
                TsSyntaxKind::GreaterThan if angle_depth > 0 => angle_depth -= 1,
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

fn is_modifier_kind(kind: TsSyntaxKind) -> bool {
    matches!(
        kind,
        TsSyntaxKind::KeywordAbstract
            | TsSyntaxKind::KeywordAsync
            | TsSyntaxKind::KeywordDeclare
            | TsSyntaxKind::KeywordOverride
            | TsSyntaxKind::KeywordPrivate
            | TsSyntaxKind::KeywordProtected
            | TsSyntaxKind::KeywordPublic
            | TsSyntaxKind::KeywordReadonly
            | TsSyntaxKind::KeywordStatic
    )
}
