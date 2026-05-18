# AGENTS

`crates/flavor-plugin-vue/` owns Vue SFC and template facts on top of
`flavor-plugin-core`, with embedded expression validation delegated to
`flavor-plugin-typescript`.

## Directory Rules

- `src/sfc/` owns Vue SFC descriptor parsing and block validation.
- `src/template/` owns template AST, template parsing, names, and embedded
  expression validation.
- `src/style/` owns style-facing substrate.
- `src/facts/` and `src/visit/` own derived Vue facts and traversal hooks.
- `src/state/` owns `VuePluginConfig`, `TemplateConfig`, and
  `VuePluginState`.
- `harness/cases/` contains representative Vue fixtures.
- `tests/` covers SFC descriptors, template parsing, run output, and harness
  behavior.

Do not add CLI scan/report behavior or product-specific Vue rules here. Return
descriptor/template/facts/diagnostics for callers to interpret.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-vue
```

## Standard Workflow

- Descriptor changes should test duplicate block handling, line/span mapping,
  and top-level block constraints.
- Template parser changes should include parser tests or a harness fixture.
- When validating embedded expressions, keep TypeScript-specific parsing in
  `flavor-plugin-typescript` and map diagnostics back to Vue source offsets.

## FAQ

### Does This Crate Own Vue Style Rules?

No. It owns Vue syntax/facts/diagnostics. Rule decisions and report wording
belong to `flavor-cli`.

### Should Script Expression Parsing Be Reimplemented Here?

No. Delegate TypeScript/JavaScript expression parsing to
`flavor-plugin-typescript` where possible.
