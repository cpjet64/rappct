<#
.SYNOPSIS
    Creates a verified local release tag. It never pushes or publishes.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$')]
    [string]$Version,
    [string]$Branch = 'main',
    [string]$Remote = 'origin'
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
$tag = "v$Version"

function Invoke-Git {
    param([Parameter(Mandatory = $true)][string[]]$Arguments)
    $output = & git -C $root @Arguments
    if ($LASTEXITCODE -ne 0) { throw "git $($Arguments -join ' ') failed." }
    @($output)
}

if ((Invoke-Git @('status', '--porcelain')).Count -gt 0) { throw 'Working tree is not clean.' }
$currentBranch = (Invoke-Git @('branch', '--show-current') | Select-Object -First 1).Trim()
if ($currentBranch -ne $Branch) { throw "Release tags must be created from $Branch." }
$head = (Invoke-Git @('rev-parse', 'HEAD') | Select-Object -First 1).Trim()
$upstream = (Invoke-Git @('rev-parse', "$Remote/$Branch") | Select-Object -First 1).Trim()
if ($head -ne $upstream) { throw "HEAD must equal $Remote/$Branch before tagging." }
if ((Invoke-Git @('tag', '--list', $tag)).Count -gt 0) { throw "Local tag $tag already exists." }

$remoteTag = & git -C $root ls-remote --tags $Remote "refs/tags/$tag"
if ($LASTEXITCODE -ne 0) { throw "Could not inspect $Remote for $tag." }
if ($remoteTag) { throw "Remote tag $tag already exists." }

$env:CI_COMMIT_TAG = $tag
try {
    & node (Join-Path $root 'scripts/verify-version-surfaces.cjs')
    if ($LASTEXITCODE -ne 0) { throw 'Version surface verification failed.' }
} finally {
    Remove-Item Env:\CI_COMMIT_TAG -ErrorAction SilentlyContinue
}
$changelog = Get-Content -Raw -LiteralPath (Join-Path $root 'CHANGELOG.md')
if ($changelog -notmatch "(?m)^## \[$([regex]::Escape($Version))\](?:\s|$)") {
    throw "CHANGELOG.md has no $Version release section."
}

Invoke-Git @('tag', $tag) | Out-Null
Write-Host "Created local tag $tag at $head."
Write-Host "Review it, then push explicitly with: git push $Remote refs/tags/$tag"
