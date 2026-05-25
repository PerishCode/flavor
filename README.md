# flavor

Personal check-only code flavor lint CLI.

`flavor` turns abstract code-shape preferences into executable checks over Rust, TypeScript, TSX, Vue, and Svelte source. Reports include bad-flavor notes and action hints. They are review pressure, not automatic repair recipes.

## Install

Unix:

```bash
curl -fsSL https://releases.flavor.perish.uk/stable/latest/install.sh \
  | sh -s -- install --public-url https://releases.flavor.perish.uk
```

Windows PowerShell:

```powershell
& ([scriptblock]::Create((irm "https://releases.flavor.perish.uk/stable/latest/install.ps1"))) install --public-url https://releases.flavor.perish.uk
```

Pin a version:

```bash
curl -fsSL https://releases.flavor.perish.uk/stable/versions/v0.1.0/install.sh \
  | sh -s -- install --version v0.1.0 --public-url https://releases.flavor.perish.uk
```

Install the latest beta:

```bash
curl -fsSL https://releases.flavor.perish.uk/beta/latest/install.sh \
  | sh -s -- install --channel beta --public-url https://releases.flavor.perish.uk
```

## Usage

```bash
flavor check                       # auto-discovers flavor.toml/yaml/json at --root
flavor check --config flavor.toml  # explicit path
flavor check --format json
flavor check --strict-warnings
flavor rules                       # browse the built-in rule catalog
flavor rules --format json
```

`flavor check` exits `1` on any deny issue, on a `--strict-warnings` failure, or when `scan.include` matched zero files. The empty-scan case prints a stderr warning so CI never silently confuses a misconfigured include pattern with a clean repo.

Run `flavor help` for the product boundary and `flavor rules` for the full rule catalog. Report parser gaps, rule noise, and install issues at <https://github.com/PerishCode/flavor/issues>.

## Config

A `flavor.toml`, `flavor.yaml`, or `flavor.json` config has two top-level keys: `scan` and `overrides`. When `scan.include` is omitted, flavor uses built-in source defaults.

```toml
[scan]
include = ["src/**", "tests/**"]
exclude = ["target/**", "node_modules/**"]

[[overrides]]
match = ["src/generated/**", "vendor/**"]
kind = "file"
priority = 10

[overrides.rules."core/naming/too-many-words"]
enabled = false
reason = "generated names mirror protocol contracts"

[overrides.rules."core/source/too-long"]
severity = "warning"
payload = { max_lines = 800 }
```

### `scan`

| field     | type       | required | meaning                                                                                           |
|-----------|------------|----------|---------------------------------------------------------------------------------------------------|
| `include` | `string[]` | no       | Glob patterns, relative to the config directory, that scope which files the check covers. Defaults to common source roots. |
| `exclude` | `string[]` | no       | Glob patterns subtracted from `include` (e.g. `**/target/**`, `**/node_modules/**`, `**/*.d.ts`). |

### `overrides[*]`

Ordered list of rule adjustments scoped to a path pattern.

| field      | type                          | required           | meaning                                                                                                       |
|------------|-------------------------------|--------------------|---------------------------------------------------------------------------------------------------------------|
| `match`    | `string` or `string[]`        | yes                | A glob, or an array of globs, identifying paths this override applies to. Empty arrays error at load time.    |
| `kind`     | `"any"` / `"file"` / `"dir"`  | no (default `any`) | Limit the override to files only, directories only, or both. Must be compatible with every rule under `rules`. |
| `priority` | integer                       | no (default `0`)   | Higher priorities apply after lower ones; ties fall back to declaration order.                                |
| `rules`    | object                        | yes                | Map of rule id → rule override (below).                                                                       |

### `overrides[*].rules[<ruleId>]`

| field      | type                       | required                            | meaning                                                                                      |
|------------|----------------------------|-------------------------------------|----------------------------------------------------------------------------------------------|
| `enabled`  | bool                       | no                                  | `false` silences the rule for matched paths. Requires a non-empty `reason`.                  |
| `severity` | `"deny"` / `"warning"`     | no                                  | Override the default severity (e.g. lower a `deny` to `warning` while a refactor lands).      |
| `reason`   | string                     | no (required when `enabled: false`) | Free-text justification, surfaced in `flavor check` output.                                  |
| `payload`  | object                     | no                                  | Rule-specific thresholds. See `flavor rules --format json` for the keys each rule consults.   |

Use `flavor rules` to browse rule ids, default severity, and payload keys without rerunning a check.

### Discovery

`flavor check` resolves the config in this order:

1. The `--config <path>` argument if provided. Missing or malformed → error.
2. `FLAVOR_CONFIG` if set. Missing or malformed → error.
3. Walk up from `--root` looking for `flavor.toml`, then `flavor.yaml`, then `flavor.json` in each directory. flavor prints `flavor: using config <path>` on stderr when discovery or `FLAVOR_CONFIG` chooses a config.
4. Built-in defaults rooted at `--root`.

`--root` is the scan boundary. Config patterns remain relative to the config file directory.

## Workspace

The installable binary lives in `crates/flavor-cli`. Compiler frontends grow as sibling crates: `flavor-compiler-core` owns compiler substrate primitives, `flavor-compiler-ts` owns TS/JS syntax facts, `flavor-compiler-vue` owns Vue SFC/template/style facts, and `flavor-compiler-svelte` owns Svelte descriptor/markup facts. Compiler crates accept typed config through state and do not define config file names.

## Scope

`flavor` does not format, rewrite, run services, manage repositories, or inspect product semantics.

## Contributing

Open issues at <https://github.com/PerishCode/flavor/issues> for parser gaps, rule noise, or install problems.

For source-change shape — branch names, commit/PR conventions, where to put tests, how agents land their own PRs — see the **Contribution Loop** section in [AGENTS.md](./AGENTS.md#contribution-loop).
