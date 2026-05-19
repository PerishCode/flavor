use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn string_kind_shapes() {
    let files = [
        (
            "crates/flavor-plugin-rust/src/internal/grammar.rs",
            "pub const SOURCE_FILE: Kind = \"source_file\";",
        ),
        (
            "crates/flavor-plugin-typescript/src/internal/grammar.rs",
            "pub const SOURCE_FILE: Kind = \"source_file\";",
        ),
        (
            "crates/flavor-plugin-vue/src/template/kind.rs",
            "pub const ROOT: Kind = \"root\";",
        ),
        (
            "crates/flavor-plugin-svelte/src/markup/kind.rs",
            "pub const ROOT: Kind = \"root\";",
        ),
    ];

    for (path, marker) in files {
        let source = read_repo(path);
        assert!(source.contains("pub type Kind = &'static str;"));
        assert!(
            source.contains(marker),
            "{path} should expose string kind constants"
        );
        assert!(source.contains("pub fn schema() -> RawAstSchema"));
        assert!(source.contains("pub const SPEC: GrammarSpec<'static>"));
        assert!(
            !source.contains("pub enum")
                && !source.contains("schema_from_g4")
                && !source.contains("pub fn raw_kind")
                && !source.contains("pub fn raw_is_node")
                && !source.contains("pub fn raw_is_token")
                && !source.contains("pub fn is_token")
                && !source.contains("RustSyntaxKind")
                && !source.contains("TsSyntaxKind")
                && !source.contains("VueTemplateKind")
                && !source.contains("SvelteMarkupKind")
                && !source.contains("generated::"),
            "{path} should not reintroduce syntax kind types or generated bindings"
        );
    }
}

#[test]
fn generated_rust_bindings_removed() {
    assert!(
        !repo_root()
            .join("crates/flavor-grammar/src/generated")
            .exists(),
        "flavor-grammar should not keep generated Rust syntax bindings"
    );
}

#[test]
fn plugin_surface_hides_internals() {
    let forbidden = [
        "pub mod ast",
        "pub mod facts",
        "pub mod lexer",
        "pub mod parser",
        "pub mod syntax_kind",
        "pub mod template",
        "pub mod markup",
        "pub mod descriptor",
        "pub mod sfc",
        "pub mod product",
    ];
    for path in [
        "crates/flavor-plugin-rust/src/lib.rs",
        "crates/flavor-plugin-typescript/src/lib.rs",
        "crates/flavor-plugin-vue/src/lib.rs",
        "crates/flavor-plugin-svelte/src/lib.rs",
        "crates/flavor-plugin-g4/src/lib.rs",
    ] {
        let source = read_repo(path);
        assert!(
            source.contains("pub mod plugin"),
            "{path} should expose first-party plugin delivery as `plugin`"
        );
        for pattern in forbidden {
            assert!(
                !source.contains(pattern),
                "{path} should not expose parser/raw AST implementation module `{pattern}`"
            );
        }
    }
}

#[test]
fn out_dir_includes_removed() {
    let pattern = concat!("include!(concat!(env!(\"", "OUT_DIR", "\")");
    for path in repo_rs_files() {
        let source = read_repo(&path);
        assert!(
            !source.contains(pattern),
            "{path} should not include hidden OUT_DIR generated Rust"
        );
    }
}

#[test]
fn builders_use_schema() {
    let raw_builder = read_repo("crates/flavor-grammar/src/raw_builder.rs");
    assert!(
        raw_builder.contains("SyntaxBuilder")
            && raw_builder.contains("schema.node_kind")
            && raw_builder.contains("schema.token_kind"),
        "RawAstBuilder should resolve string kinds through the grammar-owned runtime schema"
    );

    let files = builder_files();

    for path in files {
        let source = read_repo(&path);
        for (line_index, line) in source.lines().enumerate() {
            assert!(
                !line.contains("SyntaxBuilder"),
                "{}:{} should use flavor_grammar::RawAstBuilder or grammar-owned adapters",
                path,
                line_index + 1
            );
            assert!(
                !line.contains("start_schema_node") && !line.contains("schema_token"),
                "{}:{} should not call core schema builder APIs outside RawAstBuilder",
                path,
                line_index + 1
            );
        }
    }
}

#[test]
fn rust_parse_in_grammar() {
    let grammar_source = read_repo("crates/flavor-grammar/src/tree_sitter_raw.rs");
    assert!(
        grammar_source.contains("pub fn parse_tree_sitter")
            && grammar_source.contains("tree_sitter::Parser::new()")
            && grammar_source.contains("TreeSitterRawAstAdapter::new"),
        "tree-sitter parse orchestration should live in flavor-grammar"
    );

    let source = read_repo("crates/flavor-plugin-rust/src/internal/frontend.rs");
    assert!(
        source.contains("parse_tree_sitter"),
        "Rust plugin should call grammar-owned tree-sitter parse orchestration"
    );
    assert!(
        !source.contains("tree_sitter::Parser")
            && !source.contains("Parser::new")
            && !source.contains("TreeSitterRawAstAdapter")
            && !source.contains("RawAstBuilder")
            && !source.contains("tree_sitter_error_span")
            && !source.contains("generated::"),
        "Rust plugin should not own tree-sitter parser execution or raw AST construction"
    );
}

