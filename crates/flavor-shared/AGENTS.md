# AGENTS

`crates/flavor-shared/` owns first-party implementation helpers for flavor
plugin crates.

## Directory Rules

- `src/state.rs` owns reusable typed config/state helpers for plugin
  implementations.
- `src/product.rs` owns small helpers for producing `flavor-core` product facts.

Keep this crate out of the public ABI boundary. Do not move source text, spans,
diagnostics, syntax tree primitives, product model types, report models, CLI
scan behavior, grammar compilation helpers, or rule semantics here.

## Common Commands

```bash
cargo test --locked -p flavor-shared
```

## Standard Workflow

- Add helpers only after at least two first-party call sites show the same
  implementation shape.
- Prefer plain functions and small structs over macros.
- Keep parser traversal and language-specific fact semantics in the owning
  plugin crate.
