use flavor_compiler_core::{LineIndex, SourceText, Span};

#[test]
fn maps_offsets_to_positions() {
    let source = SourceText::new("sample.ts", "one\ntwo\n");
    let index = source.line_index();

    assert_eq!(source.name(), "sample.ts");
    assert_eq!(index.position(0).line, 1);
    assert_eq!(index.position(4).line, 2);
    assert_eq!(index.position(4).column, 1);
    assert_eq!(index.line_count(), 3);
    assert_eq!(index.line_start(1), Some(0));
    assert_eq!(index.line_start(2), Some(4));
    assert_eq!(index.line_start(4), None);
}

#[test]
fn spans_are_byte_ranges() {
    let span = Span::new(2, 7);

    assert_eq!(span.len(), 5);
    assert!(!span.is_empty());
}

#[test]
fn empty_index_starts_one() {
    let index = LineIndex::new("");

    assert_eq!(index.position(0).line, 1);
    assert_eq!(index.position(0).column, 1);
}
