<#
.SYNOPSIS
    Prepares reviewable version and changelog changes without Git or remote mutation.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$')]
    [string]$Version,
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSCommandPath)

function Invoke-Git {
    param([Parameter(Mandatory = $true)][string[]]$Arguments)
    $output = & git -C $root @Arguments
    if ($LASTEXITCODE -ne 0) { throw "git $($Arguments -join ' ') failed." }
    @($output)
}

function Get-ManifestVersion {
    $content = Get-Content -Raw -LiteralPath (Join-Path $root 'Cargo.toml')
    $match = [regex]::Match($content, '(?m)^version\s*=\s*"([^"]+)"')
    if (-not $match.Success) { throw 'Could not parse Cargo.toml package version.' }
    $match.Groups[1].Value
}

function Set-FirstMatch {
    param([string]$Path, [string]$Pattern, [string]$Replacement)
    $content = Get-Content -Raw -LiteralPath $Path
    $regex = [regex]::new($Pattern, [System.Text.RegularExpressions.RegexOptions]::Multiline)
    $updated = $regex.Replace($content, $Replacement, 1)
    if ($updated -eq $content) { throw "Expected version surface was not found in $Path." }
    if (-not $DryRun) {
        [System.IO.File]::WriteAllText($Path, $updated, [System.Text.UTF8Encoding]::new($false))
    }
}

function Get-PromotedChangelog {
    param([string]$Content, [string]$Baseline)
    if ($Content -match "(?m)^## \[$([regex]::Escape($Version))\](?:\s|$)") {
        throw "CHANGELOG.md already contains a $Version section."
    }
    $match = [regex]::Match(
        $Content,
        '(?ms)^## \[Unreleased\]\s*\r?\n(?<body>.*?)(?=^## \[)'
    )
    if (-not $match.Success -or [string]::IsNullOrWhiteSpace($match.Groups['body'].Value)) {
        throw 'CHANGELOG.md must contain a non-empty Unreleased section.'
    }
    $date = Get-Date -Format 'yyyy-MM-dd'
    $body = $match.Groups['body'].Value.Trim()
    $replacement = "## [Unreleased]`r`n`r`n## [$Version] - $date`r`n`r`n" +
        "Changes since ``$Baseline``.`r`n`r`n$body`r`n`r`n"
    $Content.Remove($match.Index, $match.Length).Insert($match.Index, $replacement)
}

if ((Invoke-Git @('status', '--porcelain')).Count -gt 0) {
    throw 'Working tree must be clean before release preparation.'
}
$branch = (Invoke-Git @('rev-parse', '--abbrev-ref', 'HEAD') | Select-Object -First 1).Trim()
if ($branch -eq 'HEAD') { throw 'Detached HEAD is not supported.' }

$current = Get-ManifestVersion
if ([version]$Version -le [version]$current) {
    throw "Target $Version must be greater than current manifest version $current."
}
$baseline = & (Join-Path $PSScriptRoot 'release-tag-baseline.ps1') `
    -RepositoryRoot $root -TargetVersion $Version
if ($LASTEXITCODE -ne 0) { throw 'Release baseline selection failed.' }
$baseline = ($baseline | Select-Object -First 1).Trim()

Write-Host "Preparing $current -> $Version from $baseline..HEAD"
if ($DryRun) {
    Write-Host 'Dry run complete; no files, index entries, commits, tags, or remotes changed.'
    return
}

Set-FirstMatch (Join-Path $root 'Cargo.toml') `
    '(^version\s*=\s*")([^"]+)(")' "`${1}$Version`${3}"
Set-FirstMatch (Join-Path $root 'Cargo.lock') `
    '(\[\[package\]\]\s+name = "rappct"\s+version = ")([^"]+)(")' "`${1}$Version`${3}"
$changelogPath = Join-Path $root 'CHANGELOG.md'
$changelog = Get-Content -Raw -LiteralPath $changelogPath
$updated = Get-PromotedChangelog -Content $changelog -Baseline $baseline
[System.IO.File]::WriteAllText($changelogPath, $updated, [System.Text.UTF8Encoding]::new($false))
Write-Host 'Prepared Cargo.toml, Cargo.lock, and CHANGELOG.md. Review and commit on a topic branch.'
