pub mod plugin;

mod internal;
mod model;

use flavor_core::{Diagnostic, SourceText, Span};
use flavor_grammar::RawAstBuilder;

pub use model::{
    PythonAnalysisOutput, PythonDispatchBranchFact, PythonFacts, PythonFunctionBodyFact,
    PythonNameFact, PythonNameKind,
};

use crate::internal::grammar;

pub fn run(source: SourceText) -> PythonAnalysisOutput {
    let syntax = raw_syntax(&source);
    let mut analyzer = Analyzer::new(&source);
    analyzer.analyze();
    PythonAnalysisOutput {
        source,
        syntax,
        facts: analyzer.facts,
        diagnostics: analyzer.diagnostics,
    }
}

fn raw_syntax(source: &SourceText) -> flavor_core::SyntaxNode {
    let mut builder = RawAstBuilder::new(grammar::schema());
    builder.start_node(grammar::SOURCE_FILE);
    if !source.is_empty() {
        builder.token(grammar::RAW_TEXT, source.as_str());
    }
    builder.finish_node();
    builder.finish()
}

struct Analyzer {
    lines: Vec<LineInfo>,
    facts: PythonFacts,
    diagnostics: Vec<Diagnostic>,
}

impl Analyzer {
    fn new(source: &SourceText) -> Self {
        Self {
            lines: source_lines(source.as_str()),
            facts: PythonFacts::default(),
            diagnostics: Vec::new(),
        }
    }

    fn analyze(&mut self) {
        let mut class_stack: Vec<BlockScope> = Vec::new();
        for index in 0..self.lines.len() {
            if !self.lines[index].significant {
                continue;
            }
            while class_stack
                .last()
                .is_some_and(|scope| self.lines[index].indent <= scope.indent)
            {
                class_stack.pop();
            }

            if let Some(class) = parse_class_header(&self.lines[index]) {
                self.check_block_header(index, "class definition");
                class_stack.push(BlockScope {
                    indent: self.lines[index].indent,
                    end_index: self.block_end_index(index),
                    _name: class.name,
                });
                continue;
            }

            if let Some(function) = parse_function_header(&self.lines[index]) {
                let in_class = class_stack.iter().any(|scope| {
                    index <= scope.end_index && self.lines[index].indent > scope.indent
                });
                self.push_function(index, function, in_class);
                continue;
            }

            if is_branch_header(self.lines[index].trimmed.as_str()) {
                self.push_branch(index);
                continue;
            }

            if let Some(binding) = parse_binding(&self.lines[index]) {
                self.facts.names.push(PythonNameFact {
                    kind: PythonNameKind::Binding,
                    name: binding.name,
                    span: binding.span,
                    line: self.lines[index].number,
                });
            }
        }
    }

    fn push_function(&mut self, index: usize, function: FunctionHeader, in_class: bool) {
        self.check_block_header(index, "function definition");
        let end_index = self.block_end_index(index);
        let line = &self.lines[index];
        let kind = if in_class {
            PythonNameKind::Method
        } else {
            PythonNameKind::Function
        };
        self.facts.names.push(PythonNameFact {
            kind,
            name: function.name.clone(),
            span: function.name_span,
            line: line.number,
        });
        self.facts.function_bodies.push(PythonFunctionBodyFact {
            name: function.name,
            kind: if in_class { "method" } else { "function" },
            span: self.block_span(index, end_index),
            line: line.number,
            lines: self.lines[end_index].number.saturating_sub(line.number) + 1,
        });
        for parameter in function.parameters {
            self.facts.names.push(PythonNameFact {
                kind: PythonNameKind::Parameter,
                name: parameter.name,
                span: parameter.span,
                line: line.number,
            });
        }
    }

    fn push_branch(&mut self, index: usize) {
        self.check_block_header(index, "branch statement");
        let end_index = self.block_end_index(index);
        self.facts.dispatch_branches.push(PythonDispatchBranchFact {
            span: self.block_span(index, end_index),
            line: self.lines[index].number,
            lines: self.lines[end_index]
                .number
                .saturating_sub(self.lines[index].number)
                + 1,
        });
    }

