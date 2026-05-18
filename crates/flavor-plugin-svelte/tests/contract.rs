use flavor_core::SourceText;
use flavor_grammar::{
    parse_contract, parse_contract_values, GrammarContractExpectation,
    GrammarEntryValueExpectation, GrammarSectionExpectation,
};
use flavor_plugin_svelte::{run, SveltePluginConfig};

const SVELTE_METADATA: &str = include_str!("../../../grammars/svelte/metadata.json");
const SVELTE_MARKUP_METADATA: &str = include_str!("../../../grammars/svelte/metadata.json");
const SVELTE_CONTRACT: GrammarContractExpectation<'static> = GrammarContractExpectation {
    name: "svelte",
    directives: &[
        ("owner", "crates/flavor-plugin-svelte"),
        ("entry", "svelte_document"),
    ],
    sections: &[
        GrammarSectionExpectation {
            name: "facts",
            entries: &[
                "descriptor.markup",
                "descriptor.module_script",
                "descriptor.script",
                "descriptor.styles",
                "script.embedded",
            ],
        },
        GrammarSectionExpectation {
            name: "diagnostics",
            entries: &[
                "duplicate.module_script",
                "duplicate.script",
                "missing.close",
            ],
        },
        GrammarSectionExpectation {
            name: "spans",
            entries: &[
                "block.content",
                "block.line",
                "embedded.offset",
                "markup.range",
            ],
        },
        GrammarSectionExpectation {
            name: "recovery",
            entries: &["descriptor.blocks", "embedded.skip", "missing.close"],
        },
    ],
};
const SVELTE_VALUES: &[GrammarEntryValueExpectation<'static>] = &[
    GrammarEntryValueExpectation {
        section: "facts",
        key: "descriptor.module_script",
        contains: &[
            "SvelteBlock",
            "payload.content",
            "payload.start_offset",
            "payload.start_line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "descriptor.script",
        contains: &[
            "SvelteBlock",
            "payload.content",
            "payload.start_offset",
            "payload.start_line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "descriptor.styles",
        contains: &[
            "SvelteFacts.style_count",
            "payload.style_count",
            "payload.style_lines",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "descriptor.markup",
        contains: &[
            "SvelteFacts.markup_lines",
            "payload.line_count",
            "payload.markup_lines",
            "payload.markup_block_count",
            "payload.markup_branch_count",
            "payload.markup_render_count",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "script.embedded",
        contains: &[
            "payload.content",
            "payload.lang",
            "payload.tsx",
            "payload.start_offset",
            "payload.start_line",
        ],
    },
];
const SVELTE_MARKUP_CONTRACT: GrammarContractExpectation<'static> = GrammarContractExpectation {
    name: "svelte-markup",
    directives: &[
        ("owner", "crates/flavor-plugin-svelte"),
        ("entry", "markup_document"),
    ],
    sections: &[
        GrammarSectionExpectation {
            name: "facts",
            entries: &[
                "embedded.expression",
                "markup.block",
                "markup.branch",
                "markup.component",
                "markup.directive",
                "markup.element",
                "markup.render",
                "markup.special",
            ],
        },
        GrammarSectionExpectation {
            name: "diagnostics",
            entries: &[
                "expression.error",
                "missing.block_close",
                "missing.tag_close",
            ],
        },
        GrammarSectionExpectation {
            name: "spans",
            entries: &["diagnostic.range", "host.range", "line", "node.range"],
        },
        GrammarSectionExpectation {
            name: "recovery",
            entries: &["each.binding", "missing.block_close", "snippet.signature"],
        },
    ],
};
const SVELTE_MARKUP_VALUES: &[GrammarEntryValueExpectation<'static>] = &[
    GrammarEntryValueExpectation {
        section: "facts",
        key: "markup.element",
        contains: &[
            "SvelteFacts.markup_elements",
            "SvelteMarkupNameFact",
            "payload.name",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "markup.component",
        contains: &[
            "SvelteFacts.markup_components",
            "SvelteMarkupNameFact",
            "payload.name",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "markup.block",
        contains: &[
            "SvelteFacts.markup_blocks",
            "SvelteMarkupNameFact",
            "payload.name",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "markup.branch",
        contains: &[
            "SvelteFacts.markup_branches",
            "SvelteMarkupNameFact",
            "payload.name",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "markup.render",
        contains: &[
            "SvelteFacts.markup_renders",
            "SvelteMarkupNameFact",
            "payload.name",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "markup.special",
        contains: &[
            "SvelteFacts.markup_specials",
            "SvelteMarkupNameFact",
            "payload.name",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "markup.directive",
        contains: &[
            "SvelteFacts.markup_directives",
            "SvelteMarkupNameFact",
            "payload.name",
            "span",
            "line",
        ],
    },
];

#[test]
fn svelte_descriptor_sections() {
    parse_contract_values(SVELTE_METADATA, &SVELTE_CONTRACT, SVELTE_VALUES).unwrap();
}

#[test]
fn svelte_descriptor_facts() {
    parse_contract(SVELTE_METADATA, &SVELTE_CONTRACT).unwrap();
    let output = run(
        SourceText::new(
            "Contract.svelte",
            "<script module lang=\"ts\">export const prerender = true;</script>\n\
             <script lang=\"ts\">let count = 0;</script>\n\
             <main>{count}</main>\n\
             <style>.root { color: red; }</style>\n",
        ),
        SveltePluginConfig::default(),
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert_eq!(
        output
            .descriptor
            .module_script
            .as_ref()
            .and_then(|block| block.attrs.get("lang"))
            .and_then(|value| value.as_deref()),
        Some("ts")
    );
    assert_eq!(
        output
            .descriptor
            .script
            .as_ref()
            .and_then(|block| block.attrs.get("lang"))
            .and_then(|value| value.as_deref()),
        Some("ts")
    );
    assert_eq!(output.descriptor.styles.len(), 1);
    assert!(output
        .descriptor
        .markup
        .content
        .contains("<main>{count}</main>"));
    assert_eq!(output.facts.script_count, 2);
    assert_eq!(output.facts.style_count, 1);
}

#[test]
fn svelte_descriptor_diagnostics() {
    parse_contract(SVELTE_METADATA, &SVELTE_CONTRACT).unwrap();

    let duplicate_module = run(
        SourceText::new(
            "DuplicateModule.svelte",
            "<script module>const first = 1;</script>\n\
             <script module>const second = 2;</script>",
        ),
        SveltePluginConfig::default(),
    );
    assert_has_diagnostic(
        &duplicate_module.diagnostics,
        "duplicate top-level module <script>",
    );

    let duplicate_script = run(
        SourceText::new(
            "DuplicateScript.svelte",
            "<script>let first = 1;</script>\n<script>let second = 2;</script>",
        ),
        SveltePluginConfig::default(),
    );
    assert_has_diagnostic(
        &duplicate_script.diagnostics,
        "duplicate top-level <script>",
    );

    let missing = run(
        SourceText::new("Missing.svelte", "<style>.root { color: red; }"),
        SveltePluginConfig::default(),
    );
    assert_has_diagnostic(&missing.diagnostics, "missing closing </style>");
}

#[test]
fn svelte_markup_sections() {
    parse_contract_values(
        SVELTE_MARKUP_METADATA,
        &SVELTE_MARKUP_CONTRACT,
        SVELTE_MARKUP_VALUES,
    )
    .unwrap();
}

#[test]
fn svelte_markup_facts() {
    parse_contract(SVELTE_MARKUP_METADATA, &SVELTE_MARKUP_CONTRACT).unwrap();
    let output = run(
        SourceText::new(
            "Markup.svelte",
            r#"<main bind:value={value} on:click|once={save} use:action>
  <Panel>{value}</Panel>
  {#if ready}{@render children?.()}{:else}<span />{/if}
  {@html raw}
</main>"#,
        ),
        SveltePluginConfig::default(),
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert_eq!(output.facts.markup_element_count, 2);
    assert_eq!(output.facts.markup_component_count, 1);
    assert_eq!(output.facts.markup_block_count, 1);
    assert_eq!(output.facts.markup_branch_count, 1);
    assert_eq!(output.facts.markup_render_count, 1);
    assert_eq!(output.facts.markup_special_count, 1);
    assert_eq!(output.facts.markup_directive_count, 3);
    assert_eq!(output.facts.markup_elements.len(), 2);
    assert_eq!(output.facts.markup_components.len(), 1);
    assert_eq!(output.facts.markup_blocks.len(), 1);
    assert_eq!(output.facts.markup_branches.len(), 1);
    assert_eq!(output.facts.markup_renders.len(), 1);
    assert_eq!(output.facts.markup_specials.len(), 1);
    assert_eq!(output.facts.markup_directives.len(), 3);

    let main = output
        .facts
        .markup_elements
        .iter()
        .find(|element| element.name == "main")
        .expect("main element fact");
    assert!(output.source.slice(main.span).starts_with("<main "));

    let panel = output
        .facts
        .markup_components
        .iter()
        .find(|component| component.name == "Panel")
        .expect("Panel component fact");
    assert!(output.source.slice(panel.span).starts_with("<Panel>"));

    let if_block = output
        .facts
        .markup_blocks
        .iter()
        .find(|block| block.name == "if")
        .expect("if block fact");
    assert!(output
        .source
        .slice(if_block.span)
        .starts_with("{#if ready}"));

    let else_branch = output
        .facts
        .markup_branches
        .iter()
        .find(|branch| branch.name == "else")
        .expect("else branch fact");
    assert!(output.source.slice(else_branch.span).starts_with("{:else}"));

    let render = output
        .facts
        .markup_renders
        .iter()
        .find(|render| render.name == "render")
        .expect("render fact");
    assert!(output.source.slice(render.span).starts_with("{@render"));

    let html = output
        .facts
        .markup_specials
        .iter()
        .find(|special| special.name == "html")
        .expect("html special fact");
    assert_eq!(output.source.slice(html.span), "{@html raw}");

    let bind = output
        .facts
        .markup_directives
        .iter()
        .find(|directive| directive.name.starts_with("bind:"))
        .expect("bind directive fact");
    assert!(output.source.slice(bind.span).starts_with("bind:value"));
}

#[test]
fn svelte_markup_diagnostics() {
    parse_contract(SVELTE_MARKUP_METADATA, &SVELTE_MARKUP_CONTRACT).unwrap();

    let missing_block = run(
        SourceText::new("MissingBlock.svelte", "{#if ready}<p>ok</p>"),
        SveltePluginConfig::default(),
    );
    assert_has_diagnostic(&missing_block.diagnostics, "missing closing {/if} block");

    let missing_tag = run(
        SourceText::new("MissingTag.svelte", "<main><span></main>"),
        SveltePluginConfig::default(),
    );
    assert_has_diagnostic(&missing_tag.diagnostics, "missing closing </span>");

    let expression = run(
        SourceText::new(
            "Expression.svelte",
            "<button onclick={() => save(}>Save</button>",
        ),
        SveltePluginConfig::default(),
    );
    assert_has_diagnostic(
        &expression.diagnostics,
        "expected ')' to close call arguments",
    );
}

#[test]
fn svelte_recovery_maps_spans() {
    parse_contract(SVELTE_MARKUP_METADATA, &SVELTE_MARKUP_CONTRACT).unwrap();
    let source = "<script>let value = 1;</script><main><span></main><button>{broken(}</button>";
    let output = run(
        SourceText::new("Recover.svelte", source),
        SveltePluginConfig::default(),
    );

    assert!(output.descriptor.errors.is_empty());
    assert_eq!(output.descriptor.markup.content.len(), source.len());
    assert!(!output.descriptor.markup.content.contains("<script"));
    assert_eq!(output.facts.markup_element_count, 3);
    let button = output
        .facts
        .markup_elements
        .iter()
        .find(|element| element.name == "button")
        .expect("button element fact after recovery");
    assert_eq!(
        output.source.slice(button.span),
        "<button>{broken(}</button>"
    );

    let missing = find_diagnostic(&output.diagnostics, "missing closing </span>");
    assert_eq!(
        missing.span.map(|span| span.start as usize),
        source.find("<span>")
    );

    let expression = find_diagnostic(&output.diagnostics, "expected ')' to close call arguments");
    let expression_span = expression.span.expect("expression span");
    assert!(
        source[..expression_span.start as usize].ends_with("broken("),
        "expression span was not mapped into host source: {expression:?}"
    );
}

fn find_diagnostic<'a>(
    diagnostics: &'a [flavor_core::Diagnostic],
    message: &str,
) -> &'a flavor_core::Diagnostic {
    diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains(message))
        .unwrap_or_else(|| panic!("missing diagnostic containing `{message}`: {diagnostics:?}"))
}

fn assert_has_diagnostic(diagnostics: &[flavor_core::Diagnostic], message: &str) {
    let diagnostic = find_diagnostic(diagnostics, message);
    assert_eq!(diagnostic.code.as_deref(), Some("svelte/parse/error"));
    assert!(diagnostic.span.is_some(), "missing span for {diagnostic:?}");
}
