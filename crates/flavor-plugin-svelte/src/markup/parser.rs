use flavor_core::{Diagnostic, Span};
use flavor_grammar::{
    find_balanced_brace_close as find_mustache_end, find_html_comment_close, is_html_void_element,
    markup_char_at, scan_markup_name, RawAstBuilder,
};

use super::{
    kind,
    kind::Kind,
    names::{is_component_tag, is_tag_name_char, is_whitespace},
    SvelteMarkupAst,
};

pub fn parse_markup(source: &str) -> SvelteMarkupAst {
    MarkupParser::new(source).parse()
}

pub(super) struct MarkupParser<'a> {
    pub(super) source: &'a str,
    pub(super) cursor: usize,
    pub(super) builder: RawAstBuilder<'static>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
struct ParsedTag {
    name: String,
    self_closing: bool,
}

impl<'a> MarkupParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            builder: RawAstBuilder::new(kind::schema()),
            diagnostics: Vec::new(),
        }
    }

    fn parse(mut self) -> SvelteMarkupAst {
        self.builder.start_node(kind::ROOT);
        self.parse_children(None);
        self.builder.finish_node();
        SvelteMarkupAst::new(self.builder.finish(), self.diagnostics)
    }

    fn parse_children(&mut self, parent_tag: Option<&str>) -> bool {
        while self.cursor < self.source.len() {
            if self.source[self.cursor..].starts_with("</") {
                let name = self.peek_close_tag_name();
                if parent_tag.is_some_and(|parent| name.as_deref() == Some(parent)) {
                    self.parse_end_tag();
                    return true;
                }
                if parent_tag.is_some() {
                    return false;
                }
                let start = self.cursor;
                self.parse_end_tag();
                self.error_at(start, "unexpected closing tag");
            } else if self.source[self.cursor..].starts_with("{/") {
                if parent_tag.is_some() {
                    return false;
                }
                self.parse_unexpected_block_close();
            } else {
                self.parse_child();
            }
        }
        parent_tag.is_none()
    }

    fn parse_block_children(&mut self, keyword: &str) -> bool {
        while self.cursor < self.source.len() {
            if self.source[self.cursor..].starts_with("{/") {
                let close_keyword = self.peek_block_keyword(2);
                if close_keyword.as_deref() == Some(keyword) {
                    self.parse_block_tag(kind::BLOCK_CLOSE, 2);
                    return true;
                }
                return false;
            }
            if self.source[self.cursor..].starts_with("{:") {
                self.parse_block_tag(kind::BLOCK_BRANCH, 2);
            } else {
                self.parse_child();
            }
        }
        false
    }

    fn parse_child(&mut self) {
        if self.source[self.cursor..].starts_with("<!--") {
            self.parse_comment();
        } else if self.source[self.cursor..].starts_with('<') {
            self.parse_element();
        } else if self.source[self.cursor..].starts_with("{#") {
            self.parse_block();
        } else if self.source[self.cursor..].starts_with("{@") {
            self.parse_special_tag();
        } else if self.source[self.cursor..].starts_with("{:") {
            let start = self.cursor;
            self.parse_block_tag(kind::BLOCK_BRANCH, 2);
            self.error_at(start, "unexpected block branch");
        } else if self.source[self.cursor..].starts_with('{') {
            self.parse_mustache();
        } else {
            self.parse_text();
        }
    }

    fn parse_element(&mut self) {
        let start = self.cursor;
        let Some(tag_name) = self.peek_open_tag_name() else {
            self.parse_error_char();
            return;
        };
        self.builder.start_node(if is_component_tag(&tag_name) {
            kind::COMPONENT
        } else {
            kind::ELEMENT
        });
        let Some(tag) = self.parse_start_tag() else {
            self.builder.finish_node();
            return;
        };
        if !tag.self_closing && !is_html_void_element(&tag.name) {
            let matched = self.parse_children(Some(&tag.name));
            if !matched {
                self.error_at(start, format!("missing closing </{}> tag", tag.name));
            }
        }
        self.builder.finish_node();
    }

    fn parse_start_tag(&mut self) -> Option<ParsedTag> {
        self.builder.start_node(kind::START_TAG);
        self.token_len(kind::LESS_THAN, 1);
        let name_start = self.cursor;
        self.cursor = scan_markup_name(self.source, self.cursor, is_tag_name_char);
        if self.cursor == name_start {
            self.error_at(name_start.saturating_sub(1), "expected tag name");
            self.parse_bad_tag_tail();
            self.builder.finish_node();
            return None;
        }
        let name = self.source[name_start..self.cursor].to_string();
        self.builder
            .token(kind::TAG_NAME, &self.source[name_start..self.cursor]);
        let self_closing = self.parse_tag_tail();
        self.builder.finish_node();
        Some(ParsedTag { name, self_closing })
    }

    fn parse_end_tag(&mut self) {
        self.builder.start_node(kind::END_TAG);
        self.token_len(kind::LESS_THAN, 1);
        self.token_len(kind::SLASH, 1);
        let name_start = self.cursor;
        self.cursor = scan_markup_name(self.source, self.cursor, is_tag_name_char);
        if self.cursor == name_start {
            self.error_at(name_start.saturating_sub(2), "expected closing tag name");
        } else {
            self.builder
                .token(kind::TAG_NAME, &self.source[name_start..self.cursor]);
        }
        self.parse_tag_tail();
        self.builder.finish_node();
    }

    fn parse_tag_tail(&mut self) -> bool {
        while self.cursor < self.source.len() {
            if self.source[self.cursor..].starts_with("/>") {
                self.token_len(kind::SLASH, 1);
                self.token_len(kind::GREATER_THAN, 1);
                return true;
            }
            if self.source[self.cursor..].starts_with('>') {
                self.token_len(kind::GREATER_THAN, 1);
                return false;
            }
            if self.peek().is_some_and(is_whitespace) {
                self.parse_whitespace();
            } else {
                self.parse_attribute();
            }
        }
        self.error_at(self.source.len(), "missing tag close delimiter");
        false
    }

    fn parse_block(&mut self) {
        let start = self.cursor;
        self.builder.start_node(kind::BLOCK);
        let keyword = self.parse_block_tag(kind::BLOCK_OPEN, 2);
        if let Some(keyword) = keyword {
            if !self.parse_block_children(&keyword) {
                self.error_at(start, format!("missing closing {{/{keyword}}} block"));
            }
        }
        self.builder.finish_node();
    }

    fn parse_block_tag(&mut self, kind: Kind, opener_len: usize) -> Option<String> {
        self.builder.start_node(kind);
        self.token_len(kind::MUSTACHE_OPEN, opener_len);
        let keyword_start = self.cursor;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.bump();
        }
        let keyword = if self.cursor > keyword_start {
            let keyword = self.source[keyword_start..self.cursor].to_string();
            self.builder.token(
                kind::BLOCK_KEYWORD,
                &self.source[keyword_start..self.cursor],
            );
            Some(keyword)
        } else {
            self.error_at(
                keyword_start.saturating_sub(opener_len),
                "expected block keyword",
            );
            None
        };
        self.parse_expression_tail();
        self.builder.finish_node();
        keyword
    }

    fn parse_render_tag(&mut self) {
        self.builder.start_node(kind::RENDER_TAG);
        self.token_len(kind::MUSTACHE_OPEN, 2);
        let keyword_start = self.cursor;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.bump();
        }
        if self.cursor > keyword_start {
            self.builder.token(
                kind::BLOCK_KEYWORD,
                &self.source[keyword_start..self.cursor],
            );
        }
        self.parse_expression_tail();
        self.builder.finish_node();
    }

    fn parse_special_tag(&mut self) {
        if self.peek_block_keyword(2).as_deref() == Some("render") {
            self.parse_render_tag();
            return;
        }
        self.parse_block_tag(kind::SPECIAL_TAG, 2);
    }

    fn parse_unexpected_block_close(&mut self) {
        let start = self.cursor;
        self.parse_block_tag(kind::BLOCK_CLOSE, 2);
        self.error_at(start, "unexpected block close");
    }

    fn parse_mustache(&mut self) {
        self.parse_mustache_like(kind::MUSTACHE);
    }

    pub(super) fn parse_mustache_like(&mut self, kind: Kind) {
        let start = self.cursor;
        self.builder.start_node(kind);
        self.token_len(kind::MUSTACHE_OPEN, 1);
        let expr_start = self.cursor;
        let end = find_mustache_end(self.source, expr_start);
        match end {
            Some(end) => {
                if end > expr_start {
                    self.builder
                        .token(kind::EXPRESSION_TEXT, &self.source[expr_start..end]);
                }
                self.cursor = end;
                self.token_len(kind::MUSTACHE_CLOSE, 1);
            }
            None => {
                if self.cursor < self.source.len() {
                    self.cursor = self.source.len();
                    self.builder
                        .token(kind::EXPRESSION_TEXT, &self.source[expr_start..]);
                }
                self.error_at(start, "missing mustache close delimiter");
            }
        }
        self.builder.finish_node();
    }

    fn parse_expression_tail(&mut self) {
        let expr_start = self.cursor;
        let end = find_mustache_end(self.source, expr_start);
        match end {
            Some(end) => {
                if end > expr_start {
                    self.builder
                        .token(kind::EXPRESSION_TEXT, &self.source[expr_start..end]);
                }
                self.cursor = end;
                self.token_len(kind::MUSTACHE_CLOSE, 1);
            }
            None => {
                if self.cursor < self.source.len() {
                    self.cursor = self.source.len();
                    self.builder
                        .token(kind::EXPRESSION_TEXT, &self.source[expr_start..]);
                }
                self.error_at(expr_start, "missing block close delimiter");
            }
        }
    }

    fn parse_comment(&mut self) {
        let start = self.cursor;
        self.builder.start_node(kind::COMMENT);
        match find_html_comment_close(self.source, self.cursor) {
            Some(end) => self.cursor = end,
            None => {
                self.cursor = self.source.len();
                self.error_at(start, "missing HTML comment close delimiter");
            }
        }
        self.builder
            .token(kind::COMMENT_TEXT, &self.source[start..self.cursor]);
        self.builder.finish_node();
    }

    fn parse_text(&mut self) {
        let start = self.cursor;
        while self.cursor < self.source.len()
            && !self.source[self.cursor..].starts_with('<')
            && !self.source[self.cursor..].starts_with('{')
        {
            self.bump();
        }
        if self.cursor > start {
            self.builder
                .token(kind::TEXT, &self.source[start..self.cursor]);
        }
    }

    pub(super) fn parse_whitespace(&mut self) {
        let start = self.cursor;
        while self.peek().is_some_and(is_whitespace) {
            self.bump();
        }
        self.builder
            .token(kind::WHITESPACE, &self.source[start..self.cursor]);
    }

    fn parse_bad_tag_tail(&mut self) {
        let start = self.cursor;
        while self.cursor < self.source.len() && !self.source[self.cursor..].starts_with('>') {
            self.bump();
        }
        if self.cursor > start {
            self.builder
                .token(kind::ERROR, &self.source[start..self.cursor]);
        }
        if self.source[self.cursor..].starts_with('>') {
            self.token_len(kind::GREATER_THAN, 1);
        }
    }

    pub(super) fn parse_error_char(&mut self) {
        let start = self.cursor;
        self.bump();
        self.builder
            .token(kind::ERROR, &self.source[start..self.cursor]);
        self.error_at(start, "unexpected character");
    }

    fn peek_open_tag_name(&self) -> Option<String> {
        if !self.source[self.cursor..].starts_with('<')
            || self.source[self.cursor..].starts_with("</")
        {
            return None;
        }
        let start = self.cursor + 1;
        let cursor = scan_markup_name(self.source, start, is_tag_name_char);
        (cursor > start).then(|| self.source[start..cursor].to_string())
    }

    fn peek_close_tag_name(&self) -> Option<String> {
        if !self.source[self.cursor..].starts_with("</") {
            return None;
        }
        let start = self.cursor + 2;
        let cursor = scan_markup_name(self.source, start, is_tag_name_char);
        (cursor > start).then(|| self.source[start..cursor].to_string())
    }

    fn peek_block_keyword(&self, opener_len: usize) -> Option<String> {
        let start = self.cursor + opener_len;
        let cursor = scan_markup_name(self.source, start, |ch| ch.is_ascii_alphabetic());
        (cursor > start).then(|| self.source[start..cursor].to_string())
    }

    pub(super) fn token_len(&mut self, kind: Kind, len: usize) {
        let start = self.cursor;
        self.cursor += len;
        self.builder.token(kind, &self.source[start..self.cursor]);
    }

    pub(super) fn peek(&self) -> Option<char> {
        markup_char_at(self.source, self.cursor).map(|(ch, _)| ch)
    }

    pub(super) fn bump(&mut self) {
        if let Some((_, width)) = markup_char_at(self.source, self.cursor) {
            self.cursor += width;
        }
    }

    pub(super) fn error_at(&mut self, offset: usize, message: impl Into<String>) {
        let offset = u32::try_from(offset).unwrap_or(u32::MAX);
        self.diagnostics.push(Diagnostic::error_code(
            Some(Span::new(offset, offset)),
            "svelte/parse/error",
            message.into(),
        ));
    }
}
