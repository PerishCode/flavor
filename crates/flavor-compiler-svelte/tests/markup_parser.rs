use flavor_compiler_core::{RawSyntaxKind, SourceText};
use flavor_compiler_svelte::{
    markup::{parse_markup, SvelteMarkupKind},
    run, SvelteCompilerConfig,
};

fn has_node(ast: &flavor_compiler_svelte::SvelteMarkupAst, kind: SvelteMarkupKind) -> bool {
    ast.syntax()
        .descendants()
        .any(|node| node.kind() == RawSyntaxKind::from(kind))
}

fn token_count(ast: &flavor_compiler_svelte::SvelteMarkupAst, kind: SvelteMarkupKind) -> usize {
    ast.syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() == RawSyntaxKind::from(kind))
        .count()
}

#[test]
fn parses_elements_and_components() {
    let source = "<main><Panel title=\"Hi\">{message}</Panel><img /></main>";
    let ast = parse_markup(source);

    assert_eq!(ast.syntax().text().to_string(), source);
    assert!(has_node(&ast, SvelteMarkupKind::Element));
    assert!(has_node(&ast, SvelteMarkupKind::Component));
    assert!(has_node(&ast, SvelteMarkupKind::Mustache));
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
            .filter(|node| node.kind() == RawSyntaxKind::from(SvelteMarkupKind::Block))
            .count(),
        2
    );
    assert!(has_node(&ast, SvelteMarkupKind::BlockBranch));
    assert!(has_node(&ast, SvelteMarkupKind::RenderTag));
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
            .filter(|node| node.kind() == RawSyntaxKind::from(SvelteMarkupKind::Directive))
            .count(),
        4
    );
    assert!(has_node(&ast, SvelteMarkupKind::SpreadAttribute));
    assert!(has_node(&ast, SvelteMarkupKind::ShorthandAttribute));
    assert_eq!(token_count(&ast, SvelteMarkupKind::DirectiveArgument), 4);
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
            .filter(|node| node.kind() == RawSyntaxKind::from(SvelteMarkupKind::Block))
            .count(),
        3
    );
    assert_eq!(
        ast.syntax()
            .descendants()
            .filter(|node| node.kind() == RawSyntaxKind::from(SvelteMarkupKind::BlockBranch))
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
    assert!(has_node(&ast, SvelteMarkupKind::Component));
    assert!(has_node(&ast, SvelteMarkupKind::SpecialTag));
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
        SvelteCompilerConfig::default(),
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
        SvelteCompilerConfig::default(),
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
        SvelteCompilerConfig::default(),
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
        SvelteCompilerConfig::default(),
    );
    let line_index = SourceText::new("Mapped.svelte", source).line_index();

    assert!(output
        .diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.span)
        .map(|span| line_index.position(span.start).line)
        .any(|line| line == 5));
}
