# AGENTS

`crates/flavor-plugin-rust/` owns the Rust plugin adapter, public Rust fact
model, config/state, and internal Rust syntax analysis on top of `flavor-core`.

## Directory Rules

- `src/lib.rs` owns the public crate facade and analysis entrypoint.
- `src/plugin.rs` owns the flavor internal plugin adapter that satisfies host
  grammar/product requests.
- `src/model.rs` owns public Rust output and fact structs.
- `src/state.rs` owns typed Rust plugin config/state.
- `src/internal/` owns Rust grammar constants, grammar parse invocation, fact
  collection, and repeated-pattern heuristics. Parser execution and raw AST
  construction are delegated to `flavor-grammar`.
- `tests/` covers Rust frontend facts and diagnostics.

Do not add CLI scan/report behavior or consumer-specific Rust rules here.
Return facts/diagnostics for callers to interpret.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-rust
```

## Standard Workflow

- Keep parser execution behind `flavor-grammar`; the Rust plugin may provide
  the tree-sitter Rust language handle but must not instantiate parser engines
  or construct raw CSTs directly.
- Raw CST shape must continue to come from the grammar-owned runtime schema and
  raw AST adapter.
- Preserve source-backed spans and deterministic diagnostics.
- Add focused fact tests when rule-facing Rust syntax behavior moves.
