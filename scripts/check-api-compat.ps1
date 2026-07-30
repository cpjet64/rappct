[CmdletBinding()]
param(
    [string]$Baseline = "rappct-v0.13.3",
    [string]$ToolPath
)

$ErrorActionPreference = "Stop"
$requiredVersion = "0.49.0"
$root = Split-Path -Parent $PSScriptRoot

if (-not $ToolPath) {
    $localTool = Join-Path $root ".cache\tools\bin\cargo-semver-checks.exe"
    if (Test-Path -LiteralPath $localTool) {
        $ToolPath = $localTool
    } else {
        $command = Get-Command cargo-semver-checks -ErrorAction SilentlyContinue
        if ($command) {
            $ToolPath = $command.Source
        }
    }
}

if (-not $ToolPath -or -not (Test-Path -LiteralPath $ToolPath)) {
    throw "cargo-semver-checks $requiredVersion is required; install it under .cache/tools."
}

$versionOutput = & $ToolPath --version
if ($LASTEXITCODE -ne 0 -or $versionOutput -notmatch [regex]::Escape($requiredVersion)) {
    throw "Expected cargo-semver-checks $requiredVersion, got: $versionOutput"
}

$arguments = @(
    "--manifest-path", (Join-Path $root "Cargo.toml"),
    "--baseline-rev", $Baseline,
    "--release-type", "minor",
    "--all-features",
    "--color", "never"
)

$processInfo = [System.Diagnostics.ProcessStartInfo]::new()
$processInfo.FileName = $ToolPath
$processInfo.UseShellExecute = $false
$processInfo.RedirectStandardOutput = $true
$processInfo.RedirectStandardError = $true
$quotedArguments = $arguments | ForEach-Object {
    '"' + $_.Replace('"', '\"') + '"'
}
$processInfo.Arguments = $quotedArguments -join " "

$process = [System.Diagnostics.Process]::new()
$process.StartInfo = $processInfo
if (-not $process.Start()) {
    throw "Failed to start $ToolPath"
}
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
$process.WaitForExit()
$output = $stdout.Result + $stderr.Result
$exitCode = $process.ExitCode
$process.Dispose()
$output | Write-Output

$expected = @(
    "constructible_struct_adds_field",
    "enum_marked_non_exhaustive",
    "enum_variant_added",
    "struct_missing"
)
$found = [regex]::Matches($output, "(?m)^--- failure ([a-z0-9_]+):") |
    ForEach-Object { $_.Groups[1].Value } |
    Sort-Object -Unique
$unexpected = @($found | Where-Object { $_ -notin $expected })
$missing = @($expected | Where-Object { $_ -notin $found })

if ($exitCode -ne 100 -or $unexpected.Count -gt 0 -or $missing.Count -gt 0) {
    throw "API delta mismatch. Exit=$exitCode; unexpected=$unexpected; missing=$missing"
}

Write-Host "API compatibility delta matches the reviewed 0.14.0 migration set."
