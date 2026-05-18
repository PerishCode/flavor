# AGENTS

`crates/flavor-plugin-svelte/` owns Svelte descriptor, markup, and syntax facts
on top of `flavor-plugin-core`, with embedded expression validation delegated to
`flavor-plugin-typescript`.

## Directory Rules

- `src/descriptor/` owns Svelte descriptor parsing for top-level script, module
  script, style, and markup regions.
- `src/markup/` owns markup AST, parser, attributes, names, cursor behavior, and
  embedded expression validation.
- `src/facts.rs` owns derived Svelte facts.
- `src/state/` owns `SveltePluginConfig` and `SveltePluginState`.
- `tests/` covers descriptor and markup parser behavior.

Do not add CLI scan/report behavior or product-specific Svelte rules here.
Return descriptor/markup/facts/diagnostics for callers to interpret.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-svelte
```

## Standard Workflow

- Descriptor changes should test duplicate block handling and source line/span
  mapping.
- Markup parser changes should cover node shape, attribute behavior, recovery,
  and embedded expression validation where applicable.
- Keep TypeScript/JavaScript expression parsing in `flavor-plugin-typescript` and map
  diagnostics back to Svelte source offsets.

## FAQ

### Does This Crate Execute Or Compile Svelte?

No. It is AST-only substrate for flavor checks.

### Should Svelte Rule Severity Live Here?

No. Severity, report wording, and action hints belong to `flavor-cli`.
