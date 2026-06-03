#!/usr/bin/env sh
set -eu

VERSION=${FLAVOR_VERSION:-}
INSTALL_ROOT=${FLAVOR_INSTALL_ROOT:-"$HOME/.local/share/flavor"}
LOCAL_BIN_DIR=${FLAVOR_LOCAL_BIN_DIR:-"$HOME/.local/bin"}

while [ $# -gt 0 ]; do
  case "$1" in
    --version)
      VERSION=${2:-}
      [ -n "$VERSION" ] || { echo "--version requires a value" >&2; exit 1; }
      shift 2
      ;;
    --version=*)
      VERSION=${1#--version=}
      shift
      ;;
    --install-root)
      INSTALL_ROOT=${2:-}
      [ -n "$INSTALL_ROOT" ] || { echo "--install-root requires a value" >&2; exit 1; }
      shift 2
      ;;
    --install-root=*)
      INSTALL_ROOT=${1#--install-root=}
      shift
      ;;
    --bin-dir)
      LOCAL_BIN_DIR=${2:-}
      [ -n "$LOCAL_BIN_DIR" ] || { echo "--bin-dir requires a value" >&2; exit 1; }
      shift 2
      ;;
    --bin-dir=*)
      LOCAL_BIN_DIR=${1#--bin-dir=}
      shift
      ;;
    -h|--help|help)
      cat <<'EOF'
flavor uninstaller

Usage:
  uninstall.sh
  uninstall.sh --version vX.Y.Z

Environment:
  FLAVOR_VERSION
  FLAVOR_INSTALL_ROOT
  FLAVOR_LOCAL_BIN_DIR
EOF
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

bin_path="$LOCAL_BIN_DIR/flavor"

remove_bin_if_current_version() {
  version="$1"
  target="$INSTALL_ROOT/$version/flavor"
  if [ -L "$bin_path" ]; then
    link_target=$(readlink "$bin_path" || true)
    if [ "$link_target" = "$target" ]; then
      rm -f "$bin_path"
      printf 'removed %s\n' "$bin_path"
    fi
  fi
}

remove_empty_dir() {
  dir="$1"
  if [ -d "$dir" ]; then
    rmdir "$dir" 2>/dev/null || true
  fi
}

if [ -n "$VERSION" ]; then
  normalized_version="v$(printf '%s' "$VERSION" | sed 's/^v//')"
  remove_bin_if_current_version "$VERSION"
  if [ "$normalized_version" != "$VERSION" ]; then
    remove_bin_if_current_version "$normalized_version"
  fi
  rm -rf "$INSTALL_ROOT/$VERSION"
  if [ "$normalized_version" != "$VERSION" ]; then
    rm -rf "$INSTALL_ROOT/$normalized_version"
  fi
  remove_empty_dir "$INSTALL_ROOT"
  printf 'removed flavor %s from %s\n' "$VERSION" "$INSTALL_ROOT"
  exit 0
fi

rm -f "$bin_path"
rm -rf "$INSTALL_ROOT"
remove_empty_dir "$LOCAL_BIN_DIR"
printf 'removed flavor from %s and %s\n' "$INSTALL_ROOT" "$bin_path"
