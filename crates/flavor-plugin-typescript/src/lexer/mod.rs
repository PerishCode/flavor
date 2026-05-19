mod kinds;

use flavor_core::{SourceText, Span, Token, Trivia, TriviaKind};

use crate::{
    internal::grammar::{self as kind, Kind},
    state::TsPluginConfig,
};

use self::kinds::{
    is_bin_digit, is_hex_digit, is_identifier_part, is_identifier_start, is_oct_digit,
    is_regex_prefix, is_whitespace, keyword_kind, punctuators,
};

pub fn scan(source: &SourceText, _config: &TsPluginConfig) -> Vec<Token<Kind>> {
    Scanner::new(source.as_str()).scan()
}

struct Scanner<'a> {
    source: &'a str,
    cursor: usize,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, cursor: 0 }
    }

    fn scan(mut self) -> Vec<Token<Kind>> {
        let mut tokens = Vec::new();
        let mut previous = None;
        loop {
            let leading = self.collect_leading_trivia();
            let token = self.next_token(leading, previous);
            let at_end = token.kind == kind::END_OF_FILE;
            if !at_end {
                previous = Some(token.kind);
            }
            tokens.push(token);
            if at_end {
                break;
            }
        }
        tokens
    }

    fn collect_leading_trivia(&mut self) -> Vec<Trivia> {
        let mut trivia = Vec::new();
        loop {
            let start = self.cursor;
            if self.cursor == 0 && self.source[self.cursor..].starts_with("#!") {
                self.scan_line();
                trivia.push(Trivia::new(TriviaKind::Shebang, span(start, self.cursor)));
                continue;
            }
            if self.peek().is_some_and(is_whitespace) {
                self.scan_while(is_whitespace);
                trivia.push(Trivia::new(
                    TriviaKind::Whitespace,
                    span(start, self.cursor),
                ));
                continue;
            }
            if self.source[self.cursor..].starts_with("//") {
                self.scan_line();
                trivia.push(Trivia::new(
                    TriviaKind::LineComment,
                    span(start, self.cursor),
                ));
                continue;
            }
            if self.source[self.cursor..].starts_with("/*") {
                self.scan_block_comment();
                trivia.push(Trivia::new(
                    TriviaKind::BlockComment,
                    span(start, self.cursor),
                ));
                continue;
            }
            break;
        }
        trivia
    }

    fn next_token(&mut self, leading: Vec<Trivia>, previous: Option<Kind>) -> Token<Kind> {
        let start = self.cursor;
        let Some(ch) = self.peek() else {
            return token(kind::END_OF_FILE, start, start, leading);
        };

        if is_identifier_start(ch) {
            self.scan_while(is_identifier_part);
            let text = &self.source[start..self.cursor];
            return token(keyword_kind(text), start, self.cursor, leading);
        }

        if ch.is_ascii_digit() {
            let kind = self.scan_number();
            return token(kind, start, self.cursor, leading);
        }

        if ch == '.' && self.peek_n(1).is_some_and(|value| value.is_ascii_digit()) {
            self.bump();
            self.scan_digits(|value| value.is_ascii_digit());
            self.scan_exponent();
            return token(kind::NUMERIC_LITERAL, start, self.cursor, leading);
        }

        if ch == '"' || ch == '\'' {
            self.scan_string(ch);
            return token(kind::STRING_LITERAL, start, self.cursor, leading);
        }

        if ch == '`' {
            self.scan_template();
            return token(kind::TEMPLATE_LITERAL, start, self.cursor, leading);
        }

        if ch == '/' && self.can_start_regex(previous) {
            self.scan_regex();
            return token(kind::REGEX_LITERAL, start, self.cursor, leading);
        }

        if let Some(kind) = self.scan_punctuator() {
            return token(kind, start, self.cursor, leading);
        }

        let kind = match ch {
            '(' => kind::OPEN_PAREN,
            ')' => kind::CLOSE_PAREN,
            '{' => kind::OPEN_BRACE,
            '}' => kind::CLOSE_BRACE,
            '[' => kind::OPEN_BRACKET,
            ']' => kind::CLOSE_BRACKET,
            '<' => kind::LESS_THAN,
            '>' => kind::GREATER_THAN,
            '/' => kind::SLASH,
            '+' => kind::PLUS,
            '-' => kind::MINUS,
            '*' => kind::STAR,
            '=' => kind::EQUALS,
            ';' => kind::SEMICOLON,
            ':' => kind::COLON,
            ',' => kind::COMMA,
            '.' => kind::DOT,
            '@' => kind::AT,
            '?' => kind::QUESTION,
            '!' => kind::BANG,
            '|' => kind::PIPE,
            '&' => kind::AMPERSAND,
            '%' => kind::PERCENT,
            _ => kind::UNKNOWN,
        };
        self.bump();
        token(kind, start, self.cursor, leading)
    }

    fn scan_string(&mut self, quote: char) {
        self.bump();
        while let Some(ch) = self.peek() {
            self.bump();
            if ch == '\\' {
                self.bump();
                continue;
            }
            if ch == quote {
                break;
            }
        }
    }

    fn scan_number(&mut self) -> Kind {
        if self.peek() == Some('0') {
            match self.peek_n(1) {
                Some('x' | 'X') => return self.scan_radix(is_hex_digit),
                Some('b' | 'B') => return self.scan_radix(is_bin_digit),
                Some('o' | 'O') => return self.scan_radix(is_oct_digit),
                _ => {}
            }
        }

        self.scan_digits(|ch| ch.is_ascii_digit());
        let mut decimal_only = true;
        if self.peek() == Some('.') && !self.source[self.cursor..].starts_with("...") {
            self.bump();
            self.scan_digits(|ch| ch.is_ascii_digit());
            decimal_only = false;
        }
        if self.scan_exponent() {
            decimal_only = false;
        }
        if decimal_only && self.peek() == Some('n') {
            self.bump();
            kind::BIG_INT_LITERAL
        } else {
            kind::NUMERIC_LITERAL
        }
    }

    fn scan_radix(&mut self, predicate: impl Fn(char) -> bool) -> Kind {
        self.bump();
        self.bump();
        self.scan_digits(predicate);
        if self.peek() == Some('n') {
            self.bump();
            kind::BIG_INT_LITERAL
        } else {
            kind::NUMERIC_LITERAL
        }
    }

    fn scan_digits(&mut self, predicate: impl Fn(char) -> bool) {
        self.scan_while(|ch| predicate(ch) || ch == '_');
    }

    fn scan_exponent(&mut self) -> bool {
        if !matches!(self.peek(), Some('e' | 'E')) {
            return false;
        }
        self.bump();
        if matches!(self.peek(), Some('+' | '-')) {
            self.bump();
        }
        self.scan_digits(|ch| ch.is_ascii_digit());
        true
    }

    fn scan_template(&mut self) {
        self.bump();
        let mut brace_depth = 0usize;
        while let Some(ch) = self.peek() {
            if ch == '\\' {
                self.bump();
                self.bump();
                continue;
            }
            if brace_depth == 0 {
                if self.source[self.cursor..].starts_with("${") {
                    self.bump();
                    self.bump();
                    brace_depth = 1;
                    continue;
                }
                self.bump();
                if ch == '`' {
                    break;
                }
                continue;
            }
            match ch {
                '"' | '\'' => self.scan_string(ch),
                '`' => self.scan_template(),
                '/' if self.source[self.cursor..].starts_with("//") => self.scan_line(),
                '/' if self.source[self.cursor..].starts_with("/*") => self.scan_block_comment(),
                '{' => {
                    brace_depth += 1;
                    self.bump();
                }
                '}' => {
                    brace_depth -= 1;
                    self.bump();
                }
                _ => self.bump(),
            }
        }
    }

    fn can_start_regex(&self, previous: Option<Kind>) -> bool {
        if self.source[self.cursor..].starts_with("//")
            || self.source[self.cursor..].starts_with("/*")
            || self.source[self.cursor..].starts_with("/=")
        {
            return false;
        }

        previous.is_none_or(is_regex_prefix)
    }

    fn scan_regex(&mut self) {
        self.bump();
        let mut in_class = false;
        while let Some(ch) = self.peek() {
            self.bump();
            if ch == '\\' {
                self.bump();
                continue;
            }
            if matches!(ch, '\n' | '\r') {
                break;
            }
            if ch == '[' {
                in_class = true;
                continue;
            }
            if ch == ']' {
                in_class = false;
                continue;
            }
            if ch == '/' && !in_class {
                break;
            }
        }
        self.scan_while(is_identifier_part);
    }

    fn scan_line(&mut self) {
        while let Some(ch) = self.peek() {
            self.bump();
            if ch == '\n' {
                break;
            }
        }
    }

    fn scan_block_comment(&mut self) {
        self.bump();
        self.bump();
        while self.cursor < self.source.len() {
            if self.source[self.cursor..].starts_with("*/") {
                self.bump();
                self.bump();
                break;
            }
            self.bump();
        }
    }

    fn scan_punctuator(&mut self) -> Option<Kind> {
        for (text, kind) in punctuators() {
            if self.source[self.cursor..].starts_with(text) {
                self.cursor += text.len();
                return Some(*kind);
            }
        }
        None
    }

    fn scan_while(&mut self, predicate: impl Fn(char) -> bool) {
        while self.peek().is_some_and(&predicate) {
            self.bump();
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.cursor..].chars().next()
    }

    fn peek_n(&self, offset: usize) -> Option<char> {
        self.source[self.cursor..].chars().nth(offset)
    }

    fn bump(&mut self) {
        if let Some(ch) = self.peek() {
            self.cursor += ch.len_utf8();
        }
    }
}

fn token(kind: Kind, start: usize, end: usize, leading: Vec<Trivia>) -> Token<Kind> {
    Token {
        kind,
        span: span(start, end),
        leading,
        trailing: Vec::new(),
    }
}

fn span(start: usize, end: usize) -> Span {
    Span::new(start as u32, end as u32)
}
