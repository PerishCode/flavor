#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/../../../.." && pwd)
VERSION=${1:-}
CHANNEL=${2:-stable}

[ -n "$VERSION" ] || { printf '%s\n' 'missing release version' >&2; exit 1; }

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM

export HOME="$tmpdir/home"
export FLAVOR_INSTALL_ROOT="$tmpdir/install"
export FLAVOR_LOCAL_BIN_DIR="$tmpdir/bin"
mkdir -p "$HOME" "$FLAVOR_INSTALL_ROOT" "$FLAVOR_LOCAL_BIN_DIR"

sh "$ROOT/manage.sh" install --channel "$CHANNEL" --version "$VERSION" --retain=false
"$FLAVOR_LOCAL_BIN_DIR/flavor" --version
"$FLAVOR_LOCAL_BIN_DIR/flavor" check --root "$ROOT" --config "$ROOT/flavor.json"
sh "$ROOT/manage.sh" uninstall --version "$VERSION"
[ ! -e "$FLAVOR_INSTALL_ROOT/$VERSION" ] || { printf '%s\n' "version uninstall left $FLAVOR_INSTALL_ROOT/$VERSION" >&2; exit 1; }

if [ "${SMOKE_LATEST:-}" = "1" ]; then
  rm -f "$FLAVOR_LOCAL_BIN_DIR/flavor"
  rm -rf "$FLAVOR_INSTALL_ROOT/latest-smoke"
  sh "$ROOT/manage.sh" install --channel "$CHANNEL" --install-root "$FLAVOR_INSTALL_ROOT/latest-smoke" --retain=false
  "$FLAVOR_LOCAL_BIN_DIR/flavor" --version
  "$FLAVOR_LOCAL_BIN_DIR/flavor" check --root "$ROOT" --config "$ROOT/flavor.json"
  sh "$ROOT/manage.sh" uninstall --install-root "$FLAVOR_INSTALL_ROOT/latest-smoke"
  [ ! -e "$FLAVOR_INSTALL_ROOT/latest-smoke" ] || { printf '%s\n' "full uninstall left $FLAVOR_INSTALL_ROOT/latest-smoke" >&2; exit 1; }
fi
