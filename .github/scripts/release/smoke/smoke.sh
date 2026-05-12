#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/../../../.." && pwd)
VERSION=${1:-}
CHANNEL=${2:-stable}

[ -n "$VERSION" ] || { printf '%s\n' 'missing release version' >&2; exit 1; }
[ -n "${FLAVOR_RELEASES_PUBLIC_URL:-}" ] || { printf '%s\n' 'FLAVOR_RELEASES_PUBLIC_URL is required' >&2; exit 1; }

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM

export HOME="$tmpdir/home"
export FLAVOR_INSTALL_ROOT="$tmpdir/install"
export FLAVOR_LOCAL_BIN_DIR="$tmpdir/bin"
mkdir -p "$HOME" "$FLAVOR_INSTALL_ROOT" "$FLAVOR_LOCAL_BIN_DIR"

sh "$ROOT/install.sh" install --channel "$CHANNEL" --version "$VERSION"
"$FLAVOR_LOCAL_BIN_DIR/flavor" --version
"$FLAVOR_LOCAL_BIN_DIR/flavor" check --root "$ROOT" --config "$ROOT/flavor.json"

if [ "${SMOKE_LATEST:-}" = "1" ]; then
  rm -f "$FLAVOR_LOCAL_BIN_DIR/flavor"
  rm -rf "$FLAVOR_INSTALL_ROOT/latest-smoke"
  sh "$ROOT/install.sh" install --channel "$CHANNEL" --install-root "$FLAVOR_INSTALL_ROOT/latest-smoke"
  "$FLAVOR_LOCAL_BIN_DIR/flavor" --version
  "$FLAVOR_LOCAL_BIN_DIR/flavor" check --root "$ROOT" --config "$ROOT/flavor.json"
fi
