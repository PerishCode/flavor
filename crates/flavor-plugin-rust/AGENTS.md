# AGENTS

`crates/flavor-plugin-rust/` owns Rust syntax and lint facts on top of
`flavor-plugin-core`.

## Directory Rules

- `src/lib.rs` owns the public plugin analysis entrypoint and output contract.
- `src/facts.rs` owns Rust facts collected from the parser tree.
- `src/state.rs` owns typed Rust plugin config/state.
- `tests/` covers Rust frontend facts and diagnostics.

Do not add CLI scan/report behavior or consumer-specific Rust rules here.
Return facts/diagnostics for callers to interpret.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-rust
```

## Standard Workflow

- Keep parser implementation details behind flavor-owned facts.
- Preserve source-backed spans and deterministic diagnostics.
- Add focused fact tests when rule-facing Rust syntax behavior moves.
