use crate::path_match::PathPattern;

#[test]
fn star_matches_segment() {
    let pattern = PathPattern::new("apps/*/src/**");

    assert!(pattern.matches("apps/controller/src/main.rs".as_ref()));
    assert!(!pattern.matches("apps/renderer/vite/src/App.vue".as_ref()));
}

#[test]
fn globstar_matches_nested() {
    let pattern = PathPattern::new("apps/renderer/vite/src/**");

    assert!(pattern.matches("apps/renderer/vite/src/App.vue".as_ref()));
    assert!(pattern.matches("apps/renderer/vite/src/components/im/model.ts".as_ref()));
    assert!(!pattern.matches("apps/renderer/src/main.rs".as_ref()));
}

#[test]
fn segment_glob_matches_ext() {
    let pattern = PathPattern::new("tools/demo/src/*.rs");

    assert!(pattern.matches("tools/demo/src/lib.rs".as_ref()));
    assert!(pattern.matches("tools/demo/src/model.test.rs".as_ref()));
    assert!(!pattern.matches("tools/demo/src/lib.ts".as_ref()));
    assert!(!pattern.matches("tools/demo/src/nested/lib.rs".as_ref()));
}
