# AGENTS

`crates/flavor-core/` owns shared source, syntax, diagnostic, product, and
state primitives used by first-party syntax crates.

## Directory Rules

- `src/source/` owns `SourceText`, `LineIndex`, positions, and spans.
- `src/syntax/` owns rowan-backed syntax tree glue, schema-aware builders,
  cursors, tokens, trivia, and language-neutral syntax wrappers.
- `src/report/` owns diagnostics, recovery sets, and snapshot dumps.
- `src/state/` owns typed flavor-core config/state injection.
- `tests/` covers substrate behavior and should stay language-neutral.

Do not add TypeScript, Vue, Svelte, CLI scan discovery, report output, or config
file discovery here.

## Common Commands

```bash
cargo test --locked -p flavor-core
```

## Standard Workflow

- Keep APIs small and typed. Callers should compose these primitives without
  reaching around them.
- Use schema-aware syntax builder methods for language raw AST nodes/tokens;
  keep raw token insertion for core trivia.
- Changes to spans, line indexing, diagnostics, or syntax wrappers can affect
  every frontend. Prefer focused tests in this crate plus downstream checks when
  behavior moves.
- Recovery and snapshot behavior should be deterministic; avoid embedding
  environment-specific paths or unstable ordering.

## FAQ

### Does This Crate Know About `flavor.json`?

No. It only exposes typed config/state primitives. Config names and discovery
belong to `flavor-cli`.

### Can This Crate Contain Language Grammar?

No. Keep grammar and language facts in the relevant plugin crate.
