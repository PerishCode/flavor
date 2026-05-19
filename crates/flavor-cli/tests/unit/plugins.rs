use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::{source_file_kind, GuardConfig, SourceKind},
    plugins::{PluginHost, ProductSet, Scope, ScopeDecl, ScopeKind, SourceScope},
    rules,
};
use flavor_grammar::{parse_metadata_validated, GrammarMetadata};

#[test]
fn bundled_manifests_are_explicit() {
    let manifests = PluginHost::bundled().manifests();
    let ids = manifests
        .iter()
        .map(|manifest| manifest.id)
        .collect::<BTreeSet<_>>();

    assert!(ids.contains("flavor-plugin-filesystem"));
    assert!(ids.contains("flavor-plugin-g4"));
    assert!(ids.contains("flavor-plugin-rust"));
    assert!(ids.contains("flavor-plugin-typescript"));
    assert!(ids.contains("flavor-plugin-vue"));
    assert!(ids.contains("flavor-plugin-svelte"));
    assert!(manifests.iter().any(|manifest| {
        manifest.scopes.iter().any(|scope| {
            scope.kind == ScopeKind::SourceFile
                && scope.source == SourceScope::Kind(SourceKind::TypeScript)
        }) && manifest
            .grammars
            .iter()
            .any(|grammar| grammar.grammar_id == "typescript")
    }));
}

#[test]
fn bundled_rules_are_known() {
    for manifest in PluginHost::bundled().manifests() {
        assert!(
            !manifest.rules.is_empty(),
            "{} should declare rule ownership",
            manifest.id
        );
        for rule in manifest.rules {
            assert!(
                rules::descriptor(rule).is_some(),
                "{} declares unknown rule {rule}",
                manifest.id
            );
        }
    }
}

#[test]
fn host_selects_manifest_scopes() {
    let manifests = PluginHost::bundled().manifests();

    assert_eq!(
        selected_plugin_ids(&manifests, ScopeKind::FilePath, None),
        vec!["flavor-plugin-filesystem"]
    );
    assert_eq!(
        selected_plugin_ids(&manifests, ScopeKind::SourceFile, Some(SourceKind::G4)),
        vec!["flavor-plugin-filesystem", "flavor-plugin-g4"]
    );
    assert_eq!(
        selected_plugin_ids(&manifests, ScopeKind::SourceFile, Some(SourceKind::Rust)),
        vec!["flavor-plugin-filesystem", "flavor-plugin-rust"]
    );
    assert_eq!(
        selected_plugin_ids(&manifests, ScopeKind::SourceFile, Some(SourceKind::Vue)),
        vec!["flavor-plugin-filesystem", "flavor-plugin-vue"]
    );
    assert_eq!(
        selected_plugin_ids(&manifests, ScopeKind::DirectoryChildren, None),
        vec!["flavor-plugin-filesystem"]
    );
}

#[test]
fn grammar_uses_have_contracts() {
    let grammars = grammar_documents();
    for manifest in PluginHost::bundled().manifests() {
        for grammar in manifest.grammars {
            assert!(
                manifest
                    .scopes
                    .iter()
                    .any(|scope| scope.kind == grammar.scope),
                "{} declares grammar {} for unowned scope {:?}",
                manifest.id,
                grammar.grammar_id,
                grammar.scope
            );
            let document = grammars.get(grammar.grammar_id).unwrap_or_else(|| {
                panic!(
                    "{} declares missing grammar {}",
                    manifest.id, grammar.grammar_id
                )
            });
            assert_eq!(
                document
                    .directive("entry")
                    .map(|entry| entry.value.as_str()),
                Some(grammar.entrypoint),
                "{} declares mismatched entrypoint for {}",
                manifest.id,
                grammar.grammar_id
            );
        }
    }
}

#[test]
fn fact_uses_have_contracts() {
    let grammars = grammar_documents();
    for manifest in PluginHost::bundled().manifests() {
        for fact in manifest.facts {
            let document = grammars.get(fact.grammar_id).unwrap_or_else(|| {
                panic!(
                    "{} declares missing grammar {}",
                    manifest.id, fact.grammar_id
                )
            });
            let entry = document
                .section("facts")
                .and_then(|section| section.entry(fact.key))
                .unwrap_or_else(|| {
                    panic!(
                        "{} declares missing fact {}.{}",
                        manifest.id, fact.grammar_id, fact.key
                    )
                });
            for fragment in fact.contains {
                assert!(
                    entry.value.contains(fragment),
                    "{} fact {}.{} should contain `{fragment}`; value was `{}`",
                    manifest.id,
                    fact.grammar_id,
                    fact.key,
                    entry.value
                );
            }
        }
    }
}

