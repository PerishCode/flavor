#!/usr/bin/env sh
set -eu

COMMAND=${1:-install}
[ $# -gt 0 ] && shift || true

CHANNEL=${FLAVOR_CHANNEL:-stable}
VERSION=${FLAVOR_VERSION:-}
PUBLIC_URL=${FLAVOR_RELEASES_PUBLIC_URL:-}
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
  install.sh install [--channel stable|beta] [--version vX.Y.Z] [--public-url <url>]
  install.sh upgrade [--channel stable|beta] [--version vX.Y.Z] [--public-url <url>]
  install.sh uninstall

Environment:
  FLAVOR_RELEASES_PUBLIC_URL
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
  [ -n "$PUBLIC_URL" ] || {
    echo "FLAVOR_RELEASES_PUBLIC_URL or --public-url is required" >&2
    exit 1
  }
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
  rm -f "$LOCAL_BIN_DIR/flavor"
  printf 'removed %s\n' "$LOCAL_BIN_DIR/flavor"
}

case "$COMMAND" in
  install|upgrade) install_flavor ;;
  uninstall) uninstall_flavor ;;
  *)
    echo "unknown command: $COMMAND" >&2
    exit 1
    ;;
esac
