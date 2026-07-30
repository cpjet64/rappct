[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$RepositoryRoot,
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$')]
    [string]$TargetVersion
)

$ErrorActionPreference = 'Stop'

function ConvertTo-Version {
    param([Parameter(Mandatory = $true)][string]$Value)
    try { return [version]::Parse($Value) } catch { throw "Invalid semantic version: $Value" }
}

$target = ConvertTo-Version $TargetVersion
$candidates = @()
$tags = & git -C $RepositoryRoot tag --merged HEAD --list
if ($LASTEXITCODE -ne 0) { throw 'Unable to list reachable release tags.' }

foreach ($tag in $tags) {
    if ($tag -notmatch '^(rappct-)?v(?<version>\d+\.\d+\.\d+)$') { continue }
    $version = ConvertTo-Version $Matches.version
    if ($version -ge $target) { continue }
    $candidates += [pscustomobject]@{
        Tag = $tag
        Version = $version
        Canonical = $tag.StartsWith('v')
    }
}

$selected = $candidates |
    Sort-Object -Property @{ Expression = 'Version'; Descending = $true },
    @{ Expression = 'Canonical'; Descending = $true } |
    Select-Object -First 1
if (-not $selected) {
    throw "No reachable vX.Y.Z or rappct-vX.Y.Z baseline exists before $TargetVersion."
}
$selected.Tag
