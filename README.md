# flavor

Personal check-only code flavor lint CLI.

`flavor` turns abstract code-shape preferences into executable checks over Python, Rust, TypeScript, TSX, Vue, and Svelte source. Reports include bad-flavor notes and action hints. They are review pressure, not automatic repair recipes.

## Install

Unix:

```bash
curl -fsSL https://flavor.perish.uk/manage.sh | sh
```

Windows PowerShell:

```powershell
irm https://flavor.perish.uk/manage.ps1 | pwsh
```

Pin a version:

```bash
curl -fsSL https://flavor.perish.uk/manage.sh | sh -s -- install --version v0.1.0
```

Install the latest beta:

```bash
curl -fsSL https://flavor.perish.uk/manage.sh | sh -s -- install --channel beta
```

Keep older installed versions instead of prompting or pruning:

```bash
curl -fsSL https://flavor.perish.uk/manage.sh | sh -s -- install --retain=true
```

Uninstall every installed version:

```bash
curl -fsSL https://flavor.perish.uk/manage.sh | sh -s -- uninstall
```

Windows PowerShell:

```powershell
& ([scriptblock]::Create((irm https://flavor.perish.uk/manage.ps1))) uninstall
```

Uninstall one version:

```bash
curl -fsSL https://flavor.perish.uk/manage.sh | sh -s -- uninstall --version v0.2.2
```

## Usage

```bash
flavor check                       # auto-discovers flavor.* at --root
flavor check --config flavor.toml  # explicit path
flavor check --format json
flavor check --strict-warnings
flavor rules                       # browse the built-in rule catalog
flavor rules --format json
```

`flavor check` exits `1` on any deny issue, on a `--strict-warnings` failure, or when `scan.include` matched zero files. The empty-scan case prints a stderr warning so CI never silently confuses a misconfigured include pattern with a clean repo.

Run `flavor help` for the product boundary and `flavor rules` for the full rule catalog. Report parser gaps, rule noise, and install issues at <https://github.com/PerishCode/flavor/issues>.

## Config

A `flavor.json`, `flavor.toml`, `flavor.yaml`, or `flavor.yml` has one required
top-level key, `scan`. Optional `preferences` expand named rule sets over
consumer paths, and `overrides` remain the final rule adjustment layer.

```json
{
  "scan": {
    "include": ["src/**", "tests/**"],
    "exclude": ["target/**", "node_modules/**"]
  },
  "preferences": [
    {
      "name": "frontend/renderer-boundary",
      "match": "apps/*/src",
      "primitiveSources": ["@acme/components"]
    }
  ],
  "overrides": [
    {
      "match": ["api/contracts/**", "vendor/**"],
      "kind": "file",
      "priority": 10,
      "rules": {
        "core/naming/too-many-words": {
          "enabled": false,
          "reason": "external names mirror protocol contracts"
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

### `preferences[*]`

Named preference sets expand to ordinary rule overrides before explicit
`overrides` are applied. This keeps built-in rules generic while allowing a
consumer config to choose an opinionated boundary shape.

| field                | type                   | required | meaning                                                                                                      |
|----------------------|------------------------|----------|--------------------------------------------------------------------------------------------------------------|
| `name`               | string                 | yes      | Preference set id. Currently `frontend/renderer-boundary`.                                                    |
| `match`              | `string` or `string[]` | yes      | Renderer `src` root glob(s), relative to `--root`. Empty arrays error at load time.                           |
| `priority`           | integer                | no       | Ordering among preference-generated rules. Explicit `overrides` still apply after preferences.                |
| `primitiveSources`   | `string[]`             | yes      | Package specifiers that count as shared primitives for renderer component composition.                        |
| `allowedIntrinsics`  | `string[]`             | no       | Lowercase JSX intrinsic names allowed despite the boundary, reserved for deliberate escape hatches.           |

`frontend/renderer-boundary` expands to atomic rules that require
`src/{lib,components,views,app.tsx,main.tsx}`, forbid local CSS extensions,
forbid raw intrinsic JSX in TSX, require `components/**/*.tsx` to compose a
configured primitive, require PascalCase component/view TSX files, and keep
`lib` file basenames to one word.

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
2. The first supported config under `<root>` if it exists. Format priority is `flavor.toml`, `flavor.yaml`, `flavor.yml`, then `flavor.json`. flavor prints `flavor: using config <path>` on stderr so a stray file at the scan root never silently changes the check.
3. Built-in defaults. In user repos these match nothing; the empty-scan warning will flag that.

## Workspace

The installable binary lives in `crates/flavor-cli`. First-party bundled plugins grow as sibling crates: `flavor-core` owns shared source text, span, diagnostic, product, and syntax tree primitives, `flavor-grammar` owns grammar contract metadata, `.g4` source indexes, raw AST schema bundles, runtime kind lookup, and backend adapter helpers, `flavor-plugin-g4` brings `.g4` files into the normal plugin/product pipeline, `flavor-plugin-filesystem` owns filesystem path/source shape plugin identity, and the language plugin crates own Python, Rust, TS/JS/TSX, Vue, and Svelte syntax facts. Plugin crates do not define config file names or report rendering.

Frontend contracts are anchored in `grammars/<bundle>/*.g4` plus
`grammars/<bundle>/metadata.json`. The `.g4` files are the repo-visible
grammar source of truth and drive the runtime raw AST schema symbols. Metadata
holds flavor-specific entrypoints, fact payload contracts, raw AST node
bindings, diagnostics, spans, and recovery expectations.
Runtime parser backends remain staged adapters for this refactor: Rust still
uses tree-sitter, TypeScript keeps its plugin lexer/parser, and Vue/Svelte keep
their descriptor/template/markup parsers. Those adapters emit rowan CSTs
through `flavor-grammar` `GrammarSpec`/`GrammarBundle` lookup and string kind
shapes, so the raw AST output shape is governed by `.g4` schemas even while
parser execution stays unchanged.

## Development

```bash
python3 scripts/init.py                       # initialize hooks and verify local prerequisites
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p flavor-cli -- check --root . --config flavor.toml
python3 scripts/dev/antlr.py check         # optional Dockerized G4 validation
```

Repo-local operator commands use runseal wrappers rather than the installable
`flavor` binary:

```bash
runseal :cloudflare <command> [args]
runseal :pr [options]
runseal :release --channel=stable|beta [options]
runseal @wrappers
```

`python3 scripts/init.py` checks that `runseal` is installed before installing
hooks.

## Scope

`flavor` does not format, rewrite, run services, manage repositories, or inspect product semantics.

## Contributing

Open issues at <https://github.com/PerishCode/flavor/issues> for parser gaps, rule noise, or install problems.

For source-change shape — branch names, commit/PR conventions, where to put tests, how agents land their own PRs — see the **Standard Workflow** section in [AGENTS.md](./AGENTS.md#standard-workflow).
