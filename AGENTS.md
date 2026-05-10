# AGENTS

`flavor` is a personal check-only code flavor lint CLI.

It owns AST-backed code-shape rules, path-scoped checks, report output, bad-flavor notes, and action hints. It does not own product semantics, formatting, rewriting, service execution, repository orchestration, or runtime management.

## Workspace Boundaries

- `crates/flavor-cli/` owns the installable `flavor` binary, rule execution, reports, and CLI-facing scan configuration.
- `crates/flavor-compiler-core/` owns compiler substrate primitives: source text, spans, syntax tree glue, diagnostics, recovery, snapshots, and state/config injection.
- `crates/flavor-compiler-ts/` owns TypeScript/JavaScript/TSX syntax and facts on top of `flavor-compiler-core`.
- `crates/flavor-compiler-vue/` owns Vue SFC/template/style facts on top of `flavor-compiler-core`, and delegates script blocks to `flavor-compiler-ts`.
- Compiler crates expose typed config injection through state; they do not define config file names, discovery rules, or historical ecosystem compatibility.

## Rules

- Keep the CLI check-only.
- Keep rules about syntax, file shape, path shape, and abstract style attributes.
- Do not encode product-specific business concepts in built-in rules.
- Prefer reports that explain the bad flavor and suggest a direction of thought.
- Keep consumer-specific scope in consumer config files.
- Keep root workflow-only scripts under `.github/scripts/`.
- Keep public installation entrypoints under `scripts/manage/`.
- Release and installer downloads use R2 metadata and artifacts as the source of truth.

## Common Commands

```bash
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p flavor-cli -- check --root . --config flavor.json
```
