<#
.SYNOPSIS
    Bumps rappct's Rust crate version, updates CHANGELOG.md from git commits,
    commits, tags, and optionally pushes to GitLab.

.PARAMETER Version
    The new semantic version, without a leading "v" prefix.

.PARAMETER DryRun
    Prints the planned changes without modifying files.

.PARAMETER Remote
    Git remote to push to. Defaults to "origin".

.PARAMETER NoPush
    Creates the version commit and tag locally without pushing.
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$')]
    [string]$Version,

    [switch]$DryRun,

    [string]$Remote = "origin",

    [switch]$NoPush
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
$versionTag = "v$Version"

function Invoke-Git {
    param([Parameter(Mandatory = $true)][string[]]$Arguments)
    $output = & git -C $root @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "git $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
    return @($output)
}

function Get-PackageVersion {
    $cargoToml = Get-Content -Raw -LiteralPath (Join-Path $root "Cargo.toml")
    $match = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([^"]+)"')
    if (-not $match.Success) {
        throw "Could not parse package version from Cargo.toml."
    }
    return $match.Groups[1].Value
}

function Set-RegexVersion {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Pattern,
        [Parameter(Mandatory = $true)][string]$Replace
    )

    $content = Get-Content -Raw -LiteralPath $Path
    $regex = [regex]::new($Pattern, [System.Text.RegularExpressions.RegexOptions]::Multiline)
    $newContent = $regex.Replace($content, $Replace, 1)
    if ($content -eq $newContent) {
        Write-Host "  SKIP  $Path already at $Version"
        return
    }

    if (-not $DryRun) {
        Set-Content -LiteralPath $Path -Value $newContent -NoNewline
    }
    Write-Host "  DONE  $Path"
}

function Get-PreviousReleaseTag {
    $tags = Invoke-Git @("tag", "--list", "v[0-9]*.[0-9]*.[0-9]*", "--sort=-v:refname")
    foreach ($tag in $tags) {
        if ($tag -ne $versionTag) {
            return $tag
        }
    }
    return $null
}

function Convert-CommitToCategory {
    param([string]$Subject)

    if ($Subject -match '^(feat|feature)(\(.+\))?!?:') { return "Added" }
    if ($Subject -match '^(fix|bugfix)(\(.+\))?!?:') { return "Fixed" }
    if ($Subject -match '^(security)(\(.+\))?!?:') { return "Security" }
    if ($Subject -match '^(perf|refactor)(\(.+\))?!?:') { return "Changed" }
    if ($Subject -match '^(docs)(\(.+\))?!?:') { return "Documentation" }
    if ($Subject -match '^(test|ci|build|chore)(\(.+\))?!?:') { return "Maintenance" }
    if ($Subject -match '!:' -or $Subject -match 'BREAKING CHANGE') { return "Changed" }
    return "Changed"
}

function Format-CommitBullet {
    param([string]$Raw)

    $parts = $Raw -split "`t", 2
    $sha = $parts[0]
    $subject = if ($parts.Count -gt 1) { $parts[1] } else { $Raw }
    $cleanSubject = $subject -replace '^(feat|feature|fix|bugfix|security|perf|refactor|docs|test|ci|build|chore)(\([^)]+\))?!?:\s*', ''
    return "- $cleanSubject ($sha)"
}

function Update-Changelog {
    $changelogPath = Join-Path $root "CHANGELOG.md"
    $today = Get-Date -Format "yyyy-MM-dd"
    $previousTag = Get-PreviousReleaseTag
    $range = if ($previousTag) { "$previousTag..HEAD" } else { "HEAD" }
    $commits = Invoke-Git @("log", "--reverse", "--format=%h`t%s", $range)

    if ($commits.Count -eq 0) {
        throw "No commits found for changelog range $range."
    }

    $groups = [ordered]@{
        "Added" = New-Object System.Collections.Generic.List[string]
        "Changed" = New-Object System.Collections.Generic.List[string]
        "Fixed" = New-Object System.Collections.Generic.List[string]
        "Security" = New-Object System.Collections.Generic.List[string]
        "Documentation" = New-Object System.Collections.Generic.List[string]
        "Maintenance" = New-Object System.Collections.Generic.List[string]
    }

    foreach ($commit in $commits) {
        $subject = ($commit -split "`t", 2)[1]
        $category = Convert-CommitToCategory -Subject $subject
        $groups[$category].Add((Format-CommitBullet -Raw $commit))
    }

    $sectionLines = New-Object System.Collections.Generic.List[string]
    $sectionLines.Add("## [$Version] - $today")
    $sectionLines.Add("")
    if ($previousTag) {
        $sectionLines.Add("Changes since `$previousTag`.")
        $sectionLines.Add("")
    }

    foreach ($entry in $groups.GetEnumerator()) {
        if ($entry.Value.Count -eq 0) { continue }
        $sectionLines.Add("### $($entry.Key)")
        $sectionLines.Add("")
        foreach ($bullet in $entry.Value) {
            $sectionLines.Add($bullet)
        }
        $sectionLines.Add("")
    }

    $content = Get-Content -Raw -LiteralPath $changelogPath
    $insert = ($sectionLines -join "`n").TrimEnd() + "`n`n"
    if ($content -match '(?m)^## \[Unreleased\]\s*$') {
        $newContent = [regex]::Replace($content, '(?m)^(## \[Unreleased\]\s*\r?\n)', "`${1}`n$insert", 1)
    } else {
        $newContent = $content.TrimEnd() + "`n`n$insert"
    }

    if (-not $DryRun) {
        Set-Content -LiteralPath $changelogPath -Value $newContent -NoNewline
    }
    Write-Host "  DONE  $changelogPath"
}

if (-not $DryRun) {
    $status = Invoke-Git @("status", "--porcelain")
    if ($status.Count -gt 0) {
        throw "Working tree is not clean. Commit or stash existing changes before bumping the version."
    }
}

$currentBranch = (Invoke-Git @("rev-parse", "--abbrev-ref", "HEAD") | Select-Object -First 1).Trim()
if ($currentBranch -eq "HEAD") {
    throw "Detached HEAD is not supported for version bumps."
}

if ((Invoke-Git @("tag", "-l", $versionTag)).Count -gt 0) {
    throw "Tag $versionTag already exists locally."
}

if (-not $DryRun -and -not $NoPush) {
    $remoteTag = & git -C $root ls-remote --tags $Remote "refs/tags/$versionTag"
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to inspect remote tag $versionTag on $Remote."
    }
    if ($remoteTag) {
        throw "Tag $versionTag already exists on remote $Remote."
    }
}