    fn check_block_header(&mut self, index: usize, label: &str) {
        let line = &self.lines[index];
        if !line.trimmed.ends_with(':') && balanced_parens(line.trimmed.as_str()) {
            self.diagnostics.push(Diagnostic::error(
                Some(line.content_span()),
                format!("Python {label} should end with `:`"),
            ));
            return;
        }
        let Some(next) = self.next_significant(index + 1) else {
            self.diagnostics.push(Diagnostic::error(
                Some(line.content_span()),
                format!("Python {label} has no indented body"),
            ));
            return;
        };
        if self.lines[next].indent <= line.indent {
            self.diagnostics.push(Diagnostic::error(
                Some(line.content_span()),
                format!("Python {label} has no indented body"),
            ));
        }
    }

    fn next_significant(&self, start: usize) -> Option<usize> {
        (start..self.lines.len()).find(|index| self.lines[*index].significant)
    }

    fn block_end_index(&self, start: usize) -> usize {
        let indent = self.lines[start].indent;
        let mut end = start;
        for index in start + 1..self.lines.len() {
            if !self.lines[index].significant {
                continue;
            }
            if self.lines[index].indent <= indent {
                break;
            }
            end = index;
        }
        end
    }

    fn block_span(&self, start: usize, end: usize) -> Span {
        Span::from_usize(self.lines[start].start, self.lines[end].end)
    }
}

#[derive(Debug, Clone)]
struct LineInfo {
    number: usize,
    start: usize,
    end: usize,
    content_start: usize,
    indent: usize,
    trimmed: String,
    significant: bool,
}

impl LineInfo {
    fn content_span(&self) -> Span {
        Span::from_usize(self.content_start, self.end)
    }
}

#[derive(Debug, Clone)]
struct BlockScope {
    indent: usize,
    end_index: usize,
    _name: String,
}

#[derive(Debug, Clone)]
struct ClassHeader {
    name: String,
}

#[derive(Debug, Clone)]
struct FunctionHeader {
    name: String,
    name_span: Span,
    parameters: Vec<ParameterFact>,
}

#[derive(Debug, Clone)]
struct ParameterFact {
    name: String,
    span: Span,
}

#[derive(Debug, Clone)]
struct BindingFact {
    name: String,
    span: Span,
}

fn source_lines(source: &str) -> Vec<LineInfo> {
    let mut result = Vec::new();
    let mut offset = 0usize;
    let mut triple_quote: Option<&'static str> = None;
    for (index, raw) in source.split_inclusive('\n').enumerate() {
        let text = raw.trim_end_matches(['\r', '\n']);
        let string_continuation = triple_quote.is_some();
        if let Some(quote) = triple_quote {
            if text.contains(quote) {
                triple_quote = None;
            }
        } else if let Some((quote, start)) = first_triple_quote(text) {
            if !text[start + quote.len()..].contains(quote) {
                triple_quote = Some(quote);
            }
        }
        result.push(line_info(index + 1, offset, text, !string_continuation));
        offset += raw.len();
    }
    if !source.is_empty() && !source.ends_with('\n') && result.is_empty() {
        result.push(line_info(1, 0, source, true));
    }
    result
}

fn first_triple_quote(text: &str) -> Option<(&'static str, usize)> {
    match (text.find("\"\"\""), text.find("'''")) {
        (Some(double), Some(single)) => Some(if double < single {
            ("\"\"\"", double)
        } else {
            ("'''", single)
        }),
        (Some(double), None) => Some(("\"\"\"", double)),
        (None, Some(single)) => Some(("'''", single)),
        (None, None) => None,
    }
}

fn line_info(number: usize, start: usize, text: &str, allow_significant: bool) -> LineInfo {
    let indent_bytes = text
        .char_indices()
        .find_map(|(index, ch)| (!matches!(ch, ' ' | '\t')).then_some(index))
        .unwrap_or(text.len());
    let indent = text[..indent_bytes]
        .chars()
        .map(|ch| if ch == '\t' { 4 } else { 1 })
        .sum();
    let trimmed = text[indent_bytes..].trim_end().to_string();
    let significant = allow_significant && !trimmed.is_empty() && !trimmed.starts_with('#');
    LineInfo {
        number,
        start,
        end: start + text.len(),
        content_start: start + indent_bytes,
        indent,
        trimmed,
        significant,
    }
}

fn parse_class_header(line: &LineInfo) -> Option<ClassHeader> {
    let rest = keyword_rest(line.trimmed.as_str(), "class")?.trim_start();
    let (name, _) = parse_identifier(rest)?;
    Some(ClassHeader {
        name: name.to_string(),
    })
}

