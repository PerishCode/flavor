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
flavor check
flavor check --root . --config flavor.json
flavor check --format json
flavor check --strict-warnings
```

Example config:

```json
{
  "scan": {
    "include": ["src/**", "tests/**"],
    "exclude": ["target/**", "node_modules/**"]
  },
  "overrides": [
    {
      "match": "src/generated/**",
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

Rules use namespaced ids such as `core/fs/too-many-children`, `core/source/too-long`, `core/naming/too-many-words`, `core/dispatch/branch-too-long`, `rust/tests/in-source`, and `vue/parse/error`.

Run `flavor help` for the product boundary and issue URL. Report parser gaps, rule noise, and install issues at <https://github.com/PerishCode/flavor/issues>.

## Workspace

The installable binary lives in `crates/flavor-cli`. Compiler frontends grow as sibling crates: `flavor-compiler-core` owns compiler substrate primitives, `flavor-compiler-ts` owns TS/JS syntax facts, `flavor-compiler-vue` owns Vue SFC/template/style facts, and `flavor-compiler-svelte` owns Svelte descriptor/markup facts. Compiler crates accept typed config through state and do not define config file names.

## Scope

`flavor` does not format, rewrite, run services, manage repositories, or inspect product semantics.
