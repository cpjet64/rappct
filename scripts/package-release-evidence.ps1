[CmdletBinding()]
param(
    [string]$PackageDir = 'target/package'
)

$ErrorActionPreference = 'Stop'
$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$packageRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $PackageDir))

if (-not $packageRoot.StartsWith($repoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "[package-release-evidence] PackageDir must stay inside repository root: $packageRoot"
}
if (-not (Test-Path -LiteralPath $packageRoot)) {
    throw "[package-release-evidence] Package directory does not exist: $packageRoot"
}

Set-Location -LiteralPath $repoRoot
$crates = @(Get-ChildItem -LiteralPath $packageRoot -Filter '*.crate' -File)
if ($crates.Count -eq 0) {
    throw "[package-release-evidence] No .crate files found in $packageRoot"
}

$metadata = & cargo metadata --locked --format-version 1 --all-features
if ($LASTEXITCODE -ne 0) {
    throw "[package-release-evidence] cargo metadata failed with exit code $LASTEXITCODE"
}

$metadataPath = Join-Path $packageRoot 'cargo-metadata.json'
$metadataText = ($metadata -join [System.Environment]::NewLine) + [System.Environment]::NewLine
[System.IO.File]::WriteAllText($metadataPath, $metadataText, [System.Text.Encoding]::UTF8)
Write-Host "[package-release-evidence] wrote $metadataPath"

$sbomPath = Join-Path $packageRoot 'rappct.cdx.json'
& python (Join-Path $PSScriptRoot 'generate_sbom.py') --output $sbomPath
if ($LASTEXITCODE -ne 0) {
    throw "[package-release-evidence] SBOM generation failed with exit code $LASTEXITCODE"
}

foreach ($crate in $crates) {
    $hash = Get-FileHash -LiteralPath $crate.FullName -Algorithm SHA256
    $checksumPath = "$($crate.FullName).sha256"
    $line = "{0}  {1}" -f $hash.Hash.ToLowerInvariant(), $crate.Name
    [System.IO.File]::WriteAllText(
        $checksumPath,
        $line + [System.Environment]::NewLine,
        [System.Text.Encoding]::ASCII
    )
    Write-Host "[package-release-evidence] wrote $checksumPath"
}
