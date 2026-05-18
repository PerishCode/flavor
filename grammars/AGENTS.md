# AGENTS

`grammars/` owns the repo-visible `.g4` source of truth and `flavor.g4.json`
sidecars for plugin-facing grammar contracts.

## Directory Rules

- Keep one grammar bundle per language or embedded language family.
- Use `.g4` for grammar source and `flavor.g4.json` for flavor-specific
  metadata.
- Do not place plugin DSL, rule DSL, or consumer config here.
- Keep parser backend details out of the grammar source of truth.
- When implementation behavior changes, update the relevant grammar and harness
  in the same change.

## Common Commands

```bash
cargo test --locked -p flavor-g4
```

## Standard Workflow

- Define tokens, nodes, productions, facts, diagnostics, span mapping, and
  recovery before expanding parser adapters or rule-facing facts.
- Harnesses should prove conformance to this contract rather than merely
  preserving ad hoc parser output.
