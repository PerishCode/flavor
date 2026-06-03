$ErrorActionPreference = 'Stop'

$command = if ($args.Length -gt 0) { $args[0] } else { 'install' }
$remaining = if ($args.Length -gt 1) { $args[1..($args.Length - 1)] } else { @() }

$channel = if ($env:FLAVOR_CHANNEL) { $env:FLAVOR_CHANNEL } else { 'stable' }
$version = if ($env:FLAVOR_VERSION) { $env:FLAVOR_VERSION } else { '' }
$publicUrl = if ($env:FLAVOR_RELEASES_PUBLIC_URL) { $env:FLAVOR_RELEASES_PUBLIC_URL } else { 'https://releases.flavor.perish.uk' }
$installRoot = if ($env:FLAVOR_INSTALL_ROOT) { $env:FLAVOR_INSTALL_ROOT } else { Join-Path $env:LOCALAPPDATA 'flavor' }
$localBinDir = if ($env:FLAVOR_LOCAL_BIN_DIR) { $env:FLAVOR_LOCAL_BIN_DIR } else { Join-Path $env:USERPROFILE '.local\bin' }

for ($i = 0; $i -lt $remaining.Length; $i++) {
    $arg = $remaining[$i]
    switch -Regex ($arg) {
        '^--channel$' { $i++; $channel = $remaining[$i]; continue }
        '^--channel=(.+)$' { $channel = $Matches[1]; continue }
        '^--version$' { $i++; $version = $remaining[$i]; continue }
        '^--version=(.+)$' { $version = $Matches[1]; continue }
        '^--public-url$' { $i++; $publicUrl = $remaining[$i]; continue }
        '^--public-url=(.+)$' { $publicUrl = $Matches[1]; continue }
        '^--install-root$' { $i++; $installRoot = $remaining[$i]; continue }
        '^--install-root=(.+)$' { $installRoot = $Matches[1]; continue }
        '^--bin-dir$' { $i++; $localBinDir = $remaining[$i]; continue }
        '^--bin-dir=(.+)$' { $localBinDir = $Matches[1]; continue }
        '^(-h|--help|help)$' {
            @'
flavor installer

Usage:
  install.ps1
  install.ps1 install [--channel stable|beta] [--version vX.Y.Z] [--public-url <url>]
  install.ps1 upgrade [--channel stable|beta] [--version vX.Y.Z] [--public-url <url>]
  install.ps1 uninstall

Environment:
  FLAVOR_RELEASES_PUBLIC_URL  # default: https://releases.flavor.perish.uk
  FLAVOR_CHANNEL
  FLAVOR_VERSION
  FLAVOR_INSTALL_ROOT
  FLAVOR_LOCAL_BIN_DIR
'@ | Write-Output
            exit 0
        }
        default { throw "unknown argument: $arg" }
    }
}

function Require-PublicUrl {
    return $publicUrl.TrimEnd('/')
}

function Install-Flavor {
    $resolvedPublicUrl = Require-PublicUrl
    $resolvedVersion = $version
    if ([string]::IsNullOrWhiteSpace($resolvedVersion)) {
        $metadataUrl = "$resolvedPublicUrl/$channel/latest/metadata.json"
        $metadata = Invoke-RestMethod -Uri $metadataUrl
        $resolvedVersion = $metadata.releaseVersion
        if ([string]::IsNullOrWhiteSpace($resolvedVersion)) {
            throw 'failed to resolve latest flavor version'
        }
    }

    $archive = 'flavor-x86_64-pc-windows-msvc.zip'
    $tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("flavor-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmpdir | Out-Null
    try {
        $archivePath = Join-Path $tmpdir $archive
        Invoke-WebRequest -Uri "$resolvedPublicUrl/$channel/versions/$resolvedVersion/$archive" -OutFile $archivePath
        $versionRoot = Join-Path $installRoot $resolvedVersion
        New-Item -ItemType Directory -Force -Path $versionRoot | Out-Null
        Expand-Archive -LiteralPath $archivePath -DestinationPath $versionRoot -Force
        New-Item -ItemType Directory -Force -Path $localBinDir | Out-Null
        Copy-Item -Force (Join-Path $versionRoot 'flavor.exe') (Join-Path $localBinDir 'flavor.exe')
        & (Join-Path $localBinDir 'flavor.exe') --version
        Write-Output "installed flavor to $(Join-Path $localBinDir 'flavor.exe')"
    }
    finally {
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmpdir
    }
}

function Uninstall-Flavor {
    Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $localBinDir 'flavor.exe')
    Write-Output "removed $(Join-Path $localBinDir 'flavor.exe')"
}

switch ($command) {
    'install' { Install-Flavor }
    'upgrade' { Install-Flavor }
    'uninstall' { Uninstall-Flavor }
    default { throw "unknown command: $command" }
}
