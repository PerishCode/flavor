# AGENTS

`crates/flavor-plugin-rust/` owns Rust syntax and lint facts on top of
`flavor-core`.

## Directory Rules

- `src/lib.rs` owns the public plugin analysis entrypoint and output contract.
- `build.rs` wires Rust `.g4` and `metadata.json` into `flavor-grammar` generated
  syntax-kind and adapter bindings.
- `src/raw_ast.rs` owns the current tree-sitter to G4-schema rowan CST adapter;
  raw AST node/token mapping code should stay generated from `flavor-grammar`.
- `src/facts.rs` owns Rust facts collected from the parser tree.
- `src/state.rs` owns typed Rust plugin config/state.
- `src/syntax_kind.rs` includes generated frontend syntax kind definitions.
- `tests/` covers Rust frontend facts and diagnostics.

Do not add CLI scan/report behavior or consumer-specific Rust rules here.
Return facts/diagnostics for callers to interpret.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-rust
```

## Standard Workflow

- Keep parser implementation details behind flavor-owned facts.
- Keep tree-sitter as the staged parser backend for this refactor; raw CST
  shape must continue to come from G4-generated syntax kinds and adapter
  bindings.
- Preserve source-backed spans and deterministic diagnostics.
- Add focused fact tests when rule-facing Rust syntax behavior moves.
