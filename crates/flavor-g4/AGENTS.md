# AGENTS

`crates/flavor-g4/` owns parsing and validation of `.g4` grammar bundle
sidecars.

## Directory Rules

- `src/lib.rs` owns the G4 sidecar contract parser.
- `tests/` validates every `flavor.g4.json` sidecar and referenced `.g4` file
  under the repository `grammars/` directory.

Do not add language parser implementation or lint rule execution here.

## Common Commands

```bash
cargo test --locked -p flavor-g4
```

## Standard Workflow

- Keep the grammar sidecar format strict and boring.
- Add validation before relying on new grammar sections from implementation
  crates.
- This crate should remain independent from CLI config and report behavior.
