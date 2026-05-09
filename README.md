# flavor

Personal check-only code flavor lint CLI.

`flavor` turns abstract code-shape preferences into executable checks over Rust, TypeScript, TSX, and Vue script source. Reports include bad-flavor notes and action hints. They are review pressure, not automatic repair recipes.

## Install

Unix:

```bash
curl -fsSL "$FLAVOR_RELEASES_PUBLIC_URL/stable/latest/install.sh" | sh -s -- install
```

Windows PowerShell:

```powershell
& ([scriptblock]::Create((irm "$env:FLAVOR_RELEASES_PUBLIC_URL/stable/latest/install.ps1"))) install
```

Pin a version:

```bash
curl -fsSL "$FLAVOR_RELEASES_PUBLIC_URL/stable/latest/install.sh" | sh -s -- install --version v0.1.0
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

Rules use namespaced ids such as `core/fs/too-many-children`, `core/source/too-long`, `core/naming/too-many-words`, and `rust/tests/in-source`.

## Scope

`flavor` does not format, rewrite, run services, manage repositories, or inspect product semantics.
