[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root "Cargo.toml"
$fixture = Join-Path $root "scripts\fixtures\package-consumer"
$scratch = Join-Path $root ".tmp\release-surface-package-consumer"
$targetDir = Join-Path $scratch "target"
$packageDir = Join-Path $targetDir "package"

$metadataText = cargo metadata --offline --format-version 1 --no-deps --manifest-path $manifest
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

New-Item -ItemType Directory -Force -Path $scratch | Out-Null
$env:CARGO_TARGET_DIR = $targetDir
& cargo package --allow-dirty --locked --offline --no-verify --manifest-path $manifest
if ($LASTEXITCODE -ne 0) {
    throw "cargo package failed with exit code $LASTEXITCODE"
}

$crate = Get-ChildItem -LiteralPath $packageDir -Filter 'rappct-*.crate' -File |
    Sort-Object LastWriteTimeUtc -Descending |
    Select-Object -First 1
if (-not $crate) {
    throw "cargo package did not produce a rappct crate"
}

$packageRoot = Join-Path $scratch "crate"
$consumerRoot = Join-Path $scratch "consumer"
Remove-Item -LiteralPath $packageRoot -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $consumerRoot -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $packageRoot | Out-Null
& tar -xf $crate.FullName -C $packageRoot
if ($LASTEXITCODE -ne 0) {
    throw "crate extraction failed with exit code $LASTEXITCODE"
}

$packagedManifest = Get-ChildItem -LiteralPath $packageRoot -Filter 'Cargo.toml' -Recurse -File |
    Select-Object -First 1
if (-not $packagedManifest) {
    throw "The packaged crate did not contain Cargo.toml"
}

Copy-Item -LiteralPath $fixture -Destination $consumerRoot -Recurse
$consumerManifest = Join-Path $consumerRoot "Cargo.toml"
$consumerToml = Get-Content -LiteralPath $consumerManifest -Raw
$escapedPackagePath = $packagedManifest.Directory.FullName.Replace('\', '\\')
$consumerToml = $consumerToml -replace 'path = "\.\.\/\.\.\/\.\."', "path = `"$escapedPackagePath`""
[System.IO.File]::WriteAllText($consumerManifest, $consumerToml, [System.Text.Encoding]::UTF8)

& cargo check --manifest-path $consumerManifest --locked --offline
if ($LASTEXITCODE -ne 0) {
    throw "Packaged downstream consumer failed with exit code $LASTEXITCODE"
}

Write-Host "Release surface excludes production test hooks and compiles the cargo-packaged crate downstream."
