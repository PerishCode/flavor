use flavor_core::{RawSyntaxKind, SourceText};
use flavor_plugin_svelte::{run, SveltePluginConfig};

#[path = "../src/markup/ast.rs"]
mod ast;
#[path = "../src/markup/attribute.rs"]
mod attribute;
#[path = "../src/markup/kind.rs"]
pub mod kind;
#[path = "../src/markup/names.rs"]
mod names;
#[path = "../src/markup/parser.rs"]
mod parser;

use ast::SvelteMarkupAst;
use kind::Kind;
use parser::parse_markup;

fn has_node(ast: &SvelteMarkupAst, kind: Kind) -> bool {
    ast.syntax()
        .descendants()
        .any(|node| node.kind() == kind::schema().raw_kind(kind))
}

fn token_count(ast: &SvelteMarkupAst, kind: Kind) -> usize {
    ast.syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() == kind::schema().raw_kind(kind))
        .count()
}

fn is_core_trivia(kind: RawSyntaxKind) -> bool {
    matches!(kind.0, 1..=4)
}

fn assert_cst_matches_schema(ast: &SvelteMarkupAst) {
    for node in ast.syntax().descendants() {
        assert!(
            kind::schema().raw_is_node(node.kind()),
            "node kind {:?} is not declared as a G4 node",
            node.kind()
        );
    }
    for token in ast
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        assert!(
            kind::schema().raw_is_token(token.kind()) || is_core_trivia(token.kind()),
            "token kind {:?} is not declared as a G4 token",
            token.kind()
        );
    }
}

#[test]
fn parses_elements_and_components() {
    let source = "<main><Panel title=\"Hi\">{message}</Panel><img /></main>";
    let ast = parse_markup(source);

    assert_eq!(ast.syntax().text().to_string(), source);
    assert!(has_node(&ast, kind::ELEMENT));
    assert!(has_node(&ast, kind::COMPONENT));
    assert!(has_node(&ast, kind::MUSTACHE));
    assert!(ast.diagnostics().is_empty());
}

#[test]
fn cst_matches_schema() {
    let ast =
        parse_markup(r#"<button bind:value={name} {...props}>{#if ready}{message}{/if}</button>"#);

    assert_cst_matches_schema(&ast);
    assert!(ast.diagnostics().is_empty());
}

#[test]
fn parses_blocks_render() {
    let source = r#"{#if ready}
  {@render children?.()}
{:else}
  <p>Loading</p>
{/if}
{#each items as item}
  <Row value={item} />
{/each}"#;
    let ast = parse_markup(source);

    assert_eq!(ast.syntax().text().to_string(), source);
    assert_eq!(
        ast.syntax()
            .descendants()
            .filter(|node| node.kind() == kind::schema().raw_kind(kind::BLOCK))
            .count(),
        2
    );
    assert!(has_node(&ast, kind::BLOCK_BRANCH));
    assert!(has_node(&ast, kind::RENDER_TAG));
    assert!(ast.diagnostics().is_empty());
}

#[test]
fn classifies_directives_and_shorthands() {
    let source = r#"<button bind:value={name} class:active on:click|once={save} use:action {...props} {disabled}>Save</button>"#;
    let ast = parse_markup(source);

    assert_eq!(ast.syntax().text().to_string(), source);
    assert_eq!(
        ast.syntax()
            .descendants()
            .filter(|node| node.kind() == kind::schema().raw_kind(kind::DIRECTIVE))
            .count(),
        4
    );
    assert!(has_node(&ast, kind::SPREAD_ATTRIBUTE));
    assert!(has_node(&ast, kind::SHORTHAND_ATTRIBUTE));
    assert_eq!(token_count(&ast, kind::DIRECTIVE_ARGUMENT), 4);
}

#[test]
fn parses_snippet_await() {
    let source = r#"{#snippet row(item)}
  {#key item.id}<p>{item.name}</p>{/key}
{/snippet}
{#await promise}
  <p>Waiting</p>
{:then value}
  <p>{value}</p>
{:catch error}
  <p>{error.message}</p>
{/await}"#;
    let ast = parse_markup(source);

    assert_eq!(ast.syntax().text().to_string(), source);
    assert_eq!(
        ast.syntax()
            .descendants()
            .filter(|node| node.kind() == kind::schema().raw_kind(kind::BLOCK))
            .count(),
        3
    );
    assert_eq!(
        ast.syntax()
            .descendants()
            .filter(|node| node.kind() == kind::schema().raw_kind(kind::BLOCK_BRANCH))
            .count(),
        2
    );
    assert!(ast.diagnostics().is_empty());
}

#[test]
fn parses_special_tags() {
    let source = r#"<svelte:head><title>{title}</title></svelte:head>
{@html raw}
{@const total = items.length}
{@debug total}"#;
    let ast = parse_markup(source);

    assert_eq!(ast.syntax().text().to_string(), source);
    assert!(has_node(&ast, kind::COMPONENT));
    assert!(has_node(&ast, kind::SPECIAL_TAG));
    assert!(ast.diagnostics().is_empty());
}

#[test]
fn reports_missing_block_close() {
    let ast = parse_markup("{#if ready}<p>ok</p>");

    assert!(ast
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.message == "missing closing {/if} block"));
}

#[test]
fn run_builds_markup_ast() {
    let output = run(
        SourceText::new(
            "Sample.svelte",
            r#"<script lang="ts">
  const answer = 42;
</script>

<section>{answer}</section>
"#,
        ),
        SveltePluginConfig::default(),
    );

    let ast = output.markup.expect("markup ast");
    assert!(ast
        .syntax()
        .text()
        .to_string()
        .contains("<section>{answer}</section>"));
    assert!(!ast.syntax().text().to_string().contains("<script"));
}

#[test]
fn validates_markup_exprs() {
    let output = run(
        SourceText::new(
            "Broken.svelte",
            "<button onclick={() => save(}>Save</button>",
        ),
        SveltePluginConfig::default(),
    );

    assert!(!output.diagnostics.is_empty());
}

#[test]
fn ignores_each_binding() {
    let output = run(
        SourceText::new(
            "Each.svelte",
            "{#each items as item, index}<p>{item.name}</p>{/each}",
        ),
        SveltePluginConfig::default(),
    );

    assert!(output.diagnostics.is_empty());
}

#[test]
fn maps_expr_lines() {
    let source = r#"<script>
  let value = 1;
</script>

<p>{broken(}</p>"#;
    let output = run(
        SourceText::new("Mapped.svelte", source),
        SveltePluginConfig::default(),
    );
    let line_index = SourceText::new("Mapped.svelte", source).line_index();

    assert!(output
        .diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.span)
        .map(|span| line_index.position(span.start).line)
        .any(|line| line == 5));
}
