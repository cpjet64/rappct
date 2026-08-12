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

function Assert-Fails {
    param([scriptblock]$Action, [string]$Pattern, [string]$Message)
    try {
        & $Action
    } catch {
        if ($_.Exception.Message -match $Pattern) { return }
        throw "$Message Unexpected error: $($_.Exception.Message)"
    }
    throw "$Message The command unexpectedly succeeded."
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

function Test-AlreadyVersionedCandidateFinalization {
    $fixture = New-FixtureRepository 'candidate'
    Invoke-Git $fixture tag rappct-v0.13.3
    $cargoToml = Join-Path $fixture 'Cargo.toml'
    $cargoLock = Join-Path $fixture 'Cargo.lock'
    $changelogPath = Join-Path $fixture 'CHANGELOG.md'
    $unicodeText = "phases 1$([char]0x2013)3 and warning $([char]0x26A0)"
    [System.IO.File]::WriteAllText(
        $cargoToml,
        (Get-Content -Raw $cargoToml).Replace('0.13.10', '0.14.0'),
        [System.Text.UTF8Encoding]::new($false)
    )
    [System.IO.File]::WriteAllText(
        $cargoLock,
        (Get-Content -Raw $cargoLock).Replace('0.13.10', '0.14.0'),
        [System.Text.UTF8Encoding]::new($false)
    )
    [System.IO.File]::WriteAllText(
        $changelogPath,
        "# Changelog`n`n## [Unreleased]`n`n### Fixed`n`n- Late fix.`n`n" +
            "## [0.14.0] - Unreleased`n`n### Added`n`n- Candidate feature.`n`n" +
            "## [0.13.3]`n`n- Preserves $unicodeText text.`n",
        [System.Text.UTF8Encoding]::new($false)
    )
    Invoke-Git $fixture add .
    Invoke-Git $fixture commit -m candidate

    & (Join-Path $fixture 'scripts\prepare-release.ps1') -Version 0.14.0
    $changed = @(& git -C $fixture diff --name-only)
    Assert-Equal 'CHANGELOG.md' ($changed -join ' ') 'Candidate finalization changed version files.'
    $changelog = [System.IO.File]::ReadAllText(
        $changelogPath,
        [System.Text.Encoding]::UTF8
    )
    $escapedUnicode = [regex]::Escape($unicodeText)
    if ($changelog -match '(?m)^## \[0\.14\.0\] - Unreleased\r?$' -or
        $changelog -notmatch '(?m)^## \[0\.14\.0\] - \d{4}-\d{2}-\d{2}\r?$' -or
        $changelog -notmatch 'Late fix' -or $changelog -notmatch 'Candidate feature' -or
        $changelog -notmatch $escapedUnicode) {
        throw "Already-versioned candidate was not finalized correctly:`n$changelog"
    }
}

function Test-PublishedCrateVerification {
    $packageDir = Join-Path $scratchRoot 'published'
    New-Item -ItemType Directory -Force -Path $packageDir | Out-Null
    $crateName = 'rappct-0.14.0.crate'
    $packaged = Join-Path $packageDir $crateName
    $published = Join-Path $scratchRoot 'published-copy.crate'
    [System.IO.File]::WriteAllBytes($packaged, [byte[]](1, 3, 3, 7))
    Copy-Item -LiteralPath $packaged -Destination $published

    $relativePackage = $packageDir.Substring($repoRoot.TrimEnd('\', '/').Length).TrimStart('\', '/')
    & (Join-Path $repoRoot 'scripts\verify-published-crate.ps1') `
        -PackageDir $relativePackage -PublishedCratePath $published
    $evidence = Get-Content -Raw -LiteralPath (Join-Path $packageDir "$crateName.published.sha256")
    if ($evidence -notmatch '(?m)^crate=rappct-0\.14\.0\.crate$' -or
        $evidence -notmatch '(?m)^version=0\.14\.0$' -or
        $evidence -notmatch '(?m)^packaged_sha256=([0-9a-f]{64})$' -or
        $evidence -notmatch '(?m)^published_sha256=([0-9a-f]{64})$') {
        throw 'Published-crate evidence is incomplete.'
    }

    [System.IO.File]::WriteAllBytes($published, [byte[]](9, 9, 9))
    Assert-Fails {
        & (Join-Path $repoRoot 'scripts\verify-published-crate.ps1') `
            -PackageDir $relativePackage -PublishedCratePath $published
    } 'does not match packaged hash' 'Mismatched published bytes were accepted.'
}

function Test-PublishedCrateInputValidation {
    Assert-Fails {
        & (Join-Path $repoRoot 'scripts\verify-published-crate.ps1') `
            -PackageDir '..\outside-repository' -PublishedCratePath 'missing'
    } 'must stay inside the repository' 'An external package directory was accepted.'
    Assert-Fails {
        & (Join-Path $repoRoot 'scripts\verify-published-crate.ps1') -Attempts 0
    } 'Attempts' 'A zero retry count was accepted.'
    Assert-Fails {
        & (Join-Path $repoRoot 'scripts\verify-published-crate.ps1') -DelaySeconds -1
    } 'DelaySeconds' 'A negative retry delay was accepted.'
}

if (Test-Path -LiteralPath $scratchRoot) {
    Remove-Item -LiteralPath $scratchRoot -Recurse -Force
}
try {
    Test-LegacyAndCanonicalBaselines
    Test-PreparationIsReviewOnly
    Test-AlreadyVersionedCandidateFinalization
    Test-PublishedCrateVerification
    Test-PublishedCrateInputValidation
    Write-Host 'release-flow tests passed'
} finally {
    if (Test-Path -LiteralPath $scratchRoot) {
        Remove-Item -LiteralPath $scratchRoot -Recurse -Force
    }
}