$currentVersion = Get-PackageVersion
if ($currentVersion -eq $Version) {
    throw "Cargo.toml is already at version $Version."
}

Write-Host "Bumping rappct from $currentVersion to $Version..."
Set-RegexVersion -Path (Join-Path $root "Cargo.toml") -Pattern '(^version\s*=\s*")([^"]+)(")' -Replace "`${1}$Version`${3}"
Set-RegexVersion -Path (Join-Path $root "Cargo.lock") -Pattern '(\[\[package\]\]\s+name = "rappct"\s+version = ")([^"]+)(")' -Replace "`${1}$Version`${3}"
Update-Changelog

if ($DryRun) {
    Write-Host "`nWOULD commit 'Bump version to $Version', tag '$versionTag', and push to '$Remote'."
    Write-Host "Dry run complete. No files were modified."
    return
}

& node (Join-Path $root "scripts/verify-version-surfaces.cjs")
if ($LASTEXITCODE -ne 0) {
    throw "Version surface verification failed."
}

Invoke-Git @("add", "--", "Cargo.toml", "Cargo.lock", "CHANGELOG.md") | Out-Null
Invoke-Git @("commit", "-m", "Bump version to $Version") | Out-Null
Invoke-Git @("tag", $versionTag) | Out-Null

if (-not $NoPush) {
    Write-Host "`nPushing branch '$currentBranch' and tag '$versionTag' to '$Remote'..."
    Invoke-Git @("push", $Remote, $currentBranch, "refs/tags/$versionTag") | Out-Null
    Write-Host "`nVersion bumped to $Version and pushed. The GitLab tag pipeline should publish crates.io and create the GitLab release."
} else {
    Write-Host "`nVersion bumped to $Version locally. Push with tags to trigger the GitLab release pipeline."
}
