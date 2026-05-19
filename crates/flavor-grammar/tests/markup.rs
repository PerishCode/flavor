use flavor_grammar::{
    find_balanced_brace_close, find_html_comment_close, is_html_void_element, is_markup_name_char,
    markup_char_at, scan_markup_name,
};

#[test]
fn scans_markup_names() {
    let source = "x-node$value";

    assert_eq!(scan_markup_name(source, 0, is_markup_name_char), 6);
    assert_eq!(
        scan_markup_name(source, 0, |ch| is_markup_name_char(ch) || ch == '$'),
        12
    );
}

#[test]
fn exposes_utf8_cursor_width() {
    assert_eq!(markup_char_at("tag", 0), Some(('t', 1)));
    assert_eq!(markup_char_at("tag", 3), None);
}

#[test]
fn recognizes_html_void_elements() {
    assert!(is_html_void_element("img"));
    assert!(is_html_void_element("input"));
    assert!(!is_html_void_element("main"));
}

#[test]
fn finds_html_comment_close() {
    let source = "<!-- keep <tag> -->after";

    assert_eq!(find_html_comment_close(source, 0), Some(19));
    assert_eq!(
        &source[find_html_comment_close(source, 0).unwrap()..],
        "after"
    );
    assert_eq!(find_html_comment_close("<!-- open", 0), None);
}

#[test]
fn finds_balanced_brace_close() {
    let source = r#"value({ nested: "}" })} tail"#;
    let close = find_balanced_brace_close(source, 0).expect("close brace");

    assert_eq!(&source[close..close + 1], "}");
    assert_eq!(&source[close + 1..], " tail");
}
