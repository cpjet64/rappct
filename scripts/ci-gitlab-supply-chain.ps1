[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

function Clear-ProcessEnvironmentVariable {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    if ($null -eq [Environment]::GetEnvironmentVariable($Name, 'Process')) {
        return
    }

    Write-Host "[ci-gitlab-supply-chain] clearing inherited $Name for audit tooling"
    [Environment]::SetEnvironmentVariable($Name, $null, 'Process')
}

Clear-ProcessEnvironmentVariable -Name 'RUSTC_WRAPPER'
Clear-ProcessEnvironmentVariable -Name 'CARGO_BUILD_RUSTC_WRAPPER'

& (Join-Path $PSScriptRoot 'with-project-temp.ps1') `
    -Scope gitlab-supply-chain `
    just `
    security

$exitCode = $LASTEXITCODE
if ($null -eq $exitCode) {
    $exitCode = if ($?) { 0 } else { 1 }
}

exit $exitCode
