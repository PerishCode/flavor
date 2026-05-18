# AGENTS

`grammars/` owns the repo-visible `.g4` source of truth and `metadata.json`
files for plugin-facing grammar contracts.

## Directory Rules

- Keep one grammar bundle per language or embedded language family.
- Use `.g4` for grammar source and `metadata.json` for flavor-specific
  contract metadata.
- Do not place plugin DSL, rule DSL, or consumer config here.
- Keep parser backend details out of the grammar source of truth.
- When implementation behavior changes, update the relevant grammar and harness
  in the same change.

## Common Commands

```bash
cargo test --locked -p flavor-grammar
python3 scripts/dev/antlr.py check
```

## Standard Workflow

- Define or update grammar shape in `.g4`; keep metadata focused on owner,
  entrypoint, facts, diagnostics, span mapping, recovery, and backend bindings.
- Use `scripts/dev/antlr.py check` as the Dockerized ANTLR helper. It accepts
  optional `.g4` files under `grammars/`, groups them by path relative to
  `grammars/`, and runs ANTLR in dependency mode without Java artifact output.
- Harnesses should prove conformance to this contract rather than merely
  preserving ad hoc parser output.
