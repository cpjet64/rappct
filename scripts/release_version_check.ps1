[CmdletBinding()]
param(
    [string]$Crate = "rappct"
)

$ErrorActionPreference = "Stop"

function Parse-SemVer {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Version
    )

    $match = [regex]::Match(
        $Version,
        '^(?<major>0|[1-9]\d*)\.(?<minor>0|[1-9]\d*)\.(?<patch>0|[1-9]\d*)(?:-(?<pre>[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+(?<build>[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$'
    )

    if (-not $match.Success) {
        throw "Unsupported version format: '$Version'. Expected semver x.y.z."
    }

    $prerelease = @()
    if ($match.Groups["pre"].Success -and -not [string]::IsNullOrWhiteSpace($match.Groups["pre"].Value)) {
        $prerelease = $match.Groups["pre"].Value -split "\."
        foreach ($identifier in $prerelease) {
            if ($identifier -match '^\d+$' -and $identifier.Length -gt 1 -and $identifier.StartsWith("0")) {
                throw "Invalid prerelease identifier '$identifier' in version '$Version': numeric identifiers must not have leading zeroes."
            }
        }
    }

    [pscustomobject]@{
        Raw = $Version
        Major = $match.Groups["major"].Value
        Minor = $match.Groups["minor"].Value
        Patch = $match.Groups["patch"].Value
        Pre = $prerelease
    }
}

function Compare-DecimalString {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Left,
        [Parameter(Mandatory = $true)]
        [string]$Right
    )

    $leftNormalized = $Left.TrimStart("0")
    if ([string]::IsNullOrEmpty($leftNormalized)) {
        $leftNormalized = "0"
    }

    $rightNormalized = $Right.TrimStart("0")
    if ([string]::IsNullOrEmpty($rightNormalized)) {
        $rightNormalized = "0"
    }

    if ($leftNormalized.Length -ne $rightNormalized.Length) {
        return [Math]::Sign($leftNormalized.Length - $rightNormalized.Length)
    }

    return [Math]::Sign([string]::CompareOrdinal($leftNormalized, $rightNormalized))
}

function Compare-SemVer {
    param(
        [Parameter(Mandatory = $true)] $Left,
        [Parameter(Mandatory = $true)] $Right
    )

    $majorComparison = Compare-DecimalString -Left $Left.Major -Right $Right.Major
    if ($majorComparison -ne 0) { return $majorComparison }

    $minorComparison = Compare-DecimalString -Left $Left.Minor -Right $Right.Minor
    if ($minorComparison -ne 0) { return $minorComparison }

    $patchComparison = Compare-DecimalString -Left $Left.Patch -Right $Right.Patch
    if ($patchComparison -ne 0) { return $patchComparison }

    $leftPre = $Left.Pre
    $rightPre = $Right.Pre

    if ($leftPre.Count -eq 0 -and $rightPre.Count -eq 0) {
        return 0
    }

    if ($leftPre.Count -eq 0) {
        return 1
    }

    if ($rightPre.Count -eq 0) {
        return -1
    }

    $limit = [Math]::Min($leftPre.Count, $rightPre.Count)
    for ($i = 0; $i -lt $limit; $i++) {
        $a = $leftPre[$i]
        $b = $rightPre[$i]

        $isANumeric = $a -match '^\d+$'
        $isBNumeric = $b -match '^\d+$'

        if ($isANumeric -and $isBNumeric) {
            $numericComparison = Compare-DecimalString -Left $a -Right $b
            if ($numericComparison -ne 0) {
                return $numericComparison
            }
            continue
        }

        if ($isANumeric -ne $isBNumeric) {
            if ($isANumeric) {
                return -1
            }
            return 1
        }

        if ($a -ne $b) {
            return [Math]::Sign([string]::CompareOrdinal($a, $b))
        }
    }

    return [Math]::Sign($leftPre.Count - $rightPre.Count)
}

function Compare-Versions {
    param(
        [Parameter(Mandatory = $true)] [string]$Left,
        [Parameter(Mandatory = $true)] [string]$Right
    )

    $leftParsed = Parse-SemVer $Left
    $rightParsed = Parse-SemVer $Right
    return Compare-SemVer $leftParsed $rightParsed
}

$tomlPath = Join-Path -Path (Get-Location) -ChildPath "Cargo.toml"
$tomlLines = Get-Content -Path $tomlPath
$inPackageSection = $false
$localMatch = $null

foreach ($line in $tomlLines) {
    if ($line -match '^\s*\[package\]\s*$') {
        $inPackageSection = $true
        continue
    }

    if ($line -match '^\s*\[[^\]]+\]\s*$') {
        if ($inPackageSection) {
            break
        }
        continue
    }

    if (-not $inPackageSection) {
        continue
    }

    $localMatch = [regex]::Match(
        $line,
        '^\s*version\s*=\s*"([^"]+)"(?:\s*#.*)?$'
    )
    if ($localMatch.Success) {
        break
    }
}

if (-not $localMatch.Success) {
    throw "Could not parse version from Cargo.toml"
}
$localVersion = $localMatch.Groups[1].Value

$cratesUrl = "https://crates.io/api/v1/crates/$Crate/versions"
Write-Output "Checking published versions for $Crate at $cratesUrl"

try {
    $versionsResponse = Invoke-RestMethod -Uri $cratesUrl -Method Get -TimeoutSec 20
}
catch {
    $statusCode = $null
    if ($_.Exception.Response -and $_.Exception.Response.StatusCode) {
        $statusCode = [int]$_.Exception.Response.StatusCode
    }

    if ($statusCode -eq 404) {
        Write-Output "No published versions found for $Crate (crate not found on crates.io). Assuming first publish; version floor check skipped."
        exit 0
    }

    throw "Failed to query crates.io for '$Crate': $($_.Exception.Message)"
}

$allVersions = @(
    $versionsResponse.versions |
        Where-Object { -not $_.yanked } |
        Where-Object { $_.num -and -not $_.num.Contains("-") } |
        Select-Object -ExpandProperty num
)

if ($allVersions.Count -eq 0) {
    Write-Output "No non-yanked stable versions found for $Crate. Assuming first stable publish; version floor check skipped."
    exit 0
}

$highest = $null
foreach ($version in $allVersions) {
    if ($null -eq $highest) {
        $highest = $version
        continue
    }

    if ((Compare-Versions $version $highest) -gt 0) {
        $highest = $version
    }
}

$comparison = Compare-Versions $localVersion $highest
if ($comparison -le 0) {
    throw "Version check failed for ${Crate}: local ${localVersion} is not greater than published ${highest}"
}

Write-Output "Version check passed for ${Crate}: local=${localVersion} > published=${highest}"
