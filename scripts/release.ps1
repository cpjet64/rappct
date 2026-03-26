[CmdletBinding()]
param(
    [string]$Crate = "rappct",
    [switch]$SkipGate
)

$ErrorActionPreference = "Stop"
$GitExe = (Get-Command git.exe -ErrorAction Stop).Source

function Invoke-Git {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    $output = & $script:GitExe @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "git command failed: git $($Arguments -join ' ') (exit $LASTEXITCODE)"
    }

    if ($null -eq $output) {
        return @()
    }

    if ($output -is [string]) {
        return @($output)
    }

    return @($output)
}

function Assert-CleanWorkingTree {
    $status = Invoke-Git @("status", "--short")
    $statusCount = @($status).Count

    if ($statusCount -gt 0) {
        Write-Host "[release] git status --short:"
        Write-Host $status
        throw "Publish is blocked: working tree has uncommitted changes. Commit or stash before running release."
    }

    Write-Host "[release] Working tree is clean."
}

function Assert-PublishBranch {
    $allowed = @("main")
    $branch = (Invoke-Git @("rev-parse", "--abbrev-ref", "HEAD")).Trim()
    if ([string]::IsNullOrWhiteSpace($branch)) {
        throw "Unable to determine current branch."
    }

    if ($branch -eq "HEAD") {
        throw "Publish is blocked from detached HEAD. Check out main (or an explicit release branch) first."
    }

    if (-not ($allowed -contains $branch)) {
        throw "Publish is blocked on branch '$branch'. Expected one of: $($allowed -join ', ')."
    }
}

function Test-CrateAuth {
    if ($env:CARGO_REGISTRY_TOKEN -and $env:CARGO_REGISTRY_TOKEN.Trim()) {
        Write-Host "Publish auth source: CARGO_REGISTRY_TOKEN is present."
        return $true
    }

    $candidates = @()
    if ($env:CARGO_HOME -and $env:CARGO_HOME.Trim()) {
        $candidates += (Join-Path $env:CARGO_HOME "credentials.toml")
    }
    if ($env:USERPROFILE -and $env:USERPROFILE.Trim()) {
        $candidates += (Join-Path $env:USERPROFILE ".cargo\credentials.toml")
    }
    if ($env:HOME -and $env:HOME.Trim()) {
        $candidates += (Join-Path $env:HOME ".cargo\credentials.toml")
    }

    foreach ($credentialFile in ($candidates | Select-Object -Unique)) {
        if (Test-Path $credentialFile) {
            Write-Host "Publish auth source: credential file exists at $credentialFile."
            return $true
        }
    }

    return $false
}

if (-not $SkipGate) {
    & ./scripts/release_gate.ps1 -Crate $Crate
}

Assert-PublishBranch
Assert-CleanWorkingTree

Write-Host "Explicit publish confirmation is required for: $Crate"
$confirm = Read-Host "Type 'PUBLISH' to continue with cargo publish"
if ($confirm -ne "PUBLISH") {
    Write-Host "Publish cancelled."
    return
}

if (-not (Test-CrateAuth)) {
    Write-Warning "No publish credential detected. Set CARGO_REGISTRY_TOKEN in your shell or run 'cargo login' before continuing."
    $proceed = Read-Host "No credential detected. Type 'FORCE' to continue, or anything else to stop"
    if ($proceed -ne "FORCE") {
        Write-Host "Publish stopped to avoid non-interactive publishing."
        return
    }
}

Write-Host "Running cargo publish --locked -p $Crate"
& cargo publish --locked -p $Crate
if ($LASTEXITCODE -ne 0) {
    throw "cargo publish failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "Publish completed successfully for $Crate."
