[CmdletBinding()]
param(
    [string]$PackageDir = 'target/package',
    [ValidateRange(1, 1000)]
    [int]$Attempts = 30,
    [ValidateRange(0, 3600)]
    [int]$DelaySeconds = 10,
    [string]$PublishedCratePath
)

$ErrorActionPreference = 'Stop'

function Resolve-ContainedPath {
    param([string]$Root, [string]$Path, [string]$Description)
    $candidate = if ([System.IO.Path]::IsPathRooted($Path)) {
        $Path
    } else {
        Join-Path $Root $Path
    }
    $resolved = [System.IO.Path]::GetFullPath($candidate)
    $rootPrefix = $Root.TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar
    if (-not $resolved.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "[verify-published-crate] $Description must stay inside the repository: $resolved"
    }
    $resolved
}

function Get-CrateVersion {
    param([string]$Root)
    $manifest = Get-Content -LiteralPath (Join-Path $Root 'Cargo.toml') -Raw
    $match = [regex]::Match($manifest, '(?m)^version\s*=\s*"(?<version>[^"]+)"')
    if (-not $match.Success) {
        throw '[verify-published-crate] Cargo.toml package version is missing.'
    }
    $match.Groups['version'].Value
}

function Get-Sha256 {
    param([string]$Path)
    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $algorithm = [System.Security.Cryptography.SHA256]::Create()
        try {
            ([System.BitConverter]::ToString($algorithm.ComputeHash($stream)) -replace '-', '').ToLowerInvariant()
        } finally {
            $algorithm.Dispose()
        }
    } finally {
        $stream.Dispose()
    }
}

function Get-PublishedCrate {
    param(
        [string]$Destination,
        [string]$Uri,
        [string]$LocalSource,
        [int]$RetryCount,
        [int]$RetryDelay
    )
    if (-not [string]::IsNullOrWhiteSpace($LocalSource)) {
        Copy-Item -LiteralPath $LocalSource -Destination $Destination
        return "file:$([System.IO.Path]::GetFileName($LocalSource))"
    }
    for ($attempt = 1; $attempt -le $RetryCount; $attempt++) {
        try {
            Invoke-WebRequest -Uri $Uri -OutFile $Destination -UseBasicParsing
            return $Uri
        } catch {
            if ($attempt -eq $RetryCount) { throw }
            Start-Sleep -Seconds $RetryDelay
        }
    }
    throw '[verify-published-crate] Published crate did not become available.'
}

function Write-VerificationEvidence {
    param(
        [string]$Path,
        [string]$Crate,
        [string]$Version,
        [string]$ExpectedHash,
        [string]$PublishedHash,
        [string]$Source
    )
    $lines = @(
        "crate=$Crate"
        "version=$Version"
        "tag=$($env:CI_COMMIT_TAG)"
        "commit_sha=$($env:CI_COMMIT_SHA)"
        "packaged_sha256=$ExpectedHash"
        "published_sha256=$PublishedHash"
        "source=$Source"
    )
    [System.IO.File]::WriteAllText($Path, ($lines -join "`n") + "`n", [System.Text.Encoding]::ASCII)
}

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$version = Get-CrateVersion -Root $repoRoot
$crateName = "rappct-$version.crate"
$packageRoot = Resolve-ContainedPath -Root $repoRoot -Path $PackageDir -Description 'Package directory'
$cratePath = Join-Path $packageRoot $crateName
if (-not (Test-Path -LiteralPath $cratePath -PathType Leaf)) {
    throw "[verify-published-crate] Expected package artifact is missing: $cratePath"
}
$localPublished = $null
if (-not [string]::IsNullOrWhiteSpace($PublishedCratePath)) {
    $localPublished = [System.IO.Path]::GetFullPath($PublishedCratePath)
    if (-not (Test-Path -LiteralPath $localPublished -PathType Leaf)) {
        throw "[verify-published-crate] Published crate input is missing: $localPublished"
    }
}

$jobScope = if ([string]::IsNullOrWhiteSpace($env:CI_JOB_ID)) { 'local' } else { $env:CI_JOB_ID }
$scratch = Join-Path $repoRoot ".tmp\release-verify\$jobScope"
$download = Join-Path $scratch $crateName
$uri = "https://crates.io/api/v1/crates/rappct/$version/download"
[System.IO.Directory]::CreateDirectory($scratch) | Out-Null
try {
    $source = Get-PublishedCrate -Destination $download -Uri $uri -LocalSource $localPublished `
        -RetryCount $Attempts -RetryDelay $DelaySeconds
    $expected = Get-Sha256 -Path $cratePath
    $published = Get-Sha256 -Path $download
    if ($published -ne $expected) {
        throw "[verify-published-crate] crates.io hash $published does not match packaged hash $expected."
    }

    $evidencePath = Join-Path $packageRoot "$crateName.published.sha256"
    Write-VerificationEvidence -Path $evidencePath -Crate $crateName -Version $version `
        -ExpectedHash $expected -PublishedHash $published -Source $source
    Write-Host "[verify-published-crate] Published bytes match: $published"
} finally {
    if (Test-Path -LiteralPath $scratch) {
        Remove-Item -LiteralPath $scratch -Recurse -Force
    }
}
