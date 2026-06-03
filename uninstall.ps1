$ErrorActionPreference = 'Stop'

$version = if ($env:FLAVOR_VERSION) { $env:FLAVOR_VERSION } else { '' }
$installRoot = if ($env:FLAVOR_INSTALL_ROOT) { $env:FLAVOR_INSTALL_ROOT } else { Join-Path $env:LOCALAPPDATA 'flavor' }
$localBinDir = if ($env:FLAVOR_LOCAL_BIN_DIR) { $env:FLAVOR_LOCAL_BIN_DIR } else { Join-Path $env:USERPROFILE '.local\bin' }

for ($i = 0; $i -lt $args.Length; $i++) {
    $arg = $args[$i]
    switch -Regex ($arg) {
        '^--version$' { $i++; $version = $args[$i]; continue }
        '^--version=(.+)$' { $version = $Matches[1]; continue }
        '^--install-root$' { $i++; $installRoot = $args[$i]; continue }
        '^--install-root=(.+)$' { $installRoot = $Matches[1]; continue }
        '^--bin-dir$' { $i++; $localBinDir = $args[$i]; continue }
        '^--bin-dir=(.+)$' { $localBinDir = $Matches[1]; continue }
        '^(-h|--help|help)$' {
            @'
flavor uninstaller

Usage:
  uninstall.ps1
  uninstall.ps1 --version vX.Y.Z

Environment:
  FLAVOR_VERSION
  FLAVOR_INSTALL_ROOT
  FLAVOR_LOCAL_BIN_DIR
'@ | Write-Output
            exit 0
        }
        default { throw "unknown argument: $arg" }
    }
}

$binPath = Join-Path $localBinDir 'flavor.exe'

function Remove-EmptyDir {
    param([string]$Path)
    if ([System.IO.Directory]::Exists($Path)) {
        try {
            Remove-Item -Force -ErrorAction Stop $Path
        }
        catch [System.IO.IOException] {}
    }
}

function Installed-Version {
    if (![System.IO.File]::Exists($binPath)) {
        return ''
    }
    try {
        $output = & $binPath --version
        if ($output -match 'v?([0-9]+\.[0-9]+\.[0-9]+(?:[-.][A-Za-z0-9]+)*)') {
            return "v$($Matches[1].TrimStart('v'))"
        }
    }
    catch {}
    return ''
}

if (![string]::IsNullOrWhiteSpace($version)) {
    $normalizedVersion = "v$($version.TrimStart('v'))"
    if ((Installed-Version) -eq $normalizedVersion) {
        Remove-Item -Force -ErrorAction SilentlyContinue $binPath
        Write-Output "removed $binPath"
    }
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $installRoot $version)
    if ($version -ne $normalizedVersion) {
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $installRoot $normalizedVersion)
    }
    Remove-EmptyDir $installRoot
    Write-Output "removed flavor $version from $installRoot"
    exit 0
}

Remove-Item -Force -ErrorAction SilentlyContinue $binPath
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $installRoot
Remove-EmptyDir $localBinDir
Write-Output "removed flavor from $installRoot and $binPath"