fn parse_function_header(line: &LineInfo) -> Option<FunctionHeader> {
    let (rest, prefix_len) = if let Some(rest) = keyword_rest(line.trimmed.as_str(), "async") {
        let rest = rest.trim_start();
        let async_gap = line.trimmed.len() - rest.len();
        let rest = keyword_rest(rest, "def")?;
        (rest, async_gap + "def".len())
    } else {
        (keyword_rest(line.trimmed.as_str(), "def")?, "def".len())
    };
    let leading = rest.len() - rest.trim_start().len();
    let rest = rest.trim_start();
    let (name, name_len) = parse_identifier(rest)?;
    let name_start = line.content_start + prefix_len + leading + 1;
    let name_span = Span::from_usize(name_start, name_start + name_len);
    let parameters = parse_parameters(line, rest, name_len);
    Some(FunctionHeader {
        name: name.to_string(),
        name_span,
        parameters,
    })
}

fn parse_parameters(line: &LineInfo, rest_after_def: &str, name_len: usize) -> Vec<ParameterFact> {
    let after_name = &rest_after_def[name_len..];
    let Some(open_relative) = after_name.find('(') else {
        return Vec::new();
    };
    let open = name_len + open_relative;
    let Some(close) = matching_paren(rest_after_def, open) else {
        return Vec::new();
    };
    let params = &rest_after_def[open + 1..close];
    let param_base =
        line.content_start + line.trimmed.find(rest_after_def).unwrap_or_default() + open + 1;
    split_top_level_commas(params)
        .into_iter()
        .filter_map(|(segment, offset)| parse_parameter(segment, param_base + offset))
        .collect()
}

fn parse_parameter(segment: &str, base: usize) -> Option<ParameterFact> {
    let leading = segment.len() - segment.trim_start().len();
    let mut text = segment.trim();
    if matches!(text, "" | "/" | "*") {
        return None;
    }
    text = text.trim_start_matches('*').trim_start();
    let name_len = text
        .char_indices()
        .take_while(|(_, ch)| is_identifier_continue(*ch))
        .map(|(index, ch)| index + ch.len_utf8())
        .last()?;
    let name = &text[..name_len];
    if !is_identifier_start(name.chars().next()?) {
        return None;
    }
    let name_offset = segment[leading..].find(name).unwrap_or_default() + leading;
    Some(ParameterFact {
        name: name.to_string(),
        span: Span::from_usize(base + name_offset, base + name_offset + name_len),
    })
}

fn parse_binding(line: &LineInfo) -> Option<BindingFact> {
    let (name, name_len) = parse_identifier(line.trimmed.as_str())?;
    let rest = line.trimmed[name_len..].trim_start();
    let assigns = rest.starts_with('=') && !rest.starts_with("==");
    let annotated_assigns = rest.starts_with(':') && rest.contains('=');
    if !assigns && !annotated_assigns {
        return None;
    }
    Some(BindingFact {
        name: name.to_string(),
        span: Span::from_usize(line.content_start, line.content_start + name_len),
    })
}

fn is_branch_header(trimmed: &str) -> bool {
    if !trimmed.ends_with(':') {
        return false;
    }
    [
        "if", "elif", "else", "for", "while", "try", "except", "finally", "with", "match", "case",
    ]
    .iter()
    .any(|keyword| keyword_rest(trimmed, keyword).is_some())
}

fn keyword_rest<'a>(source: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = source.strip_prefix(keyword)?;
    if rest.chars().next().is_some_and(is_identifier_continue) {
        return None;
    }
    Some(rest)
}

fn parse_identifier(source: &str) -> Option<(&str, usize)> {
    let mut chars = source.char_indices();
    let (_, first) = chars.next()?;
    if !is_identifier_start(first) {
        return None;
    }
    let mut end = first.len_utf8();
    for (index, ch) in chars {
        if !is_identifier_continue(ch) {
            break;
        }
        end = index + ch.len_utf8();
    }
    Some((&source[..end], end))
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn matching_paren(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in source.char_indices().skip_while(|(index, _)| *index < open) {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn balanced_parens(source: &str) -> bool {
    let mut depth = 0isize;
    for ch in source.chars() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {}
        }
    }
    depth <= 0
}

fn split_top_level_commas(source: &str) -> Vec<(&str, usize)> {
    let mut parts = Vec::new();
    let mut depth = 0isize;
    let mut start = 0usize;
    for (index, ch) in source.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                parts.push((&source[start..index], start));
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push((&source[start..], start));
    parts
}