#[test]
fn plugin_raw_builders_tracked() {
    let mut actual = Vec::new();
    collect_rs_files("crates/flavor-plugin-rust/src", &mut actual);
    collect_rs_files("crates/flavor-plugin-typescript/src", &mut actual);
    collect_rs_files("crates/flavor-plugin-vue/src", &mut actual);
    collect_rs_files("crates/flavor-plugin-svelte/src", &mut actual);
    collect_rs_files("crates/flavor-plugin-g4/src", &mut actual);
    actual.retain(|path| read_repo(path).contains("RawAstBuilder"));
    actual.sort();

    assert_eq!(
        actual,
        vec![
            "crates/flavor-plugin-svelte/src/markup/parser.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/mod.rs".to_string(),
            "crates/flavor-plugin-vue/src/template/parser.rs".to_string(),
        ],
        "remaining plugin-side raw CST builders must stay explicit until migrated into flavor-grammar"
    );
}

#[test]
fn markup_atoms_in_grammar() {
    let grammar_source = read_repo("crates/flavor-grammar/src/markup.rs");
    for pattern in [
        "pub fn markup_char_at",
        "pub fn scan_markup_name",
        "pub fn is_html_void_element",
        "pub fn is_markup_name_char",
        "pub fn find_html_comment_close",
        "pub fn find_balanced_brace_close",
    ] {
        assert!(
            grammar_source.contains(pattern),
            "flavor-grammar should own shared markup atom `{pattern}`"
        );
    }

    for path in [
        "crates/flavor-plugin-vue/src/template/names.rs",
        "crates/flavor-plugin-svelte/src/markup/names.rs",
    ] {
        let source = read_repo(path);
        assert!(
            !source.contains("fn source_char_at") && !source.contains("fn is_void_tag"),
            "{path} should not duplicate grammar-owned markup cursor or void-element atoms"
        );
    }
    assert!(
        !repo_path("crates/flavor-plugin-svelte/src/markup/cursor.rs").exists(),
        "Svelte markup should use grammar-owned brace close scanning"
    );
}

#[test]
fn parser_escapes_tracked() {
    let mut actual = Vec::new();
    collect_rs_files("crates/flavor-plugin-rust/src", &mut actual);
    collect_rs_files("crates/flavor-plugin-typescript/src", &mut actual);
    collect_rs_files("crates/flavor-plugin-vue/src", &mut actual);
    collect_rs_files("crates/flavor-plugin-svelte/src", &mut actual);
    collect_rs_files("crates/flavor-plugin-g4/src", &mut actual);
    actual.retain(|path| {
        let source = read_repo(path);
        source.contains("RawAstBuilder")
            || source.contains("struct Scanner")
            || source.contains("struct Parser")
            || source.contains("struct TemplateParser")
            || source.contains("struct MarkupParser")
            || source.contains("fn parse_")
            || source.contains("pub fn parse_")
    });
    actual.sort();

    assert_eq!(
        actual,
        vec![
            "crates/flavor-plugin-svelte/src/descriptor/parser.rs".to_string(),
            "crates/flavor-plugin-svelte/src/markup/attribute.rs".to_string(),
            "crates/flavor-plugin-svelte/src/markup/parser.rs".to_string(),
            "crates/flavor-plugin-typescript/src/lexer/mod.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/bindings.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/declarations.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/expressions.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/grammar.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/jsx.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/members.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/mod.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/modules.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/statements.rs".to_string(),
            "crates/flavor-plugin-typescript/src/parser/types.rs".to_string(),
            "crates/flavor-plugin-vue/src/sfc/parser.rs".to_string(),
            "crates/flavor-plugin-vue/src/template/parser.rs".to_string(),
        ],
        "remaining plugin-side parser execution files must stay explicit until migrated into flavor-grammar"
    );
}

#[test]
fn unused_dynamic_meta_removed() {
    let pattern = concat!("Meta", "Value");
    for path in repo_rs_files() {
        let source = read_repo(&path);
        assert!(
            !source.contains(pattern),
            "{path} should not keep unused dynamic AST meta value APIs"
        );
    }
}

#[test]
fn fact_atoms_used() {
    let patterns = [
        (".child_tokens().find", "child_tokens_any"),
        (".child_tokens().any", "has_token"),
        (".tokens().filter", "tokens_any"),
        (".tokens_named(", "token_text or tokens_any"),
        (
            ".children().map(|child| child.span().start).min()",
            "head_tokens",
        ),
        (r#".child("directive_name").map"#, "child_text"),
    ];

    for path in fact_files() {
        let source = read_repo(path);
        for (pattern, helper) in patterns {
            assert!(
                !source.contains(pattern),
                "{path} should use grammar atom `{helper}` instead of `{pattern}`"
            );
        }
    }
}

fn builder_files() -> Vec<String> {
    let mut files = vec![
        "crates/flavor-grammar/src/tree_sitter_raw.rs".to_string(),
        "crates/flavor-plugin-vue/src/template/parser.rs".to_string(),
    ];
    collect_rs_files("crates/flavor-plugin-typescript/src/parser", &mut files);
    collect_rs_files("crates/flavor-plugin-svelte/src/markup", &mut files);
    files.sort();
    files
}

fn fact_files() -> [&'static str; 4] {
    [
        "crates/flavor-plugin-rust/src/internal/collect.rs",
        "crates/flavor-plugin-typescript/src/facts/mod.rs",
        "crates/flavor-plugin-vue/src/facts/mod.rs",
        "crates/flavor-plugin-svelte/src/facts.rs",
    ]
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

fn repo_rs_files() -> Vec<String> {
    let mut files = Vec::new();
    collect_rs_files("crates", &mut files);
    files
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
