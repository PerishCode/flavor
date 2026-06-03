#!/usr/bin/env bash
set -euo pipefail

for name in GITHUB_STEP_SUMMARY RELEASE_CHANNEL RELEASE_VERSION R2_METADATA_URL R2_VERSION_METADATA_URL R2_VERSION_PREFIX FLAVOR_RELEASES_PUBLIC_URL; do
  if [ -z "${!name:-}" ]; then
    echo "$name is required" >&2
    exit 1
  fi
done

{
  echo "## Flavor ${RELEASE_CHANNEL} release"
  echo ""
  echo "| Field | Value |"
  echo "| --- | --- |"
  echo "| Channel | \`${RELEASE_CHANNEL}\` |"
  echo "| Version | \`${RELEASE_VERSION}\` |"
  if [ -n "${BASE_VERSION:-}" ]; then
    echo "| Base version | \`${BASE_VERSION}\` |"
  fi
  if [ -n "${BETA_NUMBER:-}" ]; then
    echo "| Beta number | \`${BETA_NUMBER}\` |"
  fi
  if [ -n "${STATE_SOURCE:-}" ]; then
    echo "| State source | \`${STATE_SOURCE}\` |"
  fi
  echo "| R2 prefix | \`${R2_VERSION_PREFIX}\` |"
  echo ""
  echo "### Links"
  echo ""
  if [ "$RELEASE_CHANNEL" = "stable" ]; then
    echo "- Stable unix installer: ${FLAVOR_RELEASES_PUBLIC_URL%/}/install.sh"
    echo "- Stable windows installer: ${FLAVOR_RELEASES_PUBLIC_URL%/}/install.ps1"
    echo "- Stable unix uninstaller: ${FLAVOR_RELEASES_PUBLIC_URL%/}/uninstall.sh"
    echo "- Stable windows uninstaller: ${FLAVOR_RELEASES_PUBLIC_URL%/}/uninstall.ps1"
  fi
  echo "- Latest metadata: ${R2_METADATA_URL}"
  echo "- Version metadata: ${R2_VERSION_METADATA_URL}"
} >> "$GITHUB_STEP_SUMMARY"
