# AGENTS

`flavor` is a personal check-only code flavor lint CLI.

It owns AST-backed code-shape rules, path-scoped checks, report output,
bad-flavor notes, and action hints. It does not own product semantics,
formatting, rewriting, service execution, repository orchestration, or runtime
management.

## Directory Rules

### Repository Shape

- `crates/` contains the Rust workspace crates. Each core crate owns its local
  `AGENTS.md`; read the child file before editing that subtree.
- `.github/workflows/` contains CI and release workflows.
- `.github/scripts/` contains workflow-only helper scripts. Keep workflow-only
  scripts there.
- `runseal.toml` and `.runseal/wrappers/` are the repo-local operator entrypoints
  for support tasks that do not belong in the installable `flavor` product
  binary. Current support commands are `runseal :cloudflare`, `runseal :pr`,
  and `runseal :release`.
- `grammars/` contains the repo-visible `.g4` grammar source of truth plus
  `metadata.json` contract metadata. Parser backends, facts, diagnostics, and
  harnesses should align to these files.
- `scripts/` contains both developer helpers (`scripts/dev/`) and the
  repo-local uv-managed Python support command tree used by runseal wrappers.
- `.local/` is repo-local private operator state (for example Cloudflare
  secrets). It must stay gitignored and must not become a source of truth for
  product behavior.
- `scripts/init.py` is the idempotent post-clone initializer. It quick-fails on
  missing required tools or repository entrypoints, installs local hooks, and
  exits cleanly only when the checkout is ready for development.
- `manage.sh` and `manage.ps1` are the public install/uninstall entrypoints at
  the repository root.
- Release and manager downloads use R2 metadata and artifacts as the source of
  truth.

### Recursive AGENTS Index

- `crates/flavor-cli/AGENTS.md`: installable `flavor` binary, scan config,
  rule execution, reports, and CLI-facing behavior.
- `crates/flavor-core/AGENTS.md`: shared source text, spans, syntax tree glue,
  diagnostics, recovery, snapshots, product primitives, and typed state/config
  injection.
- `crates/flavor-shared/AGENTS.md`: first-party plugin implementation helpers
  that should not become public ABI.
- `crates/flavor-plugin-filesystem/AGENTS.md`: filesystem/source path and shape
  bundled plugin identity and behavior.
- `crates/flavor-plugin-g4/AGENTS.md`: `.g4` source analysis plugin identity
  and behavior.
- `crates/flavor-plugin-python/AGENTS.md`: Python syntax facts and code-shape
  frontend behavior.
- `crates/flavor-plugin-rust/AGENTS.md`: Rust syntax/facts frontend and
  embedded Rust lint facts.
- `crates/flavor-plugin-typescript/AGENTS.md`: TypeScript, JavaScript, and TSX
  lexer, parser, raw tree facts, visitor, and frontend state.
- `crates/flavor-plugin-vue/AGENTS.md`: Vue SFC descriptor, template/style
  facts, template parsing, and embedded expression validation.
- `crates/flavor-plugin-svelte/AGENTS.md`: Svelte descriptor, markup parsing,
  facts, and embedded expression validation.
- `crates/flavor-grammar/AGENTS.md`: grammar contract metadata, `.g4` source
  indexing, raw AST schema derivation, runtime kind lookup, parser backend
  adapters, dynamic grammar tree views, and validation harnesses.

When adding or removing a core subtree, update this index in the same change.
Child `AGENTS.md` files should stay local: ownership, directory shape, commands,
workflow notes, and FAQ for that subtree.

### Project Boundaries

- Keep the CLI check-only.
- Keep rules about syntax, file shape, path shape, and abstract style attributes.
- Do not encode product-specific business concepts in built-in rules.
- Prefer reports that explain the bad flavor and suggest a direction of thought.
- Keep consumer-specific scope in consumer config files.
- Plugin crates may expose typed config injection through state. They do not
  define config file names, discovery rules, report rendering, or historical
  ecosystem compatibility.

## Common Commands

```bash
python3 scripts/init.py
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p flavor-cli -- check --root . --config flavor.json
python3 scripts/dev/antlr.py check
runseal :pr --help
```

`python3 scripts/init.py` is the default post-clone command. It requires
`runseal` so repo-local support commands have one entrypoint shape. Use
`--force` only when intentionally replacing existing non-init hooks; the script
backs them up first.
`python3 scripts/dev/antlr.py check` is an optional Dockerized ANTLR validation
helper. It lazily builds its Docker image when needed, checks `.g4` files under
`grammars/` in ANTLR dependency mode, and does not generate Java artifacts.

