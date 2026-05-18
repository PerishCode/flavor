# AGENTS

`crates/flavor-grammar/` owns grammar contract metadata, `.g4` source indexing,
raw AST schema derivation, and adapter rendering for flavor.

## Directory Rules

- `src/lib.rs` owns the public grammar source-shape and metadata contract API.
- `src/source.rs` owns the lightweight `.g4` rule/token index used to validate
  raw AST contract symbols and derive raw AST schemas.
- `src/render.rs` owns Rust syntax-kind enum rendering from raw AST schemas,
  generated node/token category helpers, the schema trait impl consumed by
  `SyntaxBuilder`, and raw AST adapter rendering from G4/metadata bindings.
- `src/metadata.rs` owns the `metadata.json` contract parser.
- `tests/` validates every `metadata.json` file and referenced `.g4` file
  under the repository `grammars/` directory.

Do not add language parser implementation or lint rule execution here.

## Common Commands

```bash
cargo test --locked -p flavor-grammar
```

## Standard Workflow

- Keep the grammar source-shape and metadata formats strict and boring.
- Add validation before relying on new grammar sections from implementation
  crates.
- Parser backends may remain staged adapters in plugin crates, but generated
  raw AST syntax kinds, node/token categories, and adapter binding renderers
  belong here.
- This crate should remain independent from CLI config and report behavior.
