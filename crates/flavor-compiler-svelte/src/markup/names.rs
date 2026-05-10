pub fn source_char_at(source: &str, offset: usize) -> Option<(char, usize)> {
    source[offset..]
        .chars()
        .next()
        .map(|ch| (ch, ch.len_utf8()))
}

pub fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace()
}

pub fn is_tag_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':' | '$')
}

pub fn is_attribute_name_char(ch: char) -> bool {
    !ch.is_whitespace() && !matches!(ch, '=' | '>' | '/' | '"' | '\'' | '{' | '}')
}

pub fn is_void_tag(name: &str) -> bool {
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

pub fn is_component_tag(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
        || name.contains('.')
        || name.starts_with("svelte:")
}

pub fn is_directive_name(name: &str) -> bool {
    directive_base_len(name) > 0
}

pub fn directive_base_len(name: &str) -> usize {
    for prefix in [
        "bind:",
        "class:",
        "on:",
        "use:",
        "transition:",
        "animate:",
        "in:",
        "out:",
        "let:",
        "slot:",
    ] {
        if name.starts_with(prefix) {
            return prefix.len() - 1;
        }
    }
    0
}
