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
cargo run --locked -p flavor-cli -- check --root . --config flavor.toml
```

## Contribution Loop

This section describes how source-change contributions are shaped so an agent can land a PR without per-change guidance. The aim is to match the historical shape rather than invent new conventions; the examples here are taken from recent merged PRs.

### When to file an issue first

Open <https://github.com/PerishCode/flavor/issues> first when the right shape isn't obvious from a rule's behavior or this AGENTS.md — for example, a new rule with a payload decision, a CLI shape change that affects output stability, or a release-flow tweak. For clear, scoped fixes (a discoverability gap, a misleading exit code, a missing accessor), opening a PR directly is fine; reference any related issue from the PR body via `Closes #N`.

### Branch names

`<area>/<kebab-case-slug>` where `<area>` matches the touched crate or concern. Recent examples:

- `cli/auto-discover-config`
- `cli/warn-empty-scan`
- `config/match-array`
- `cli/rules-subcommand`
- `docs/config-schema`

### Commit messages

Subject: `<area>: <imperative summary>` on one line, ideally ≤ 72 chars. The body explains *why* the change is shaped this way first, then the change list. End with any `Co-Authored-By:` trailers when pair-coded or agent-assisted.

### PR descriptions

Three top-level sections, in order:

```
## Why
<what's broken / missing today; one paragraph or short bullets>

## What
<concrete change list; reference filenames and modules>

## Tests
<commands run + their results, e.g.,
  cargo test --locked -p flavor-cli --test unit  ->  N passed
  cargo fmt --all --check                        ->  clean
  cargo clippy --locked --workspace --all-targets -- -D warnings  ->  clean>
```

Add `## Compatibility` when an output shape, config field, or exit-code behavior moves. Add `## Trade-off worth flagging` when the change has a downside that reviewers should hold in mind.

### Tests

Unit tests live under `crates/flavor-cli/tests/unit/<area>.rs` and are registered in `crates/flavor-cli/tests/unit.rs`:

```rust
// in crates/flavor-cli/tests/unit.rs
#[path = "../src/<file>.rs"]   // only if the touched module isn't already mounted
mod <module>;
#[path = "unit/<area>.rs"]
mod <area>_cases;
```

Tests that need a writable fixture follow the `std::env::temp_dir().join(format!("flavor-<slug>-{pid}-{seq}"))` pattern and clean up with `fs::remove_dir_all` at the end of each case (see `tests/unit/scan.rs` / `tests/unit/config.rs` for live examples). Pure-function tests (no fs) follow the `tests/unit/model.rs` / `tests/unit/naming.rs` shape.

### Pre-PR checks

Every PR must pass the four commands listed under "Common Commands" above. CI re-runs them; matching locally avoids a round-trip.

### Merging

Default to `gh pr merge <num> --merge --delete-branch` once checks are green. Repos that disable merge commits fall back to `--squash`. Agents merge their own PRs after the issue resolution is concrete and CI has passed; no manual approval handoff is part of this loop.
