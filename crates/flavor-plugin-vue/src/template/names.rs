pub(super) fn is_attribute_name_char(ch: char) -> bool {
    !is_whitespace(ch) && !matches!(ch, '=' | '>' | '/' | '"' | '\'')
}

pub(super) fn is_directive_name(name: &str) -> bool {
    name.starts_with("v-")
        || name.starts_with(':')
        || name.starts_with('@')
        || name.starts_with('#')
}

pub(super) fn directive_base_len(name: &str) -> usize {
    if is_shorthand_directive(name) {
        return 1;
    }
    name.find([':', '.']).unwrap_or(name.len())
}

pub(super) fn is_shorthand_directive(name: &str) -> bool {
    name.starts_with(':') || name.starts_with('@') || name.starts_with('#')
}

pub(super) fn is_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\n' | '\r' | '\t')
}
