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
    & (Join-Path $root 'install.ps1') install --channel $channel --version $version
    & (Join-Path $env:FLAVOR_LOCAL_BIN_DIR 'flavor.exe') --version
    & (Join-Path $env:FLAVOR_LOCAL_BIN_DIR 'flavor.exe') check --root $root --config (Join-Path $root 'flavor.json')

    if ($env:SMOKE_LATEST -eq '1') {
        Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $env:FLAVOR_LOCAL_BIN_DIR 'flavor.exe')
        $env:FLAVOR_INSTALL_ROOT = Join-Path $tmpdir 'latest-smoke'
        & (Join-Path $root 'install.ps1') install --channel $channel
        & (Join-Path $env:FLAVOR_LOCAL_BIN_DIR 'flavor.exe') --version
        & (Join-Path $env:FLAVOR_LOCAL_BIN_DIR 'flavor.exe') check --root $root --config (Join-Path $root 'flavor.json')
    }
}
finally {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmpdir
}
