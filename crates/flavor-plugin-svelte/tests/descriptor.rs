use flavor_plugin_core::SourceText;
use flavor_plugin_svelte::{run, SveltePluginConfig};

#[test]
fn parses_svelte_descriptor() {
    let output = run(
        SourceText::new(
            "Sample.svelte",
            r#"<script module lang="ts">
  export const prerender = true;
</script>

<script lang="ts">
  let count = $state(0);
</script>

<main>{count}</main>

<style>
  main { color: red; }
</style>
"#,
        ),
        SveltePluginConfig::default(),
    );

    assert!(output.diagnostics.is_empty());
    assert_eq!(
        output
            .descriptor
            .module_script
            .as_ref()
            .and_then(|block| block.attrs.get("lang"))
            .and_then(|value| value.as_deref()),
        Some("ts")
    );
    assert!(output
        .descriptor
        .script
        .as_ref()
        .is_some_and(|block| block.content.contains("$state")));
    assert_eq!(output.descriptor.styles.len(), 1);
    assert!(output
        .descriptor
        .markup
        .content
        .contains("<main>{count}</main>"));
    assert!(!output.descriptor.markup.content.contains("<script"));
}

#[test]
fn recognizes_legacy_module_context() {
    let output = run(
        SourceText::new(
            "Sample.svelte",
            r#"<script context="module">export const ssr = true;</script>
<p>ok</p>"#,
        ),
        SveltePluginConfig::default(),
    );

    assert!(output.descriptor.module_script.is_some());
    assert!(output.descriptor.script.is_none());
}

#[test]
fn maps_script_line_offsets() {
    let output = run(
        SourceText::new(
            "Sample.svelte",
            "\n\n<script lang=\"ts\">\nconst answer = 42;\n</script>",
        ),
        SveltePluginConfig::default(),
    );

    assert_eq!(
        output
            .descriptor
            .script
            .as_ref()
            .map(|block| block.start_line),
        Some(2)
    );
}

#[test]
fn reports_duplicate_scripts() {
    let output = run(
        SourceText::new(
            "Sample.svelte",
            "<script>let a = 1;</script>\n<script>let b = 2;</script>",
        ),
        SveltePluginConfig::default(),
    );

    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("duplicate top-level <script>")));
}

#[test]
fn reports_missing_closing_block() {
    let output = run(
        SourceText::new("Sample.svelte", "<style>.panel { color: red; }"),
        SveltePluginConfig::default(),
    );

    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("missing closing </style>")));
}
