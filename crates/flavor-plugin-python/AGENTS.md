# AGENTS

`flavor-plugin-python` owns Python syntax facts on top of `flavor-core` and
the Python grammar contract under `grammars/python`.

## Directory Rules

- `src/lib.rs`, `src/model.rs`, and `src/plugin.rs` own the analyzer facade,
  output model, and first-party plugin delivery.
- `src/internal/grammar.rs` owns plugin-local string kind constants and schema
  loading from `grammars/python`; it is not a public API.
- Keep parser behavior aligned with `grammars/python/*.g4` and
  `grammars/python/metadata.json`.
- Do not add CLI rule/report behavior or config discovery here.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-python
```
