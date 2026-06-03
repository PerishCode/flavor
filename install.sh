#!/usr/bin/env sh
set -eu

COMMAND=${1:-install}
[ $# -gt 0 ] && shift || true

CHANNEL=${FLAVOR_CHANNEL:-stable}
VERSION=${FLAVOR_VERSION:-}
PUBLIC_URL=${FLAVOR_RELEASES_PUBLIC_URL:-https://releases.flavor.perish.uk}
INSTALL_ROOT=${FLAVOR_INSTALL_ROOT:-"$HOME/.local/share/flavor"}
LOCAL_BIN_DIR=${FLAVOR_LOCAL_BIN_DIR:-"$HOME/.local/bin"}

while [ $# -gt 0 ]; do
  case "$1" in
    --channel)
      CHANNEL=${2:-}
      [ -n "$CHANNEL" ] || { echo "--channel requires a value" >&2; exit 1; }
      shift 2
      ;;
    --channel=*)
      CHANNEL=${1#--channel=}
      shift
      ;;
    --version)
      VERSION=${2:-}
      [ -n "$VERSION" ] || { echo "--version requires a value" >&2; exit 1; }
      shift 2
      ;;
    --version=*)
      VERSION=${1#--version=}
      shift
      ;;
    --public-url)
      PUBLIC_URL=${2:-}
      [ -n "$PUBLIC_URL" ] || { echo "--public-url requires a value" >&2; exit 1; }
      shift 2
      ;;
    --public-url=*)
      PUBLIC_URL=${1#--public-url=}
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
flavor installer

Usage:
  install.sh
  install.sh install [--channel stable|beta] [--version vX.Y.Z] [--public-url <url>]
  install.sh upgrade [--channel stable|beta] [--version vX.Y.Z] [--public-url <url>]
  install.sh uninstall

Environment:
  FLAVOR_RELEASES_PUBLIC_URL  # default: https://releases.flavor.perish.uk
  FLAVOR_CHANNEL
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

need_public_url() {
  PUBLIC_URL=${PUBLIC_URL%/}
}

platform_archive() {
  os=$(uname -s)
  arch=$(uname -m)
  case "$os:$arch" in
    Linux:x86_64|Linux:amd64) echo "flavor-x86_64-unknown-linux-gnu.tar.gz" ;;
    Darwin:arm64|Darwin:aarch64) echo "flavor-aarch64-apple-darwin.tar.gz" ;;
    Darwin:x86_64|Darwin:amd64) echo "flavor-x86_64-apple-darwin.tar.gz" ;;
    *) echo "unsupported platform: $os $arch" >&2; exit 1 ;;
  esac
}

latest_version() {
  metadata="$1"
  sed -n 's/.*"releaseVersion"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$metadata" | head -n 1
}

install_flavor() {
  need_public_url
  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT INT TERM

  if [ -z "$VERSION" ]; then
    curl -fsSL "$PUBLIC_URL/$CHANNEL/latest/metadata.json" -o "$tmpdir/metadata.json"
    VERSION=$(latest_version "$tmpdir/metadata.json")
    [ -n "$VERSION" ] || { echo "failed to resolve latest flavor version" >&2; exit 1; }
  fi

  archive=$(platform_archive)
  archive_url="$PUBLIC_URL/$CHANNEL/versions/$VERSION/$archive"
  mkdir -p "$INSTALL_ROOT/$VERSION" "$LOCAL_BIN_DIR"
  curl -fsSL "$archive_url" -o "$tmpdir/$archive"
  tar -xzf "$tmpdir/$archive" -C "$INSTALL_ROOT/$VERSION"
  chmod +x "$INSTALL_ROOT/$VERSION/flavor"

  link="$LOCAL_BIN_DIR/flavor"
  rm -f "$link"
  ln -s "$INSTALL_ROOT/$VERSION/flavor" "$link"
  "$link" --version
  printf 'installed flavor to %s\n' "$link"
}

uninstall_flavor() {
  bin_path="$LOCAL_BIN_DIR/flavor"
  if [ -n "$VERSION" ]; then
    normalized_version="v$(printf '%s' "$VERSION" | sed 's/^v//')"
    target="$INSTALL_ROOT/$VERSION/flavor"
    normalized_target="$INSTALL_ROOT/$normalized_version/flavor"
    if [ -L "$bin_path" ]; then
      link_target=$(readlink "$bin_path" || true)
      if [ "$link_target" = "$target" ] || [ "$link_target" = "$normalized_target" ]; then
        rm -f "$bin_path"
        printf 'removed %s\n' "$bin_path"
      fi
    fi
    rm -rf "$INSTALL_ROOT/$VERSION"
    if [ "$normalized_version" != "$VERSION" ]; then
      rm -rf "$INSTALL_ROOT/$normalized_version"
    fi
    rmdir "$INSTALL_ROOT" 2>/dev/null || true
    printf 'removed flavor %s from %s\n' "$VERSION" "$INSTALL_ROOT"
    return
  fi

  rm -f "$bin_path"
  rm -rf "$INSTALL_ROOT"
  rmdir "$LOCAL_BIN_DIR" 2>/dev/null || true
  printf 'removed flavor from %s and %s\n' "$INSTALL_ROOT" "$bin_path"
}

case "$COMMAND" in
  install|upgrade) install_flavor ;;
  uninstall) uninstall_flavor ;;
  *)
    echo "unknown command: $COMMAND" >&2
    exit 1
    ;;
esac
