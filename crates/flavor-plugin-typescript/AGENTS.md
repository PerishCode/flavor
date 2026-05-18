# AGENTS

`crates/flavor-plugin-typescript/` owns TypeScript, JavaScript, and TSX syntax and
facts on top of `flavor-core`.

## Directory Rules

- `src/lexer/` owns token scanning.
- `src/parser/` owns TypeScript, JavaScript, and JSX/TSX grammar.
- `src/ast/`, `src/facts/`, and `src/visit/` own typed AST access, derived facts,
  and traversal hooks.
- `src/state/` owns `TsPluginConfig` and `TsPluginState`.
- `build.rs` derives `TsSyntaxKind` from the TypeScript G4 raw AST schema.
- `src/syntax_kind.rs` includes generated frontend syntax kind definitions.
- `harness/cases/` contains representative parser fixtures.
- `tests/` covers scanner, parser, run output, and harness behavior.

Do not add CLI rule/report behavior or config discovery here. This crate should
return AST/facts/diagnostics for callers to interpret.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-typescript
```

## Standard Workflow

- Parser changes should include direct parser tests or a harness fixture when
  the syntax shape is meaningful.
- Keep recovery deterministic and diagnostics source-backed.
- Keep the plugin lexer/parser as the staged parser backend for this refactor;
  raw CST node/token kinds must continue to come from the TypeScript G4 raw AST
  schema and schema-aware builder paths.
- If syntax kinds move, update the TypeScript G4 surface, parser, AST/fact
  consumers, and tests together.
- Embedded expression users, such as Vue and Svelte frontends, depend on this
  crate staying language-frontend oriented rather than CLI-oriented.

## FAQ

### Should This Crate Decide Rule Severity?

No. It should expose syntax, facts, and diagnostics. Rule severity and reporting
belong to `flavor-cli`.

### Should JSX/TSX Live Elsewhere?

No. TSX grammar is part of this TypeScript frontend.
