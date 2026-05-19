# AGENTS

`crates/flavor-plugin-filesystem/` owns first-party filesystem path, source
file, and source directory shape plugin identity and behavior.

## Directory Rules

- `src/lib.rs` owns the plugin id and rule ids used by the bundled host.
- Keep this crate independent from CLI config discovery and report rendering.
- Rule semantics stay about path, source file budget, source directory depth,
  and direct-child shape, not product-specific business concepts.

## Common Commands

```bash
cargo test --locked -p flavor-plugin-filesystem
```
