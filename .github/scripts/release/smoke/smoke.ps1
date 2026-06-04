$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))))
$version = if ($args.Length -gt 0) { $args[0] } else { throw 'missing release version' }
$channel = if ($args.Length -gt 1) { $args[1] } else { 'stable' }

$tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("flavor-smoke-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmpdir | Out-Null

try {
    $env:FLAVOR_INSTALL_ROOT = Join-Path $tmpdir 'install'
    $env:FLAVOR_LOCAL_BIN_DIR = Join-Path $tmpdir 'bin'
    New-Item -ItemType Directory -Force -Path $env:FLAVOR_INSTALL_ROOT, $env:FLAVOR_LOCAL_BIN_DIR | Out-Null
    & (Join-Path $root 'manage.ps1') install --channel $channel --version $version --retain=false
    & (Join-Path $env:FLAVOR_LOCAL_BIN_DIR 'flavor.exe') --version
    & (Join-Path $env:FLAVOR_LOCAL_BIN_DIR 'flavor.exe') check --root $root --config (Join-Path $root 'flavor.json')
    & (Join-Path $root 'manage.ps1') uninstall --version $version
    if (Test-Path (Join-Path $env:FLAVOR_INSTALL_ROOT $version)) {
        throw "version uninstall left $(Join-Path $env:FLAVOR_INSTALL_ROOT $version)"
    }

    if ($env:SMOKE_LATEST -eq '1') {
        Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $env:FLAVOR_LOCAL_BIN_DIR 'flavor.exe')
        $env:FLAVOR_INSTALL_ROOT = Join-Path $tmpdir 'latest-smoke'
        & (Join-Path $root 'manage.ps1') install --channel $channel --retain=false
        & (Join-Path $env:FLAVOR_LOCAL_BIN_DIR 'flavor.exe') --version
        & (Join-Path $env:FLAVOR_LOCAL_BIN_DIR 'flavor.exe') check --root $root --config (Join-Path $root 'flavor.json')
        & (Join-Path $root 'manage.ps1') uninstall --install-root $env:FLAVOR_INSTALL_ROOT
        if (Test-Path $env:FLAVOR_INSTALL_ROOT) {
            throw "full uninstall left $env:FLAVOR_INSTALL_ROOT"
        }
    }
}
finally {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmpdir
}
