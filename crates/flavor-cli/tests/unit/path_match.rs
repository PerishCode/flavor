use crate::path_match::PathPattern;

#[test]
fn star_matches_segment() {
    let pattern = PathPattern::new("apps/*/src/**").unwrap();

    assert!(pattern.matches("apps/controller/src/main.rs".as_ref()));
    assert!(!pattern.matches("apps/renderer/vite/src/App.vue".as_ref()));
}

#[test]
fn globstar_matches_nested() {
    let pattern = PathPattern::new("apps/renderer/vite/src/**").unwrap();

    assert!(pattern.matches("apps/renderer/vite/src/App.vue".as_ref()));
    assert!(pattern.matches("apps/renderer/vite/src/components/im/model.ts".as_ref()));
    assert!(!pattern.matches("apps/renderer/src/main.rs".as_ref()));
}

#[test]
fn segment_glob_matches_ext() {
    let pattern = PathPattern::new("tools/demo/src/*.rs").unwrap();

    assert!(pattern.matches("tools/demo/src/lib.rs".as_ref()));
    assert!(pattern.matches("tools/demo/src/model.test.rs".as_ref()));
    assert!(!pattern.matches("tools/demo/src/lib.ts".as_ref()));
    assert!(!pattern.matches("tools/demo/src/nested/lib.rs".as_ref()));
}

#[test]
fn brace_glob_matches_extensions() {
    let pattern = PathPattern::new("src/**/*.{ts,tsx,vue}").unwrap();

    assert!(pattern.matches("src/app/main.ts".as_ref()));
    assert!(pattern.matches("src/app/view.tsx".as_ref()));
    assert!(pattern.matches("src/app/Panel.vue".as_ref()));
    assert!(!pattern.matches("src/app/main.rs".as_ref()));
}
