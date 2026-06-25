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
  binary. Current support commands are `runseal :antlr`, `runseal :cloudflare`,
  `runseal :pr`, and `runseal :release`.
- `grammars/` contains the repo-visible `.g4` grammar source of truth plus
  `metadata.json` contract metadata. Parser backends, facts, diagnostics, and
  harnesses should align to these files.
- `scripts/` contains the repo-local uv-managed Python support command tree
  used by runseal wrappers.
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
cargo run --locked -p flavor-cli -- check --root . --config flavor.toml
runseal :antlr init
runseal :antlr check
runseal :pr --help
```

`python3 scripts/init.py` is the default post-clone command. It requires
`runseal` so repo-local support commands have one entrypoint shape. Use
`--force` only when intentionally replacing existing non-init hooks; the script
backs them up first.
`runseal :antlr init` builds the pinned local ANTLR Docker image.
`runseal :antlr check` is an optional Dockerized ANTLR validation helper. It
requires the image prepared by init, checks `.g4` files under `grammars/` in
ANTLR dependency mode, and does not generate Java artifacts.

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
flow tweak. For clear, scoped fixes that do not need the issue hot follow-up
loop below, open a PR directly and reference any related issue from the PR body
with `Closes #N`.

### Issue Hot Follow-Up Flow

Use this flow for active issue follow-up where the reporter needs to verify a
real installed CLI before the fix is considered resolved:

1. Create a branch from `main`, complete the code change there, and run
   no-install validation from the checkout. This means validating through local
   build/test/check commands such as the Pre-PR checks and direct
   `cargo run --locked -p flavor-cli -- ...` invocations, without installing a
   new released `flavor`.
2. Push the branch, publish a beta release from that branch, update the local
   installed CLI to the beta, and comment on the issue with the branch, beta
   version or run, local verification result, and what the reporter should
   retry. Use the GitHub text payload hygiene rules for the issue comment.
3. After updating the local CLI and posting the issue comment, stop and wait for
   explicit human direction. The user owns the cadence and will tell the agent
   when to continue; do not poll, watch, or loop-listen for issue activity.
   When directed to continue, repeat the branch change, no-install validation,
   push, beta release, local CLI update, and issue-comment cycle until the
   issue reporter actively closes the issue. Do not close the issue on the
   reporter's behalf, and do not use a PR body `Closes #N` to bypass this
   verification loop.
4. After the reporter closes the issue, use the repo-local `runseal :pr` flow
   to merge the already-verified branch into `main`, publish a stable release,
   and update the local installed CLI to the stable version.

Keep beta/stable publication and local CLI update as required parts of this hot
path. A green branch check without installing the published artifact is not
enough to finish reporter-driven issue follow-up.

### GitHub Tool Auth

For `runseal @tool github ...` writes, prefer runseal profile env injection over
ad hoc shell substitution. Generate a dedicated GitHub token for repo operator
writes, keep it in repo-local private state such as `.local/secrets/`, expose it
through the runseal profile as an environment variable, and call the tool with
`--token-env <name>` plus `--prefix-enable=true`.

Do not commit token material, do not paste tokens into command lines, and do not
fall back to `GITHUB_TOKEN="$(gh auth token)" ...` unless the dedicated
profile-injected token is unavailable and the user explicitly accepts the
fallback for that one operation.

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
cargo run --locked -p flavor-cli -- check --root . --config flavor.toml
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
initialization entrypoint is `scripts/init.py`; additional local support
commands, if added, should use runseal wrappers plus `scripts/cli/`.

### Does Init Replace Existing Hooks?

Not by default. If a hook exists and was not generated by `scripts/init.py`, the
script stops and asks for `--force`. With `--force`, it backs up the existing
hook to a numbered `.bak` path before replacing it. Older bootstrap-generated
hooks are treated as generated and are replaced idempotently.
