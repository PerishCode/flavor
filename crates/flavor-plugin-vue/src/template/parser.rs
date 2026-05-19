use flavor_core::{Diagnostic, Span};
use flavor_grammar::{
    find_html_comment_close, is_html_void_element, is_markup_name_char, markup_char_at,
    scan_markup_name, RawAstBuilder,
};

use super::{
    kind,
    kind::Kind,
    names::{
        directive_base_len, is_attribute_name_char, is_directive_name, is_shorthand_directive,
        is_whitespace,
    },
    TemplateAst,
};

pub fn parse_template(source: &str) -> TemplateAst {
    let parser = TemplateParser::new(source);
    parser.parse()
}

struct TemplateParser<'a> {
    source: &'a str,
    cursor: usize,
    builder: RawAstBuilder,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> TemplateParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            builder: RawAstBuilder::new(kind::schema()),
            diagnostics: Vec::new(),
        }
    }

    fn parse(mut self) -> TemplateAst {
        self.builder.start_node(kind::ROOT);
        self.parse_children(None);
        self.builder.finish_node();
        TemplateAst::new(self.builder.finish(), self.diagnostics)
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
            } else if self.source[self.cursor..].starts_with("{{") {
                self.parse_interpolation();
            } else if self.source[self.cursor..].starts_with("<!--") {
                self.parse_comment();
            } else if self.source[self.cursor..].starts_with('<') {
                self.parse_element();
            } else {
                self.parse_text();
            }
        }
        parent_tag.is_none()
    }

    fn parse_interpolation(&mut self) {
        let start = self.cursor;
        self.builder.start_node(kind::INTERPOLATION);
        self.token_len(kind::INTERPOLATION_OPEN, 2);
        let expr_start = self.cursor;
        while self.cursor < self.source.len() && !self.source[self.cursor..].starts_with("}}") {
            self.bump();
        }
        if self.cursor > expr_start {
            self.builder
                .token(kind::EXPRESSION_TEXT, &self.source[expr_start..self.cursor]);
        }
        if self.source[self.cursor..].starts_with("}}") {
            self.token_len(kind::INTERPOLATION_CLOSE, 2);
        } else {
            self.error_at(start, "missing interpolation close delimiter");
        }
        self.builder.finish_node();
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

    fn parse_element(&mut self) {
        let start = self.cursor;
        self.builder.start_node(kind::ELEMENT);
        let Some(tag) = self.parse_start_tag() else {
            self.builder.finish_node();
            return;
        };
        if tag.v_pre {
            let matched = self.parse_raw_children(&tag.name);
            if !matched {
                self.error_at(start, format!("missing closing </{}> tag", tag.name));
            }
        } else if !tag.self_closing && !is_html_void_element(&tag.name) {
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
        self.cursor = scan_markup_name(self.source, self.cursor, is_markup_name_char);
        if self.cursor == name_start {
            self.error_at(name_start.saturating_sub(1), "expected tag name");
            self.parse_bad_tag_tail();
            self.builder.finish_node();
            return None;
        }
        let name = self.source[name_start..self.cursor].to_string();
        self.builder
            .token(kind::TAG_NAME, &self.source[name_start..self.cursor]);
        let tail = self.parse_tag_tail();
        self.builder.finish_node();
        Some(ParsedTag {
            name,
            self_closing: tail.self_closing,
            v_pre: tail.v_pre,
        })
    }

    fn parse_end_tag(&mut self) {
        self.builder.start_node(kind::END_TAG);
        self.token_len(kind::LESS_THAN, 1);
        self.token_len(kind::SLASH, 1);
        let name_start = self.cursor;
        self.cursor = scan_markup_name(self.source, self.cursor, is_markup_name_char);
        if self.cursor == name_start {
            self.error_at(name_start.saturating_sub(2), "expected closing tag name");
        } else {
            self.builder
                .token(kind::TAG_NAME, &self.source[name_start..self.cursor]);
        }
        self.parse_tag_tail();
        self.builder.finish_node();
    }

    fn parse_tag_tail(&mut self) -> ParsedTagTail {
        let mut tail = ParsedTagTail::default();
        while self.cursor < self.source.len() {
            if self.source[self.cursor..].starts_with("/>") {
                self.token_len(kind::SLASH, 1);
                self.token_len(kind::GREATER_THAN, 1);
                tail.self_closing = true;
                return tail;
            }
            if self.source[self.cursor..].starts_with('>') {
                self.token_len(kind::GREATER_THAN, 1);
                return tail;
            }
            if self.peek().is_some_and(is_whitespace) {
                self.parse_whitespace();
            } else {
                tail.v_pre |= self.parse_attribute();
            }
        }
        self.error_at(self.source.len(), "missing tag close delimiter");
        tail
    }

    fn parse_attribute(&mut self) -> bool {
        let name_start = self.cursor;
        while self.peek().is_some_and(is_attribute_name_char) {
            self.bump();
        }
        if self.cursor == name_start {
            self.parse_error_char();
            return false;
        }
        let name = &self.source[name_start..self.cursor];
        let is_directive = is_directive_name(name);
        let is_v_pre = name == "v-pre";
        self.builder.start_node(if is_directive {
            kind::DIRECTIVE
        } else {
            kind::ATTRIBUTE
        });
        if is_directive {
            self.parse_directive_name(name);
        } else {
            self.builder.token(kind::ATTRIBUTE_NAME, name);
        }
        while self.peek().is_some_and(is_whitespace) {
            self.parse_whitespace();
        }
        if self.source[self.cursor..].starts_with('=') {
            self.token_len(kind::EQUALS, 1);
            while self.peek().is_some_and(is_whitespace) {
                self.parse_whitespace();
            }
            if is_directive {
                self.builder.start_node(kind::DIRECTIVE_EXPRESSION);
                self.parse_attribute_value();
                self.builder.finish_node();
            } else {
                self.parse_attribute_value();
            }
        }
        self.builder.finish_node();
        is_v_pre
    }

    fn parse_directive_name(&mut self, name: &str) {
        self.builder.start_node(kind::DIRECTIVE_NAME);
        let shorthand = is_shorthand_directive(name);
        let mut offset = directive_base_len(name);
        self.builder.token(kind::DIRECTIVE_BASE, &name[..offset]);
        if shorthand && offset < name.len() {
            let start = offset;
            offset = scan_arg(name, offset);
            self.builder
                .token(kind::DIRECTIVE_ARGUMENT, &name[start..offset]);
        } else if offset < name.len() && name.as_bytes()[offset] == b':' {
            let start = offset;
            offset += 1;
            offset = scan_arg(name, offset);
            self.builder
                .token(kind::DIRECTIVE_ARGUMENT, &name[start..offset]);
        }
        while offset < name.len() && name.as_bytes()[offset] == b'.' {
            let start = offset;
            offset += 1;
            while offset < name.len() && name.as_bytes()[offset] != b'.' {
                offset += 1;
            }
            self.builder
                .token(kind::DIRECTIVE_MODIFIER, &name[start..offset]);
        }
        self.builder.finish_node();
    }

    fn parse_attribute_value(&mut self) {
        let start = self.cursor;
        let Some(ch) = self.peek() else {
            return;
        };
        if ch == '"' || ch == '\'' {
            self.bump();
            while let Some(value_ch) = self.peek() {
                self.bump();
                if value_ch == ch {
                    break;
                }
            }
            if !self.source[start..self.cursor].ends_with(ch) {
                self.error_at(start, "missing attribute value quote");
            }
        } else {
            while self.cursor < self.source.len()
                && !self.peek().is_some_and(is_whitespace)
                && !self.source[self.cursor..].starts_with('>')
                && !self.source[self.cursor..].starts_with("/>")
            {
                self.bump();
            }
        }
        if self.cursor > start {
            self.builder
                .token(kind::ATTRIBUTE_VALUE, &self.source[start..self.cursor]);
        }
    }

    fn parse_whitespace(&mut self) {
        let start = self.cursor;
        while self.peek().is_some_and(is_whitespace) {
            self.bump();
        }
        self.builder
            .token(kind::WHITESPACE, &self.source[start..self.cursor]);
    }

    fn parse_text(&mut self) {
        let start = self.cursor;
        while self.cursor < self.source.len()
            && !self.source[self.cursor..].starts_with('<')
            && !self.source[self.cursor..].starts_with("{{")
        {
            self.bump();
        }
        if self.cursor > start {
            self.builder
                .token(kind::TEXT, &self.source[start..self.cursor]);
        }
    }

    fn parse_raw_children(&mut self, parent_tag: &str) -> bool {
        let start = self.cursor;
        while self.cursor < self.source.len() {
            if self.source[self.cursor..].starts_with("</") {
                let name = self.peek_close_tag_name();
                if name.as_deref() == Some(parent_tag) {
                    if self.cursor > start {
                        self.builder
                            .token(kind::TEXT, &self.source[start..self.cursor]);
                    }
                    self.parse_end_tag();
                    return true;
                }
            }
            self.bump();
        }
        if self.cursor > start {
            self.builder
                .token(kind::TEXT, &self.source[start..self.cursor]);
        }
        false
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

    fn parse_error_char(&mut self) {
        let start = self.cursor;
        self.bump();
        self.builder
            .token(kind::ERROR, &self.source[start..self.cursor]);
        self.error_at(start, "unexpected token in tag");
    }

    fn peek_close_tag_name(&self) -> Option<String> {
        let start = self.cursor + 2;
        let cursor = scan_markup_name(self.source, start, is_markup_name_char);
        (cursor > start).then(|| self.source[start..cursor].to_string())
    }

    fn error_at(&mut self, offset: usize, message: impl Into<String>) {
        let offset = u32::try_from(offset).unwrap_or(u32::MAX);
        self.diagnostics.push(Diagnostic::error_code(
            Span::new(offset, offset),
            "vue/parse/error",
            message.into(),
        ));
    }

    fn token_len(&mut self, kind: Kind, len: usize) {
        let start = self.cursor;
        self.cursor += len;
        self.builder.token(kind, &self.source[start..self.cursor]);
    }

    fn peek(&self) -> Option<char> {
        markup_char_at(self.source, self.cursor).map(|(ch, _)| ch)
    }

    fn bump(&mut self) {
        if let Some((_, width)) = markup_char_at(self.source, self.cursor) {
            self.cursor += width;
        }
    }
}

struct ParsedTag {
    name: String,
    self_closing: bool,
    v_pre: bool,
}

#[derive(Default)]
struct ParsedTagTail {
    self_closing: bool,
    v_pre: bool,
}

fn scan_arg(name: &str, mut offset: usize) -> usize {
    if name.as_bytes().get(offset) == Some(&b'[') {
        offset += 1;
        while offset < name.len() {
            let value = name.as_bytes()[offset];
            offset += 1;
            if value == b']' {
                break;
            }
        }
        return offset;
    }
    while offset < name.len() && name.as_bytes()[offset] != b'.' {
        offset += 1;
    }
    offset
}
