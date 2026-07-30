[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$scratchRoot = Join-Path $repoRoot '.tmp\release-flow-tests'

function Invoke-Git {
    param([string]$Root, [Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)
    & git -C $Root @Arguments | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "git $($Arguments -join ' ') failed." }
}

function New-FixtureRepository {
    param([string]$Name)
    $root = Join-Path $scratchRoot $Name
    New-Item -ItemType Directory -Force -Path (Join-Path $root 'scripts') | Out-Null
    Copy-Item (Join-Path $repoRoot 'scripts\release-tag-baseline.ps1') (Join-Path $root 'scripts')
    Copy-Item (Join-Path $repoRoot 'scripts\prepare-release.ps1') (Join-Path $root 'scripts')
    [System.IO.File]::WriteAllText(
        (Join-Path $root 'Cargo.toml'),
        "[package]`nname = `"rappct`"`nversion = `"0.13.10`"`n",
        [System.Text.UTF8Encoding]::new($false)
    )
    [System.IO.File]::WriteAllText(
        (Join-Path $root 'Cargo.lock'),
        "[[package]]`nname = `"rappct`"`nversion = `"0.13.10`"`n",
        [System.Text.UTF8Encoding]::new($false)
    )
    [System.IO.File]::WriteAllText(
        (Join-Path $root 'CHANGELOG.md'),
        "# Changelog`n`n## [Unreleased]`n`n### Changed`n`n- Prepared feature.`n`n## [0.13.3]`n",
        [System.Text.UTF8Encoding]::new($false)
    )
    Invoke-Git $root init
    Invoke-Git $root config user.email 'release-flow@example.invalid'
    Invoke-Git $root config user.name 'Release Flow Test'
    Invoke-Git $root add .
    Invoke-Git $root commit -m baseline
    $root
}

function Assert-Equal {
    param($Expected, $Actual, [string]$Message)
    if ($Expected -ne $Actual) { throw "$Message Expected '$Expected', got '$Actual'." }
}

function Test-LegacyAndCanonicalBaselines {
    $legacy = New-FixtureRepository 'legacy'
    Invoke-Git $legacy tag rappct-v0.13.3
    $selected = & (Join-Path $repoRoot 'scripts\release-tag-baseline.ps1') `
        -RepositoryRoot $legacy -TargetVersion 0.14.0
    Assert-Equal 'rappct-v0.13.3' $selected 'Legacy baseline selection failed.'

    $mixed = New-FixtureRepository 'mixed'
    Invoke-Git $mixed tag rappct-v0.13.3
    Invoke-Git $mixed tag v0.13.3
    Invoke-Git $mixed tag dev-v9.0.0
    $selected = & (Join-Path $repoRoot 'scripts\release-tag-baseline.ps1') `
        -RepositoryRoot $mixed -TargetVersion 0.14.0
    Assert-Equal 'v0.13.3' $selected 'Canonical duplicate should be preferred.'
}

function Test-PreparationIsReviewOnly {
    $fixture = New-FixtureRepository 'prepare'
    Invoke-Git $fixture tag rappct-v0.13.3
    $before = (& git -C $fixture rev-parse HEAD).Trim()
    & (Join-Path $fixture 'scripts\prepare-release.ps1') -Version 0.14.0
    if ($LASTEXITCODE -ne 0) { throw 'Preparation command failed.' }

    Assert-Equal $before ((& git -C $fixture rev-parse HEAD).Trim()) 'Preparation created a commit.'
    Assert-Equal '' ((& git -C $fixture tag --list v0.14.0) -join '') 'Preparation created a tag.'
    $changed = @(& git -C $fixture diff --name-only) | Sort-Object
    Assert-Equal 'Cargo.lock Cargo.toml CHANGELOG.md' ($changed -join ' ') 'Unexpected changed files.'
    $changelog = Get-Content -Raw -LiteralPath (Join-Path $fixture 'CHANGELOG.md')
    if ($changelog -notmatch '(?m)^## \[Unreleased\]\s*$' -or
        $changelog -notmatch '(?m)^## \[0\.14\.0\]') {
        throw 'Changelog promotion did not create expected sections.'
    }
}

if (Test-Path -LiteralPath $scratchRoot) {
    Remove-Item -LiteralPath $scratchRoot -Recurse -Force
}
try {
    Test-LegacyAndCanonicalBaselines
    Test-PreparationIsReviewOnly
    Write-Host 'release-flow tests passed'
} finally {
    if (Test-Path -LiteralPath $scratchRoot) {
        Remove-Item -LiteralPath $scratchRoot -Recurse -Force
    }
}
