# AGENTS

`flavor` is a personal check-only code flavor lint CLI.

It owns AST-backed code-shape rules, path-scoped checks, report output, bad-flavor notes, and action hints. It does not own product semantics, formatting, rewriting, service execution, repository orchestration, or runtime management.

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
cargo fmt --all --check --manifest-path app/Cargo.toml
cargo clippy --locked --manifest-path app/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path app/Cargo.toml
cargo run --locked --manifest-path app/Cargo.toml -- check --root . --config flavor.json
```
