use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct SvelteDescriptor {
    pub module_script: Option<SvelteBlock>,
    pub script: Option<SvelteBlock>,
    pub styles: Vec<SvelteBlock>,
    pub markup: SvelteMarkup,
    pub errors: Vec<SvelteDescriptorError>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct SvelteMarkup {
    pub content: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SvelteBlock {
    pub tag: String,
    pub content: String,
    pub attrs: BTreeMap<String, Option<String>>,
    pub start_offset: usize,
    pub end_offset: usize,
    pub line: usize,
    pub start_line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SvelteDescriptorError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
struct OpenTag {
    name: String,
    attrs: BTreeMap<String, Option<String>>,
    end: usize,
}

#[derive(Debug, Clone, Copy)]
struct CloseTag {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy)]
struct SpecialRange {
    start: usize,
    end: usize,
}

pub fn parse_descriptor(source: &str) -> SvelteDescriptor {
    let mut descriptor = SvelteDescriptor::default();
    let mut ranges = Vec::new();
    let mut cursor = 0;

    while cursor < source.len() {
        let Some(open_offset) = source[cursor..].find('<') else {
            break;
        };
        let open_start = cursor + open_offset;

        if source[open_start..].starts_with("<!--") {
            let Some(comment_end) = source[open_start + 4..].find("-->") else {
                descriptor.errors.push(error_at(
                    source,
                    open_start,
                    "unclosed top-level HTML comment",
                ));
                break;
            };
            cursor = open_start + 4 + comment_end + 3;
            continue;
        }

        if source[open_start..].starts_with("</") {
            cursor = open_start + 2;
            continue;
        }

        let open_tag = match parse_open_tag(source, open_start) {
            Ok(Some(open_tag)) => open_tag,
            Ok(None) => {
                cursor = open_start + 1;
                continue;
            }
            Err(message) => {
                descriptor
                    .errors
                    .push(error_at(source, open_start, message));
                break;
            }
        };

        if !matches!(open_tag.name.as_str(), "script" | "style") {
            cursor = open_tag.end + 1;
            continue;
        }

        let content_start = open_tag.end + 1;
        let Some(close_tag) = find_closing_tag(source, &open_tag.name, content_start) else {
            descriptor.errors.push(error_at(
                source,
                open_start,
                format!("missing closing </{}> block", open_tag.name),
            ));
            break;
        };
        let block = SvelteBlock {
            tag: open_tag.name,
            content: source[content_start..close_tag.start].to_string(),
            attrs: open_tag.attrs,
            start_offset: content_start,
            end_offset: close_tag.start,
            line: line_at(source, open_start),
            start_line: line_offset(source, content_start),
        };
        ranges.push(SpecialRange {
            start: open_start,
            end: close_tag.end,
        });
        push_block(&mut descriptor, source, open_start, block);
        cursor = close_tag.end;
    }

    descriptor.markup = SvelteMarkup {
        content: markup_without_special_blocks(source, &ranges),
    };
    descriptor
}

fn push_block(
    descriptor: &mut SvelteDescriptor,
    source: &str,
    open_start: usize,
    block: SvelteBlock,
) {
    match block.tag.as_str() {
        "script" if is_module_script(&block) => {
            if descriptor.module_script.is_some() {
                descriptor.errors.push(error_at(
                    source,
                    open_start,
                    "duplicate top-level module <script> block",
                ));
            } else {
                descriptor.module_script = Some(block);
            }
        }
        "script" => {
            if descriptor.script.is_some() {
                descriptor.errors.push(error_at(
                    source,
                    open_start,
                    "duplicate top-level <script> block",
                ));
            } else {
                descriptor.script = Some(block);
            }
        }
        "style" => descriptor.styles.push(block),
        _ => {}
    }
}

fn is_module_script(block: &SvelteBlock) -> bool {
    block.attrs.contains_key("module")
        || block
            .attrs
            .get("context")
            .and_then(|value| value.as_deref())
            .is_some_and(|value| value.eq_ignore_ascii_case("module"))
}

fn markup_without_special_blocks(source: &str, ranges: &[SpecialRange]) -> String {
    if ranges.is_empty() {
        return source.to_string();
    }

    let mut markup = String::new();
    let mut cursor = 0;
    for range in ranges {
        if cursor < range.start {
            markup.push_str(&source[cursor..range.start]);
        }
        preserve_layout(&mut markup, &source[range.start..range.end]);
        cursor = range.end;
    }
    if cursor < source.len() {
        markup.push_str(&source[cursor..]);
    }
    markup
}

fn preserve_layout(output: &mut String, source: &str) {
    output.extend(
        source
            .bytes()
            .map(|byte| if byte == b'\n' { '\n' } else { ' ' }),
    );
}

fn parse_open_tag(source: &str, open_start: usize) -> Result<Option<OpenTag>, String> {
    let name_start = open_start + 1;
    let Some((first, width)) = char_at(source, name_start) else {
        return Ok(None);
    };
    if !is_tag_name_start(first) {
        return Ok(None);
    }

    let mut name_end = name_start + width;
    while let Some((ch, width)) = char_at(source, name_end) {
        if !is_tag_name_char(ch) {
            break;
        }
        name_end += width;
    }

    let name = source[name_start..name_end].to_ascii_lowercase();
    let open_end = find_open_tag_end(source, name_end)
        .ok_or_else(|| format!("unclosed <{name}> opening tag"))?;

    Ok(Some(OpenTag {
        name,
        attrs: parse_attrs(&source[name_end..open_end]),
        end: open_end,
    }))
}

fn find_open_tag_end(source: &str, from: usize) -> Option<usize> {
    let mut cursor = from;
    let mut quote = None;

    while let Some((ch, width)) = char_at(source, cursor) {
        if let Some(quote_char) = quote {
            if ch == quote_char {
                quote = None;
            }
            cursor += width;
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '>' => return Some(cursor),
            _ => {}
        }
        cursor += width;
    }

    None
}

fn find_closing_tag(source: &str, tag: &str, from: usize) -> Option<CloseTag> {
    let mut cursor = from;

    while cursor < source.len() {
        let close_offset = source[cursor..].find("</")?;
        let start = cursor + close_offset;
        let name_start = start + 2;
        if source[name_start..]
            .get(..tag.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(tag))
        {
            let mut after_name = name_start + tag.len();
            while let Some((ch, width)) = char_at(source, after_name) {
                if !ch.is_whitespace() {
                    break;
                }
                after_name += width;
            }
            if source[after_name..].starts_with('>') {
                return Some(CloseTag {
                    start,
                    end: after_name + 1,
                });
            }
        }
        cursor = name_start;
    }

    None
}

fn parse_attrs(source: &str) -> BTreeMap<String, Option<String>> {
    let mut attrs = BTreeMap::new();
    let mut cursor = 0;

    while cursor < source.len() {
        cursor = skip_whitespace(source, cursor);
        if cursor >= source.len() {
            break;
        }

        let name_start = cursor;
        while let Some((ch, width)) = char_at(source, cursor) {
            if is_attr_name_end(ch) {
                break;
            }
            cursor += width;
        }
        if cursor == name_start {
            cursor += 1;
            continue;
        }

        let name = source[name_start..cursor].to_ascii_lowercase();
        cursor = skip_whitespace(source, cursor);

        let value = if source[cursor..].starts_with('=') {
            cursor += 1;
            cursor = skip_whitespace(source, cursor);
            let (value, next_cursor) = parse_attr_value(source, cursor);
            cursor = next_cursor;
            value
        } else {
            None
        };
        attrs.insert(name, value);
    }

    attrs
}

fn parse_attr_value(source: &str, cursor: usize) -> (Option<String>, usize) {
    let Some((first, width)) = char_at(source, cursor) else {
        return (Some(String::new()), cursor);
    };

    if first == '"' || first == '\'' {
        let value_start = cursor + width;
        if let Some(end_offset) = source[value_start..].find(first) {
            let value_end = value_start + end_offset;
            return (
                Some(source[value_start..value_end].to_string()),
                value_end + width,
            );
        }
        return (Some(source[value_start..].to_string()), source.len());
    }

    let value_start = cursor;
    let mut value_end = cursor;
    while let Some((ch, width)) = char_at(source, value_end) {
        if ch.is_whitespace() || ch == '>' {
            break;
        }
        value_end += width;
    }

    (Some(source[value_start..value_end].to_string()), value_end)
}

fn skip_whitespace(source: &str, mut cursor: usize) -> usize {
    while let Some((ch, width)) = char_at(source, cursor) {
        if !ch.is_whitespace() {
            break;
        }
        cursor += width;
    }
    cursor
}

fn is_tag_name_start(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

fn is_tag_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':')
}

fn is_attr_name_end(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '=' | '>' | '/' | '"' | '\'')
}

fn char_at(source: &str, offset: usize) -> Option<(char, usize)> {
    source[offset..]
        .chars()
        .next()
        .map(|ch| (ch, ch.len_utf8()))
}

fn error_at(source: &str, offset: usize, message: impl Into<String>) -> SvelteDescriptorError {
    SvelteDescriptorError {
        line: line_at(source, offset),
        message: message.into(),
    }
}

fn line_at(source: &str, offset: usize) -> usize {
    line_offset(source, offset) + 1
}

fn line_offset(source: &str, offset: usize) -> usize {
    source[..offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
}
