use flavor_grammar::find_balanced_brace_close as find_mustache_end;

use super::{
    kind,
    names::{directive_base_len, is_attribute_name_char, is_directive_name, is_whitespace},
    parser::MarkupParser,
};

impl MarkupParser<'_> {
    pub(super) fn parse_attribute(&mut self) {
        if self.source[self.cursor..].starts_with("{...") {
            self.parse_spread_attribute();
            return;
        }
        if self.source[self.cursor..].starts_with('{') {
            self.parse_shorthand_attribute();
            return;
        }

        let name_start = self.cursor;
        while self.peek().is_some_and(is_attribute_name_char) {
            self.bump();
        }
        if self.cursor == name_start {
            self.parse_error_char();
            return;
        }

        let name = &self.source[name_start..self.cursor];
        let is_directive = is_directive_name(name);
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
    }

    fn parse_directive_name(&mut self, name: &str) {
        self.builder.start_node(kind::DIRECTIVE_NAME);
        let mut offset = directive_base_len(name);
        self.builder.token(kind::DIRECTIVE_BASE, &name[..offset]);
        if offset < name.len() && name.as_bytes()[offset] == b':' {
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
        } else if ch == '{' {
            let Some(end) = find_mustache_end(self.source, self.cursor + 1) else {
                self.cursor = self.source.len();
                self.error_at(start, "missing attribute expression close delimiter");
                if self.cursor > start {
                    self.builder
                        .token(kind::ATTRIBUTE_VALUE, &self.source[start..self.cursor]);
                }
                return;
            };
            self.cursor = end + 1;
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

    fn parse_spread_attribute(&mut self) {
        self.builder.start_node(kind::SPREAD_ATTRIBUTE);
        let start = self.cursor;
        let Some(end) = find_mustache_end(self.source, self.cursor + 1) else {
            self.cursor = self.source.len();
            self.builder
                .token(kind::EXPRESSION_TEXT, &self.source[start..]);
            self.error_at(start, "missing spread attribute close delimiter");
            self.builder.finish_node();
            return;
        };
        self.cursor = end + 1;
        self.builder
            .token(kind::EXPRESSION_TEXT, &self.source[start..self.cursor]);
        self.builder.finish_node();
    }

    fn parse_shorthand_attribute(&mut self) {
        self.builder.start_node(kind::SHORTHAND_ATTRIBUTE);
        self.parse_mustache_like(kind::MUSTACHE);
        self.builder.finish_node();
    }
}

fn scan_arg(name: &str, mut offset: usize) -> usize {
    if name[offset..].starts_with('[') {
        let mut depth = 0usize;
        while offset < name.len() {
            let byte = name.as_bytes()[offset];
            if byte == b'[' {
                depth += 1;
            } else if byte == b']' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    offset += 1;
                    break;
                }
            }
            offset += 1;
        }
        return offset;
    }

    while offset < name.len() && name.as_bytes()[offset] != b'.' {
        offset += 1;
    }
    offset
}
