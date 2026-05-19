# AGENTS

`crates/flavor-plugin-vue/` owns Vue SFC and template facts on top of
`flavor-core`, with embedded expression validation delegated to
`flavor-plugin-typescript`.

## Directory Rules

- `src/lib.rs`, `src/model.rs`, `src/plugin.rs`, and `src/state/` own the
  public analyzer facade, output model, first-party plugin delivery, and
  injected state/config.
- `src/sfc/` owns internal Vue SFC descriptor parsing and block validation.
- `src/template/` owns internal template AST, template parsing, names, and embedded
  expression validation.
- `src/template/kind.rs` owns plugin-local string kind constants and schema
  loading from `grammars/vue`; it is not a public API.
- `src/style/` owns style-facing substrate.
- `src/facts/` consumes the grammar-owned dynamic tree view to derive Vue facts.
  `src/visit/` owns traversal hooks.
- `harness/cases/` contains representative Vue fixtures.
- `tests/` covers SFC descriptors, template parsing, run output, and harness
  behavior.

Do not add CLI scan/report behavior or product-specific Vue rules here. Return
model-level descriptor/template/facts/diagnostics for callers to interpret.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-vue
```

## Standard Workflow

- Descriptor changes should test duplicate block handling, line/span mapping,
  and top-level block constraints.
- Template parser changes should include parser tests or a harness fixture.
- Keep the descriptor/template parsers as staged parser backends for this
  refactor; template raw CST node/token kinds must continue to come from the
  Vue template G4 raw AST schema and raw AST construction must go through
  `flavor-grammar`.
- When validating embedded expressions, keep TypeScript-specific parsing in
  `flavor-plugin-typescript` and map diagnostics back to Vue source offsets.

## FAQ

### Does This Crate Own Vue Style Rules?

No. It owns Vue syntax/facts/diagnostics. Rule decisions and report wording
belong to `flavor-cli`.

### Should Script Expression Parsing Be Reimplemented Here?

No. Delegate TypeScript/JavaScript expression parsing to
`flavor-plugin-typescript` where possible.
