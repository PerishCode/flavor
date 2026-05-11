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
flavor check                       # auto-discovers flavor.json at --root
flavor check --config flavor.json  # explicit path
flavor check --format json
flavor check --strict-warnings
flavor rules                       # browse the built-in rule catalog
flavor rules --format json
```

`flavor check` exits `1` on any deny issue, on a `--strict-warnings` failure, or when `scan.include` matched zero files. The empty-scan case prints a stderr warning so CI never silently confuses a misconfigured include pattern with a clean repo.

Run `flavor help` for the product boundary and `flavor rules` for the full rule catalog. Report parser gaps, rule noise, and install issues at <https://github.com/PerishCode/flavor/issues>.

## Config

A `flavor.json` has two top-level keys: `scan` (required) and `overrides` (optional).

```json
{
  "scan": {
    "include": ["src/**", "tests/**"],
    "exclude": ["target/**", "node_modules/**"]
  },
  "overrides": [
    {
      "match": ["src/generated/**", "vendor/**"],
      "kind": "file",
      "priority": 10,
      "rules": {
        "core/naming/too-many-words": {
          "enabled": false,
          "reason": "generated names mirror protocol contracts"
        },
        "core/source/too-long": {
          "payload": { "max_lines": 800 },
          "severity": "warning"
        }
      }
    }
  ]
}
```

### `scan`

| field     | type       | required | meaning                                                                                           |
|-----------|------------|----------|---------------------------------------------------------------------------------------------------|
| `include` | `string[]` | yes      | Glob patterns, relative to `--root`, that scope which files the check covers.                     |
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
2. `<root>/flavor.json` if it exists. flavor prints `flavor: using config <path>` on stderr so a stray file at the scan root never silently changes the check.
3. Built-in defaults. In user repos these match nothing; the empty-scan warning will flag that.

## Workspace

The installable binary lives in `crates/flavor-cli`. Compiler frontends grow as sibling crates: `flavor-compiler-core` owns compiler substrate primitives, `flavor-compiler-ts` owns TS/JS syntax facts, `flavor-compiler-vue` owns Vue SFC/template/style facts, and `flavor-compiler-svelte` owns Svelte descriptor/markup facts. Compiler crates accept typed config through state and do not define config file names.

## Scope

`flavor` does not format, rewrite, run services, manage repositories, or inspect product semantics.
