use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn syntax_kinds_generated() {
    let files = [
        (
            "crates/flavor-plugin-rust/src/syntax_kind.rs",
            r#"include!(concat!(env!("OUT_DIR"), "/rust_syntax_kind.rs"));"#,
        ),
        (
            "crates/flavor-plugin-typescript/src/syntax_kind.rs",
            r#"include!(concat!(env!("OUT_DIR"), "/ts_syntax_kind.rs"));"#,
        ),
        (
            "crates/flavor-plugin-vue/src/template/kind.rs",
            r#"include!(concat!(env!("OUT_DIR"), "/vue_template_kind.rs"));"#,
        ),
        (
            "crates/flavor-plugin-svelte/src/markup/kind.rs",
            r#"include!(concat!(env!("OUT_DIR"), "/svelte_markup_kind.rs"));"#,
        ),
    ];

    for (path, expected) in files {
        let source = read_repo(path);
        assert_eq!(
            source.trim(),
            expected,
            "{path} should be an include-only generated binding"
        );
        assert!(
            !source.contains("pub enum"),
            "{path} should not reintroduce a hand-written raw syntax enum"
        );
    }
}

#[test]
fn builders_use_schema() {
    let files = builder_files();
    let mut schema_nodes = 0;
    let mut schema_tokens = 0;

    for path in files {
        let source = read_repo(&path);
        if source.contains("start_schema_node") {
            schema_nodes += 1;
        }
        if source.contains("schema_token") {
            schema_tokens += 1;
        }
        for (line_index, line) in source.lines().enumerate() {
            assert!(
                !line.contains(".start_node("),
                "{}:{} should use start_schema_node for raw AST nodes",
                path,
                line_index + 1
            );
            if line.contains(".token(") && !allowed_raw_token(&path, line) {
                panic!(
                    "{}:{} should use schema_token for raw AST tokens",
                    path,
                    line_index + 1
                );
            }
        }
    }

    assert!(
        schema_nodes > 0,
        "raw AST builders should emit schema nodes"
    );
    assert!(
        schema_tokens > 0,
        "raw AST builders should emit schema tokens"
    );
}

#[test]
fn rust_adapter_generated() {
    let source = read_repo("crates/flavor-plugin-rust/src/raw_ast.rs");
    let build = read_repo("crates/flavor-plugin-rust/build.rs");
    assert!(
        source.contains(r#"include!(concat!(env!("OUT_DIR"), "/rust_tree_sitter_nodes.rs"));"#),
        "Rust tree-sitter node bindings should be generated from metadata"
    );
    assert!(
        source.contains(r#"include!(concat!(env!("OUT_DIR"), "/rust_tree_sitter_tokens.rs"));"#),
        "Rust tree-sitter token bindings should be generated from G4"
    );
    assert!(
        source.contains(r#"include!(concat!(env!("OUT_DIR"), "/rust_gap_kind.rs"));"#),
        "Rust gap token bindings should be generated from G4"
    );
    assert!(
        !source.contains("fn node_kind(")
            && !source.contains("fn token_kind_for_node(")
            && !source.contains("fn gap_kind("),
        "Rust tree-sitter and gap raw AST bindings should not be hand-written"
    );
    assert!(
        build.contains(r#"join("metadata.json")"#)
            && build.contains("parse_metadata_validated")
            && build.contains("render_rust_node_adapter")
            && build.contains("render_rust_token_adapter")
            && build.contains("render_rust_gap_adapter"),
        "Rust raw AST adapter bindings should be generated from metadata/G4"
    );
}

fn allowed_raw_token(path: &str, line: &str) -> bool {
    path == "crates/flavor-plugin-typescript/src/parser/mod.rs"
        && line.contains(".token(trivia.kind")
}

fn builder_files() -> Vec<String> {
    let mut files = vec![
        "crates/flavor-plugin-rust/src/raw_ast.rs".to_string(),
        "crates/flavor-plugin-vue/src/template/parser.rs".to_string(),
    ];
    collect_rs_files("crates/flavor-plugin-typescript/src/parser", &mut files);
    collect_rs_files("crates/flavor-plugin-svelte/src/markup", &mut files);
    files.sort();
    files
}

fn collect_rs_files(path: &str, files: &mut Vec<String>) {
    let full_path = repo_path(path);
    for entry in fs::read_dir(&full_path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            let relative = relative_path(&path);
            collect_rs_files(&relative, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(relative_path(&path));
        }
    }
}

fn read_repo(path: &str) -> String {
    fs::read_to_string(repo_path(path)).unwrap()
}

fn repo_path(path: &str) -> PathBuf {
    repo_root().join(path)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn relative_path(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}
