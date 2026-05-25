# flavor v0.2.0

## Why

`flavor` 0.2 resets the CLI configuration contract around explicit scan boundaries, strict current-format config parsing, and versioned test-source metadata.

## What

- Treat `--root` as the scan boundary while keeping config patterns relative to the config file directory.
- Discover default config files in `flavor.toml`, `flavor.yaml`, then `flavor.json` order.
- Support `FLAVOR_CONFIG` between `--config` and automatic discovery.
- Parse JSON, YAML, and TOML configs with strict `snake_case` field names.
- Add root `metadata.json` with `tests.hash` and `tests.scopes`.
- Add a guard script that validates the test-source hash and changelog requirements.
- Store `tests.hash` and `tests.scopes` in R2 release metadata.
- Limit test-hash Y-version baseline enforcement to release workflows using R2 stable latest metadata as the baseline: beta emits a warning, stable fails.

## Tests

- `cargo fmt --all --check` -> clean
- `cargo clippy --locked --workspace --all-targets -- -D warnings` -> clean
- `cargo test --locked --workspace` -> passed
- `cargo run --locked -p flavor-cli -- check --root . --config flavor.toml` -> passed
- `.github/scripts/guard/version.ps1` -> passed
