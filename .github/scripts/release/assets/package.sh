#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/../../../.." && pwd)
APP_DIR="$ROOT/app"
NAME=flavor
VERSION=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$APP_DIR/Cargo.toml" | head -n 1)
RELEASE_VERSION=${1:-${RELEASE_VERSION:-v$VERSION}}
TARGET=${TARGET:-$(rustc -Vv | sed -n 's/^host: //p')}
DIST_DIR=${DIST_DIR:-"$ROOT/dist"}
ARTIFACT_DIR="$DIST_DIR/$RELEASE_VERSION"

mkdir -p "$ARTIFACT_DIR"

if [ -n "${TARGET:-}" ]; then
  cargo build --release --locked --manifest-path "$APP_DIR/Cargo.toml" --target "$TARGET"
  BIN="$APP_DIR/target/$TARGET/release/$NAME"
else
  cargo build --release --locked --manifest-path "$APP_DIR/Cargo.toml"
  BIN="$APP_DIR/target/release/$NAME"
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM

cp "$BIN" "$tmpdir/$NAME"
chmod +x "$tmpdir/$NAME"

archive="$NAME-$TARGET.tar.gz"
tar -C "$tmpdir" -czf "$ARTIFACT_DIR/$archive" "$NAME"

printf '%s\n' "$ARTIFACT_DIR/$archive"
