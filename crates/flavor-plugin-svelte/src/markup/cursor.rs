use super::names::source_char_at;

pub fn find_mustache_end(source: &str, from: usize) -> Option<usize> {
    let mut cursor = from;
    let mut quote = None;
    let mut depth = 0usize;

    while let Some((ch, width)) = source_char_at(source, cursor) {
        if let Some(quote_char) = quote {
            if ch == '\\' {
                cursor += width;
                if let Some((_, escaped_width)) = source_char_at(source, cursor) {
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
