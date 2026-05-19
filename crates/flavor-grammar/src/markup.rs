pub fn markup_char_at(source: &str, offset: usize) -> Option<(char, usize)> {
    source
        .get(offset..)?
        .chars()
        .next()
        .map(|ch| (ch, ch.len_utf8()))
}

pub fn scan_markup_name(
    source: &str,
    offset: usize,
    mut is_name_char: impl FnMut(char) -> bool,
) -> usize {
    let mut cursor = offset;
    while let Some((ch, width)) = markup_char_at(source, cursor) {
        if !is_name_char(ch) {
            break;
        }
        cursor += width;
    }
    cursor
}

pub fn is_markup_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.')
}

pub fn is_html_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

pub fn find_html_comment_close(source: &str, from: usize) -> Option<usize> {
    source
        .get(from..)?
        .find("-->")
        .map(|relative| from + relative + 3)
}

pub fn find_balanced_brace_close(source: &str, from: usize) -> Option<usize> {
    let mut cursor = from;
    let mut quote = None;
    let mut depth = 0usize;

    while let Some((ch, width)) = markup_char_at(source, cursor) {
        if let Some(quote_char) = quote {
            if ch == '\\' {
                cursor += width;
                if let Some((_, escaped_width)) = markup_char_at(source, cursor) {
                    cursor += escaped_width;
                }
                continue;
            }
            if ch == quote_char {
                quote = None;
            }
            cursor += width;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '{' => depth += 1,
            '}' if depth == 0 => return Some(cursor),
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        cursor += width;
    }

    None
}