## Standard Workflow

### Initialize

After cloning or when hooks look stale, run:

```bash
python3 scripts/init.py
```

The generated hooks contain their concrete actions directly. The pre-commit hook
currently runs fmt, cargo check, flavor self-check, shell syntax checks, and
PowerShell syntax checks when `pwsh` is available. The commit-msg hook validates
the commit subject shape.

### When To File An Issue First

Open <https://github.com/PerishCode/flavor/issues> first when the right shape is
not obvious from behavior or this AGENTS tree. Examples include a new rule with a
payload decision, a CLI shape change that affects output stability, or a release
flow tweak. For clear, scoped fixes, open a PR directly and reference any related
issue from the PR body with `Closes #N`.

### Branch Names

Use `<area>/<kebab-case-slug>`, where `<area>` matches the touched crate or
concern. Recent examples:

- `cli/auto-discover-config`
- `cli/warn-empty-scan`
- `config/match-array`
- `cli/rules-subcommand`
- `docs/config-schema`

### Commit Messages

Subject: `<area>: <imperative summary>` on one line, ideally <= 72 characters.
The body explains why the change is shaped this way first, then the change list.
End with any `Co-Authored-By:` trailers when pair-coded or agent-assisted.

### Tests

Unit tests for `flavor-cli` live under `crates/flavor-cli/tests/unit/<area>.rs`
and are registered in `crates/flavor-cli/tests/unit.rs`:

```rust
// in crates/flavor-cli/tests/unit.rs
#[path = "../src/<file>.rs"]   // only if the touched module is not already mounted
mod <module>;
#[path = "unit/<area>.rs"]
mod <area>_cases;
```

Tests that need a writable fixture follow the
`std::env::temp_dir().join(format!("flavor-<slug>-{pid}-{seq}"))` pattern and
clean up with `fs::remove_dir_all` at the end of each case. See
`tests/unit/scan.rs` and `tests/unit/config.rs` for live examples. Pure-function
tests follow `tests/unit/model.rs` and `tests/unit/naming.rs`.

### Pre-PR Checks

Every PR must pass these commands before review:

```bash
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p flavor-cli -- check --root . --config flavor.json
```

CI reruns them across Linux, Windows, and macOS.

### PR Descriptions

Use these top-level sections, in order:

```markdown
## Why
<what is broken or missing today>

## What
<concrete change list; reference filenames and modules>

## Tests
<commands run and results>
```

Add `## Compatibility` when an output shape, config field, or exit-code behavior
moves. Add `## Trade-off worth flagging` when the change has a downside that
reviewers should hold in mind.

### Merging

`main` is PR-only and protected by the `guard` workflow. Required approvals are
intentionally `0`; the guard matrix is the merge gate.

After opening a non-draft PR, default to enabling repository auto-merge:

```bash
gh pr merge <num> --auto --squash --delete-branch
```

Do not add workflow files just to auto-enable auto-merge. If auto-merge cannot
be enabled or the repository disables merge commits, wait for green checks and
fall back to the smallest equivalent manual command, usually
`gh pr merge <num> --squash --delete-branch`. Agents merge their own PRs after
the issue resolution is concrete and CI has passed; no manual approval handoff
is part of this loop.

## FAQ

### Does `flavor` Format Or Rewrite Code?

No. The CLI is check-only. It reports bad flavor and action hints, but it does
not format, rewrite, run services, or manage runtime state.

### Where Do Consumer-Specific Rules Belong?

In consumer config files. Built-in rules stay about syntax, file shape, path
shape, and abstract style attributes.

### Which Crate Owns Config Discovery?

`flavor-cli` owns config file names, discovery, scan setup, report output, and
exit behavior. Plugin crates only expose plugin-facing inputs, facts,
diagnostics, and typed state/config injection where needed.

### Where Do Installer Changes Go?

Public install/uninstall entrypoints live at the repository root as `manage.sh`
and `manage.ps1`. Release and smoke scripts should reference those root files.

### Where Do Workflow Helper Scripts Go?

Workflow-only helpers belong under `.github/scripts/`. The repository
initialization entrypoint is `scripts/init.py`; additional local developer
harness helpers, if added, belong under `scripts/dev/`.

### Does Init Replace Existing Hooks?

Not by default. If a hook exists and was not generated by `scripts/init.py`, the
script stops and asks for `--force`. With `--force`, it backs up the existing
hook to a numbered `.bak` path before replacing it. Older bootstrap-generated
hooks are treated as generated and are replaced idempotently.
