# AGENTS

`crates/flavor-plugin-source-structure/` owns first-party source file and source
directory shape plugin identity and behavior.

## Directory Rules

- `src/lib.rs` owns the plugin id and rule ids used by the bundled host.
- Keep this crate independent from CLI config discovery and report rendering.
- Rule semantics stay about source file and source tree shape.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-source-structure
```
