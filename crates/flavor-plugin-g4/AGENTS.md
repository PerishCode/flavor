# AGENTS

`crates/flavor-plugin-g4/` owns first-party `.g4` source analysis for flavor.
It connects repo grammar files to the normal plugin delivery pipeline while
delegating grammar contract parsing to `flavor-grammar`.

## Directory Rules

- `src/lib.rs` owns the public analysis entrypoint and output contract.
- `src/plugin.rs` maps analysis output into `GrammarProduct` facts and
  diagnostics.
- `tests/` covers `.g4` fact and diagnostic behavior.

Do not add ANTLR runtime execution, Java generation, or metadata schema
ownership here. This crate reports on `.g4` source shape; `flavor-grammar`
owns contract parsing, schema derivation, and renderers.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-g4
```

## Standard Workflow

- Keep `.g4` parsing behavior aligned with `flavor-grammar`.
- Expose language-neutral facts that help flavor reason about grammar files
  without making ANTLR a runtime dependency.
- Keep diagnostics source-backed by line where possible.