#[test]
fn product_queries_rust_facts() {
    let manifest = manifest("flavor-plugin-rust");
    let source = "#[cfg(test)] mod tests { #[test] fn sample(input_value: i32) { match input_value { 1 => { let local_value = input_value; local_value } _ => 0 } } }";
    let config = core_config();
    let products = ProductSet::new(
        &config,
        manifest,
        Scope::source_file(
            Path::new("src/lib.rs"),
            "src/lib.rs",
            source,
            SourceKind::Rust,
        ),
    );

    let parameter = products
        .facts("rust", "name.parameter")
        .find(|fact| fact.text("name") == Some("input_value"))
        .expect("Rust parameter fact");
    assert_eq!(parameter.text("kind"), Some("parameter"));
    assert_eq!(parameter.text("issue_kind"), Some("binding"));
    assert!(parameter.line.is_some());
    assert!(products
        .facts("rust", "dispatch.branch")
        .any(|fact| fact.usize("lines").is_some()));
    assert!(products.facts("rust", "test.attribute").count() >= 2);
}

#[test]
fn product_queries_tsx_facts() {
    let manifest = manifest("flavor-plugin-typescript");
    let source = r#"import DefaultThing, { Stack as Surface } from "@mini-stim/components";
import * as Primitive from "@mini-stim/components";

export function Panel() {
  switch (DefaultThing.kind) {
    case "ready":
      return <Primitive.Stack><button /></Primitive.Stack>;
    default:
      return <Surface />;
  }
}
"#;
    let config = core_config();
    let products = ProductSet::new(
        &config,
        manifest,
        Scope::source_file(
            Path::new("src/Panel.tsx"),
            "src/Panel.tsx",
            source,
            SourceKind::TypeScript,
        ),
    );

    let import = products
        .facts("typescript", "module.import")
        .find(|fact| fact.text("source") == Some("@mini-stim/components"))
        .expect("TypeScript import fact");
    assert!(import
        .texts("named_imports")
        .unwrap_or_default()
        .contains(&"Surface".to_string()));
    assert!(products
        .facts("typescript", "dispatch.branch")
        .any(|fact| fact.usize("lines").is_some()));
    let button = products
        .facts("tsx", "jsx.self_closing")
        .find(|fact| fact.text("intrinsic") == Some("button"))
        .expect("TSX self-closing intrinsic fact");
    assert_eq!(button.bool("self_closing"), Some(true));
}

#[test]
fn product_queries_embedded_facts() {
    let config = core_config();
    let vue = ProductSet::new(
        &config,
        manifest("flavor-plugin-vue"),
        Scope::source_file(
            Path::new("src/App.vue"),
            "src/App.vue",
            "<template></template>\n<script setup lang=\"tsx\">\nconst rendererOperationEventHandlerName = 1;\n</script>",
            SourceKind::Vue,
        ),
    );
    let embedded = vue
        .facts("vue-sfc", "script.embedded")
        .next()
        .expect("Vue embedded script fact");
    assert_eq!(embedded.text("lang"), Some("tsx"));
    assert_eq!(embedded.bool("tsx"), Some(true));
    assert_eq!(embedded.usize("start_line"), Some(1));
    assert!(vue
        .facts("typescript", "name.binding")
        .any(|fact| fact.line == Some(3)
            && fact.text("name") == Some("rendererOperationEventHandlerName")));

    let svelte = ProductSet::new(
        &config,
        manifest("flavor-plugin-svelte"),
        Scope::source_file(
            Path::new("src/Panel.svelte"),
            "src/Panel.svelte",
            "<script lang=\"ts\">\nconst rendererOperationEventHandlerName = 1;\n</script>\n{#if rendererOperationEventHandlerName}<p>ok</p>{/if}",
            SourceKind::Svelte,
        ),
    );
    let shape = svelte
        .facts("svelte", "descriptor.markup")
        .next()
        .expect("Svelte shape fact");
    assert_eq!(shape.usize("script_count"), Some(1));
    assert_eq!(shape.usize("markup_block_count"), Some(1));
    assert!(svelte
        .facts("svelte-markup", "markup.block")
        .next()
        .is_some());
    assert!(svelte
        .facts("typescript", "name.binding")
        .any(|fact| fact.line == Some(2)
            && fact.text("name") == Some("rendererOperationEventHandlerName")));
}

fn core_config() -> GuardConfig {
    GuardConfig::core(PathBuf::from("."))
}

