param(
    [ValidateSet('none', 'beta', 'stable')]
    [string]$ReleaseChannel = 'none'
)

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
Set-Location $repoRoot

function Fail($message) {
    throw "version guard: $message"
}

function Warn($message) {
    Write-Warning "version guard: $message"
}

function Read-JsonFile($path) {
    if (-not (Test-Path -LiteralPath $path)) {
        Fail "missing $path"
    }
    Get-Content -Raw -LiteralPath $path | ConvertFrom-Json
}

function Get-CliVersion {
    $cargoPath = Join-Path $repoRoot 'crates/flavor-cli/Cargo.toml'
    $line = Get-Content -LiteralPath $cargoPath | Where-Object { $_ -match '^version\s*=\s*"([^"]+)"' } | Select-Object -First 1
    if (-not $line) {
        Fail "missing package version in crates/flavor-cli/Cargo.toml"
    }
    [version]$Matches[1]
}

function Convert-GlobToRegex($glob) {
    $normalized = $glob -replace '\\', '/'
    $builder = [System.Text.StringBuilder]::new()
    [void]$builder.Append('^')
    for ($i = 0; $i -lt $normalized.Length; $i++) {
        $ch = $normalized[$i]
        if ($ch -eq '*') {
            if (($i + 1) -lt $normalized.Length -and $normalized[$i + 1] -eq '*') {
                [void]$builder.Append('.*')
                $i++
            } else {
                [void]$builder.Append('[^/]*')
            }
            continue
        }
        if ('\.[]{}()+-^$?|'.Contains($ch)) {
            [void]$builder.Append('\')
        }
        [void]$builder.Append($ch)
    }
    [void]$builder.Append('$')
    $builder.ToString()
}

function Get-ScopeFiles($scopes) {
    $regexes = @($scopes | ForEach-Object { [regex](Convert-GlobToRegex $_) })
    $files = Get-ChildItem -LiteralPath $repoRoot -Recurse -File -Force |
        ForEach-Object {
            $relative = [System.IO.Path]::GetRelativePath($repoRoot, $_.FullName) -replace '\\', '/'
            foreach ($regex in $regexes) {
                if ($regex.IsMatch($relative)) {
                    $relative
                    break
                }
            }
        } |
        Sort-Object -Unique
    @($files)
}

function Get-FileSha256($relative) {
    $path = Join-Path $repoRoot ($relative -replace '/', [System.IO.Path]::DirectorySeparatorChar)
    (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-TestsHash($scopes) {
    if (-not $scopes -or $scopes.Count -eq 0) {
        Fail "metadata.tests.scopes must be a non-empty array"
    }
    foreach ($scope in $scopes) {
        if ([string]::IsNullOrWhiteSpace($scope)) {
            Fail "metadata.tests.scopes contains an empty scope"
        }
    }

    $files = Get-ScopeFiles $scopes
    if ($files.Count -eq 0) {
        Fail "metadata.tests.scopes matched 0 files"
    }

    $records = foreach ($file in $files) {
        "$file`n$(Get-FileSha256 $file)`n"
    }
    $bytes = [System.Text.Encoding]::UTF8.GetBytes(($records -join ''))
    $hash = [System.Security.Cryptography.SHA256]::HashData($bytes)
    "sha256:" + ([Convert]::ToHexString($hash).ToLowerInvariant())
}

function Read-R2StableMetadata {
    if ($ReleaseChannel -eq 'none') {
        return $null
    }
    $metadataUrl = $env:FLAVOR_STABLE_METADATA_URL
    if ([string]::IsNullOrWhiteSpace($metadataUrl)) {
        $publicUrl = ($env:FLAVOR_RELEASES_PUBLIC_URL ?? '').TrimEnd('/')
        if ([string]::IsNullOrWhiteSpace($publicUrl)) {
            Fail "FLAVOR_RELEASES_PUBLIC_URL or FLAVOR_STABLE_METADATA_URL is required for release version guard"
        }
        $metadataUrl = "$publicUrl/stable/latest/metadata.json"
    }

    try {
        $response = Invoke-WebRequest -Uri $metadataUrl -Headers @{ 'Cache-Control' = 'no-cache' } -TimeoutSec 10
        $response.Content | ConvertFrom-Json
    } catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        if ($statusCode -eq 404) {
            Write-Output "version guard: R2 stable latest metadata not found; skipping release baseline comparison"
            return $null
        }
        Fail "failed to fetch R2 stable metadata ${metadataUrl}: $($_.Exception.Message)"
    }
}

function Get-MetadataVersion($metadata) {
    $value = $metadata.stableVersion ?? $metadata.releaseVersion ?? $metadata.baseVersion
    if ([string]::IsNullOrWhiteSpace($value)) {
        Fail "R2 stable metadata must include stableVersion, releaseVersion, or baseVersion"
    }
    $value = $value -replace '^v', ''
    [version]$value
}

$metadata = Read-JsonFile (Join-Path $repoRoot 'metadata.json')
if (-not $metadata.tests) {
    Fail "metadata.json must contain tests"
}
if ([string]::IsNullOrWhiteSpace($metadata.tests.hash)) {
    Fail "metadata.tests.hash is required"
}

$scopes = @($metadata.tests.scopes)
$actualHash = Get-TestsHash $scopes
if ($actualHash -ne $metadata.tests.hash) {
    Fail "metadata.tests.hash is $($metadata.tests.hash), expected $actualHash"
}

$version = Get-CliVersion
$versionName = "v$($version.Major).$($version.Minor).$($version.Build)"
$changelogDir = Join-Path $repoRoot "docs/changelog/$versionName"
if (-not (Test-Path -LiteralPath (Join-Path $changelogDir 'INDEX.md'))) {
    Fail "missing docs/changelog/$versionName/INDEX.md"
}
if ($version.Build -eq 0 -and -not (Test-Path -LiteralPath (Join-Path $changelogDir 'MIGRATION.md'))) {
    Fail "missing docs/changelog/$versionName/MIGRATION.md for Y version"
}

$r2Metadata = Read-R2StableMetadata
if ($r2Metadata) {
    if (-not $r2Metadata.tests) {
        Warn "R2 stable metadata is missing tests; skipping release baseline comparison for metadata bootstrap"
        Write-Output "version guard passed: $versionName $actualHash ($ReleaseChannel)"
        exit 0
    }
    $r2Version = Get-MetadataVersion $r2Metadata
    $r2Scopes = @($r2Metadata.tests.scopes)
    $scopesChanged = (@($r2Scopes) -join "`n") -ne (@($scopes) -join "`n")
    $hashChanged = $r2Metadata.tests.hash -ne $metadata.tests.hash
    if ($scopesChanged -or $hashChanged) {
        $minorAdvanced = $version.Major -gt $r2Version.Major -or
            ($version.Major -eq $r2Version.Major -and $version.Minor -gt $r2Version.Minor)
        if (-not $minorAdvanced -or $version.Build -ne 0) {
            $message = "tests hash/scope changed; version should advance to a new Y release with patch 0 before stable release"
            if ($ReleaseChannel -eq 'beta') {
                Warn $message
            } else {
                Fail $message
            }
        }
    }
}

Write-Output "version guard passed: $versionName $actualHash ($ReleaseChannel)"
