# AGENTS

`crates/flavor-plugin-typescript/` owns TypeScript, JavaScript, and TSX syntax and
facts on top of `flavor-core`.

## Directory Rules

- `src/lib.rs`, `src/model.rs`, `src/plugin.rs`, and `src/state/` own the
  public analyzer facade, output model, first-party plugin delivery, and
  injected state/config.
- `src/lexer/` owns internal token scanning.
- `src/parser/` owns the internal TypeScript, JavaScript, and JSX/TSX grammar.
- `src/ast/` owns the internal source file wrapper. `src/facts/` consumes the
  grammar-owned dynamic tree view to derive facts. `src/visit/` owns traversal
  hooks.
- `src/internal/grammar.rs` owns plugin-local string kind constants and schema
  loading from `grammars/typescript`; it is not a public API.
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
  schema and raw AST construction must go through `flavor-grammar`.
- If string kind constants move, update the TypeScript G4 surface, parser, raw
  tree fact consumers, and tests together.
- Embedded expression users, such as Vue and Svelte frontends, must use the
  public analyzer facade rather than internal lexer/parser modules.

## FAQ

### Should This Crate Decide Rule Severity?

No. It should expose syntax, facts, and diagnostics. Rule severity and reporting
belong to `flavor-cli`.

### Should JSX/TSX Live Elsewhere?

No. TSX grammar is part of this TypeScript frontend.