#[test]
fn parser_coupling_stays_private() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    for path in rust_sources(&source_root) {
        let source = fs::read_to_string(&path).unwrap();
        for forbidden in [
            "tree_sitter",
            "RustNameKind",
            "TsNameKind",
            "VueSfcBlock",
            "SvelteBlock",
            "run as run_",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} imports parser/backend detail `{forbidden}`",
                path.display()
            );
        }
    }
}

#[test]
fn delivery_boundary_names() {
    let root = repo_root();
    let forbidden = old_boundary_names();
    for path in repo_text_files(&root) {
        let source = fs::read_to_string(&path).unwrap();
        for forbidden in &forbidden {
            assert!(
                !source.contains(forbidden),
                "{} keeps old architecture boundary `{forbidden}`",
                path.display()
            );
        }
    }
}

#[test]
fn language_uses_product_queries() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/plugins");
    let source = fs::read_to_string(root.join("language.rs")).unwrap();
    for forbidden in [
        "RustProduct",
        "TypeScriptProduct",
        "VueProduct",
        "SvelteProduct",
        "RustFacts",
        "TsFacts",
        "VueSfcBlock",
        "SvelteBlock",
    ] {
        assert!(
            !source.contains(forbidden),
            "language analysis should not consume typed parser/backend product {forbidden}"
        );
    }
    assert!(source.contains(".facts("));
    assert!(source.contains(".diagnostics("));
}

#[test]
fn source_kind_routes_vue() {
    assert_eq!(
        source_file_kind(std::path::Path::new("Grammar.g4")),
        Some(SourceKind::G4)
    );
    assert_eq!(
        source_file_kind(std::path::Path::new("App.vue")),
        Some(SourceKind::Vue)
    );
    assert_eq!(
        source_file_kind(std::path::Path::new("App.tsx")),
        Some(SourceKind::TypeScript)
    );
}

fn manifest(id: &'static str) -> crate::plugins::PluginManifest {
    PluginHost::bundled()
        .manifests()
        .into_iter()
        .find(|manifest| manifest.id == id)
        .unwrap_or_else(|| panic!("missing manifest {id}"))
}

fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_rust_sources(root, &mut paths);
    paths
}

fn collect_rust_sources(path: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rust_sources(&path, paths);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            paths.push(path);
        }
    }
}

fn grammar_documents() -> BTreeMap<String, GrammarMetadata> {
    let mut documents = BTreeMap::new();
    for path in grammar_files(&grammar_root()) {
        if path.file_name().and_then(|name| name.to_str()) != Some("metadata.json") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let parsed = parse_metadata_validated(&source).unwrap_or_else(|errors| {
            panic!("{} parse errors: {errors:?}", path.display());
        });
        for document in parsed {
            documents.insert(document.name.clone(), document);
        }
    }
    documents
}

fn grammar_files(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_grammar_files(root, &mut paths);
    paths
}

fn collect_grammar_files(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_grammar_files(&path, paths);
        } else {
            paths.push(path);
        }
    }
}

fn selected_plugin_ids(
    manifests: &[crate::plugins::PluginManifest],
    kind: ScopeKind,
    source_kind: Option<SourceKind>,
) -> Vec<&'static str> {
    manifests
        .iter()
        .filter(|manifest| {
            manifest
                .scopes
                .iter()
                .any(|scope| scope_matches(*scope, kind, source_kind))
        })
        .map(|manifest| manifest.id)
        .collect()
}

fn scope_matches(scope: ScopeDecl, kind: ScopeKind, source_kind: Option<SourceKind>) -> bool {
    scope.kind == kind
        && match scope.source {
            SourceScope::Any => true,
            SourceScope::Kind(expected) => source_kind == Some(expected),
        }
}

fn grammar_root() -> PathBuf {
    repo_root().join("grammars")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn old_boundary_names() -> Vec<String> {
    ["core", "rust", "ts", "vue", "svelte"]
        .into_iter()
        .flat_map(|name| {
            [
                format!("flavor-{}-{name}", concat!("comp", "iler")),
                format!("flavor_{}_{}", concat!("comp", "iler"), name),
            ]
        })
        .collect()
}

fn repo_text_files(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_repo_text_files(root, &mut paths);
    paths
}

fn collect_repo_text_files(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if path.is_dir() {
            if matches!(name, ".git" | ".task" | "target") {
                continue;
            }
            if path
                .components()
                .any(|component| component.as_os_str() == "target")
            {
                continue;
            }
            collect_repo_text_files(&path, paths);
            continue;
        }
        if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("md" | "toml" | "rs" | "json" | "g4" | "yml" | "yaml" | "sh" | "ps1" | "lock")
        ) {
            paths.push(path);
        }
    }
}
