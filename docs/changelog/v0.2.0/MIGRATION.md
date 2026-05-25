# Migrating to flavor v0.2.0

## Config Files

Automatic discovery now checks only these default names, in order:

```text
flavor.toml
flavor.yaml
flavor.json
```

Use `--config` or `FLAVOR_CONFIG` for any non-default path.

## Config Fields

Config fields are strict `snake_case`. For example:

```toml
allow_empty_scan = true

[scan]
include = ["src/**", "tests/**"]
exclude = ["target/**"]
```

## Scan Root

`--root` is now the scan boundary. Config patterns remain relative to the config file directory.

## Defaults

If a config omits `scan.include`, flavor uses built-in source defaults for common project layouts such as `src/**`, `tests/**`, `crates/*/src/**`, `packages/*/src/**`, and related source roots.
