# AGENTS

`crates/flavor-grammar/` owns grammar contract metadata, `.g4` source indexing,
raw AST schema derivation, runtime kind lookup, parser backend orchestration,
parser backend adapters, and dynamic grammar tree views for flavor.

## Directory Rules

- `src/lib.rs` owns the public grammar source-shape and metadata contract API.
- `src/source.rs` owns the lightweight `.g4` rule/token index used to validate
  raw AST contract symbols.
- `src/schema.rs` owns `RawAstSchema`, `GrammarSpec`, and `GrammarBundle`.
  Schema lookup is runtime string-kind resolution with indexed raw/name maps;
  do not add generated syntax kind types or enum renderers.
- `src/raw_builder.rs` owns the grammar-facing raw AST construction helper used
  by parser backends and adapters. It resolves string kind shapes through
  `RawAstSchema`; do not add generated syntax kind types.
- `src/parse.rs` owns parser output contracts shared by grammar-owned parser
  backends.
- `src/tree_sitter_raw.rs` owns the optional `tree-sitter-backend` generic
  tree-sitter parse orchestration and flavor raw AST builder. Backend bindings
  are interpreted from `metadata.json` and `.g4` comments at runtime.
- `src/metadata.rs` owns the `metadata.json` contract parser.
- `src/view.rs` owns dynamic grammar tree helpers and language-neutral AST
  query atoms over flavor raw CSTs. Keep it string-kind and metadata-value
  oriented; do not introduce typed AST models or language-specific feature
  helpers.
- `tests/` validates every `metadata.json` file and referenced `.g4` file
  under the repository `grammars/` directory.

Do not add lint rule execution here. Parser backends may be staged behind
compiled grammar/runtime helpers, but plugin-facing typed AST models should not
live here.

## Common Commands

```bash
cargo test --locked -p flavor-grammar
cargo test --locked -p flavor-grammar --features tree-sitter-backend
```

## Standard Workflow

- Keep the grammar source-shape and metadata formats strict and boring.
- Add validation before relying on new grammar sections from implementation
  crates.
- Parser backend engines may remain staged in plugin crates only as tracked
  migration debt. Parser execution orchestration, raw AST schemas, node/token
  categories, raw AST builders, backend adapters, and dynamic raw tree views
  belong here.
- AST interpretation helpers in this crate must stay atomic and
  language-neutral. Plugins may compose them into Rust/TS/Vue/Svelte facts, but
  new raw tree traversal strategies should not first appear as ad hoc plugin
  loops.
- This crate should remain independent from CLI config and report behavior.
