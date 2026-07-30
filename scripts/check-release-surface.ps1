[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root "Cargo.toml"

$metadataText = cargo metadata --format-version 1 --no-deps --manifest-path $manifest
if ($LASTEXITCODE -ne 0) {
    throw "cargo metadata failed with exit code $LASTEXITCODE"
}
$metadata = $metadataText | ConvertFrom-Json
$package = $metadata.packages | Where-Object { $_.name -eq "rappct" }
if (-not $package) {
    throw "rappct package was not found in cargo metadata"
}
if ($package.features.PSObject.Properties.Name -contains "_test_helpers") {
    throw "The production package still exposes the _test_helpers feature"
}

$forbidden = @("RAPPCT_TEST_LPAC_STATUS", "RAPPCT_TEST_FORCE_ENV", "RAPPCT_TEST_NO_CWD")
$sourceFiles = Get-ChildItem -LiteralPath (Join-Path $root "src") -Recurse -File
foreach ($name in $forbidden) {
    $match = $sourceFiles | Select-String -SimpleMatch $name
    if ($match) {
        throw "Production source still contains forbidden test hook $name"
    }
}

cargo check --manifest-path (Join-Path $root "scripts\fixtures\package-consumer\Cargo.toml") --locked
if ($LASTEXITCODE -ne 0) {
    throw "Downstream package consumer failed with exit code $LASTEXITCODE"
}

Write-Host "Release surface excludes production test hooks and compiles downstream."
