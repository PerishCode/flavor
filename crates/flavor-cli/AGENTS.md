# AGENTS

`crates/flavor-cli/` owns the installable `flavor` binary, rule execution,
reports, and CLI-facing scan configuration.

## Directory Rules

- `src/main.rs` wires command dispatch, config resolution, scan execution,
  report printing, and exit codes.
- `src/cli.rs` owns command parsing and help text. Keep CLI behavior stable and
  update tests when output or accepted arguments move.
- `src/config.rs`, `src/scan.rs`, and `src/path_match.rs` own config discovery,
  scan traversal, and include/exclude matching. Plugin crates do not own these
  concerns.
- `src/plugins/` owns the first-party bundled plugin boundary for filesystem,
  source-structure, language, naming, and framework checks. Keep it internal to
  the CLI until the external plugin model exists.
- `src/rules.rs`, `src/model.rs`, `src/output.rs`, and `src/naming/` own rule
  definitions, issue/report modeling, text/JSON output, and naming helpers used
  by bundled plugins.
- `src/rust_tests.rs` owns Rust test-shape inspection.
- `tests/unit/` contains CLI unit coverage. Register each new unit test module
  in `tests/unit.rs`.

Keep this crate check-only. Do not add formatting, rewriting, service execution,
repository orchestration, or runtime management here.

## Common Commands

```bash
cargo test --locked -p flavor-cli --test unit
cargo run --locked -p flavor-cli -- rules
cargo run --locked -p flavor-cli -- check --root . --config flavor.json
```

## Standard Workflow

- For a new or changed rule, update the rule definition, report model/output if
  needed, and focused unit tests.
- Reports should explain the bad flavor and suggest a direction of thought.
  Hints are review pressure, not automatic fix instructions.
- Config and exit-code changes are compatibility-sensitive. Update help text,
  tests, and the PR `## Compatibility` section when behavior moves.
- Writable fixtures should use the repository pattern:
  `std::env::temp_dir().join(format!("flavor-<slug>-{pid}-{seq}"))`, then clean
  up with `fs::remove_dir_all`.

## FAQ

### Can A Built-In Rule Encode Product Semantics?

No. Built-in rules stay about syntax, file shape, path shape, and abstract style
attributes. Product-specific scope belongs in consumer config.

### Where Should Scan Discovery Logic Live?

Here, in `flavor-cli`. Plugin crates should receive typed config/state and
return syntax/facts/diagnostics without knowing config filenames or discovery
rules.
