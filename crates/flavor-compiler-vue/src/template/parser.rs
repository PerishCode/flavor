use flavor_compiler_core::{Diagnostic, Span, SyntaxBuilder};

use super::{
    names::{
        directive_base_len, is_attribute_name_char, is_directive_name, is_shorthand_directive,
        is_tag_name_char, is_void_tag, is_whitespace, source_char_at,
    },
    TemplateAst, VueTemplateKind,
};

pub fn parse_template(source: &str) -> TemplateAst {
    let parser = TemplateParser::new(source);
    parser.parse()
}

struct TemplateParser<'a> {
    source: &'a str,
    cursor: usize,
    builder: SyntaxBuilder,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> TemplateParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            builder: SyntaxBuilder::new(),
            diagnostics: Vec::new(),
        }
    }

    fn parse(mut self) -> TemplateAst {
        self.builder.start_node(VueTemplateKind::Root);
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
        self.builder.start_node(VueTemplateKind::Interpolation);
        self.token_len(VueTemplateKind::InterpolationOpen, 2);
        let expr_start = self.cursor;
        while self.cursor < self.source.len() && !self.source[self.cursor..].starts_with("}}") {
            self.bump();
        }
        if self.cursor > expr_start {
            self.builder.token(
                VueTemplateKind::ExpressionText,
                &self.source[expr_start..self.cursor],
            );
        }
        if self.source[self.cursor..].starts_with("}}") {
            self.token_len(VueTemplateKind::InterpolationClose, 2);
        } else {
            self.error_at(start, "missing interpolation close delimiter");
        }
        self.builder.finish_node();
    }

    fn parse_comment(&mut self) {
        let start = self.cursor;
        self.builder.start_node(VueTemplateKind::Comment);
        while self.cursor < self.source.len() && !self.source[self.cursor..].starts_with("-->") {
            self.bump();
        }
        if self.source[self.cursor..].starts_with("-->") {
            self.cursor += 3;
        } else {
            self.error_at(start, "missing HTML comment close delimiter");
        }
        self.builder
            .token(VueTemplateKind::Comment, &self.source[start..self.cursor]);
        self.builder.finish_node();
    }

    fn parse_element(&mut self) {
        let start = self.cursor;
        self.builder.start_node(VueTemplateKind::Element);
        let Some(tag) = self.parse_start_tag() else {
            self.builder.finish_node();
            return;
        };
        if tag.v_pre {
            let matched = self.parse_raw_children(&tag.name);
            if !matched {
                self.error_at(start, format!("missing closing </{}> tag", tag.name));
            }
        } else if !tag.self_closing && !is_void_tag(&tag.name) {
            let matched = self.parse_children(Some(&tag.name));
            if !matched {
                self.error_at(start, format!("missing closing </{}> tag", tag.name));
            }
        }
        self.builder.finish_node();
    }

    fn parse_start_tag(&mut self) -> Option<ParsedTag> {
        self.builder.start_node(VueTemplateKind::StartTag);
        self.token_len(VueTemplateKind::LessThan, 1);
        let name_start = self.cursor;
        while self.peek().is_some_and(is_tag_name_char) {
            self.bump();
        }
        if self.cursor == name_start {
            self.error_at(name_start.saturating_sub(1), "expected tag name");
            self.parse_bad_tag_tail();
            self.builder.finish_node();
            return None;
        }
        let name = self.source[name_start..self.cursor].to_string();
        self.builder.token(
            VueTemplateKind::TagName,
            &self.source[name_start..self.cursor],
        );
        let tail = self.parse_tag_tail();
        self.builder.finish_node();
        Some(ParsedTag {
            name,
            self_closing: tail.self_closing,
            v_pre: tail.v_pre,
        })
    }

    fn parse_end_tag(&mut self) {
        self.builder.start_node(VueTemplateKind::EndTag);
        self.token_len(VueTemplateKind::LessThan, 1);
        self.token_len(VueTemplateKind::Slash, 1);
        let name_start = self.cursor;
        while self.peek().is_some_and(is_tag_name_char) {
            self.bump();
        }
        if self.cursor == name_start {
            self.error_at(name_start.saturating_sub(2), "expected closing tag name");
        } else {
            self.builder.token(
                VueTemplateKind::TagName,
                &self.source[name_start..self.cursor],
            );
        }
        self.parse_tag_tail();
        self.builder.finish_node();
    }

    fn parse_tag_tail(&mut self) -> ParsedTagTail {
        let mut tail = ParsedTagTail::default();
        while self.cursor < self.source.len() {
            if self.source[self.cursor..].starts_with("/>") {
                self.token_len(VueTemplateKind::Slash, 1);
                self.token_len(VueTemplateKind::GreaterThan, 1);
                tail.self_closing = true;
                return tail;
            }
            if self.source[self.cursor..].starts_with('>') {
                self.token_len(VueTemplateKind::GreaterThan, 1);
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
            VueTemplateKind::Directive
        } else {
            VueTemplateKind::Attribute
        });
        if is_directive {
            self.parse_directive_name(name);
        } else {
            self.builder.token(VueTemplateKind::AttributeName, name);
        }
        while self.peek().is_some_and(is_whitespace) {
            self.parse_whitespace();
        }
        if self.source[self.cursor..].starts_with('=') {
            self.token_len(VueTemplateKind::Equals, 1);
            while self.peek().is_some_and(is_whitespace) {
                self.parse_whitespace();
            }
            if is_directive {
                self.builder
                    .start_node(VueTemplateKind::DirectiveExpression);
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
        self.builder.start_node(VueTemplateKind::DirectiveName);
        let shorthand = is_shorthand_directive(name);
        let mut offset = directive_base_len(name);
        self.builder
            .token(VueTemplateKind::DirectiveBase, &name[..offset]);
        if shorthand && offset < name.len() {
            let start = offset;
            offset = scan_arg(name, offset);
            self.builder
                .token(VueTemplateKind::DirectiveArgument, &name[start..offset]);
        } else if offset < name.len() && name.as_bytes()[offset] == b':' {
            let start = offset;
            offset += 1;
            offset = scan_arg(name, offset);
            self.builder
                .token(VueTemplateKind::DirectiveArgument, &name[start..offset]);
        }
        while offset < name.len() && name.as_bytes()[offset] == b'.' {
            let start = offset;
            offset += 1;
            while offset < name.len() && name.as_bytes()[offset] != b'.' {
                offset += 1;
            }
            self.builder
                .token(VueTemplateKind::DirectiveModifier, &name[start..offset]);
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
            self.builder.token(
                VueTemplateKind::AttributeValue,
                &self.source[start..self.cursor],
            );
        }
    }

    fn parse_whitespace(&mut self) {
        let start = self.cursor;
        while self.peek().is_some_and(is_whitespace) {
            self.bump();
        }
        self.builder.token(
            VueTemplateKind::Whitespace,
            &self.source[start..self.cursor],
        );
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
                .token(VueTemplateKind::Text, &self.source[start..self.cursor]);
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
                            .token(VueTemplateKind::Text, &self.source[start..self.cursor]);
                    }
                    self.parse_end_tag();
                    return true;
                }
            }
            self.bump();
        }
        if self.cursor > start {
            self.builder
                .token(VueTemplateKind::Text, &self.source[start..self.cursor]);
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
                .token(VueTemplateKind::Error, &self.source[start..self.cursor]);
        }
        if self.source[self.cursor..].starts_with('>') {
            self.token_len(VueTemplateKind::GreaterThan, 1);
        }
    }

    fn parse_error_char(&mut self) {
        let start = self.cursor;
        self.bump();
        self.builder
            .token(VueTemplateKind::Error, &self.source[start..self.cursor]);
        self.error_at(start, "unexpected token in tag");
    }

    fn peek_close_tag_name(&self) -> Option<String> {
        let mut cursor = self.cursor + 2;
        let start = cursor;
        while source_char_at(self.source, cursor).is_some_and(|ch| is_tag_name_char(ch.0)) {
            cursor += source_char_at(self.source, cursor)
                .map(|(_, width)| width)
                .unwrap_or(0);
        }
        if cursor > start {
            Some(self.source[start..cursor].to_string())
        } else {
            None
        }
    }

    fn error_at(&mut self, offset: usize, message: impl Into<String>) {
        let offset = u32::try_from(offset).unwrap_or(u32::MAX);
        self.diagnostics
            .push(Diagnostic::error(Span::new(offset, offset), message.into()));
    }

    fn token_len(&mut self, kind: VueTemplateKind, len: usize) {
        let start = self.cursor;
        self.cursor += len;
        self.builder.token(kind, &self.source[start..self.cursor]);
    }

    fn peek(&self) -> Option<char> {
        self.source[self.cursor..].chars().next()
    }

    fn bump(&mut self) {
        if let Some(ch) = self.peek() {
            self.cursor += ch.len_utf8();
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
