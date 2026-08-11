[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$apiVersion = "2022-11-28"
$repository = "cpjet64/rappct"

function Get-RequiredEnv {
    param([Parameter(Mandatory = $true)][string]$Name)

    $value = [Environment]::GetEnvironmentVariable($Name)
    if ([string]::IsNullOrWhiteSpace($value)) {
        throw "Required environment variable '$Name' is not set."
    }
    return $value
}

function Invoke-GitHubJson {
    param(
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Uri,
        [hashtable]$Body
    )

    $params = @{
        Method = $Method
        Uri = $Uri
        Headers = $script:Headers
        ErrorAction = "Stop"
    }
    if ($null -ne $Body) {
        $params.ContentType = "application/json"
        $params.Body = $Body | ConvertTo-Json -Depth 8 -Compress
    }
    try {
        return @{ Ok = $true; Status = 200; Data = Invoke-RestMethod @params }
    } catch {
        $status = if ($_.Exception.Response) { [int]$_.Exception.Response.StatusCode } else { 0 }
        return @{ Ok = $false; Status = $status; Error = $_.Exception.Message }
    }
}

function Get-ChangelogEntry {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Version
    )

    $content = Get-Content -LiteralPath $Path -Raw
    $escaped = [regex]::Escape($Version)
    $pattern = "(?ms)^##\s+(?:\[$escaped\](?:\([^)]+\))?|$escaped)(?:\s+[^\r\n]*)?\r?\n.*?(?=^##\s+|\z)"
    $match = [regex]::Match($content, $pattern)
    if (-not $match.Success) {
        throw "CHANGELOG.md does not contain a release entry for version $Version."
    }
    return $match.Value.Trim()
}

function Push-GitHubMain {
    param([Parameter(Mandatory = $true)][string]$Token)

    $credentials = [Convert]::ToBase64String(
        [Text.Encoding]::ASCII.GetBytes("x-access-token:$Token")
    )
    $saved = @{
        Count = $env:GIT_CONFIG_COUNT
        Key = $env:GIT_CONFIG_KEY_0
        Value = $env:GIT_CONFIG_VALUE_0
        Prompt = $env:GIT_TERMINAL_PROMPT
    }
    try {
        $env:GIT_CONFIG_COUNT = "1"
        $env:GIT_CONFIG_KEY_0 = "http.https://github.com/.extraheader"
        $env:GIT_CONFIG_VALUE_0 = "AUTHORIZATION: basic $credentials"
        $env:GIT_TERMINAL_PROMPT = "0"
        & git push --quiet "https://github.com/$repository.git" "HEAD:refs/heads/main"
        if ($LASTEXITCODE -ne 0) {
            throw "GitHub main source-mirror push failed; divergence is never forced."
        }
    } finally {
        $env:GIT_CONFIG_COUNT = $saved.Count
        $env:GIT_CONFIG_KEY_0 = $saved.Key
        $env:GIT_CONFIG_VALUE_0 = $saved.Value
        $env:GIT_TERMINAL_PROMPT = $saved.Prompt
    }
}

function Assert-GitLabMainCommit {
    param([Parameter(Mandatory = $true)][string]$Commit)

    $defaultBranch = Get-RequiredEnv "CI_DEFAULT_BRANCH"
    if ($defaultBranch -ne "main") {
        throw "GitLab default branch must be main; got '$defaultBranch'."
    }
    & git fetch --quiet --no-tags origin main
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to refresh authoritative GitLab main."
    }
    $mainCommit = (& git rev-parse "refs/remotes/origin/main").Trim()
    if ($LASTEXITCODE -ne 0 -or $mainCommit -ne $Commit) {
        throw "Release commit $Commit is not the authoritative GitLab main tip $mainCommit."
    }
}

function Resolve-TagCommit {
    param([Parameter(Mandatory = $true)][string]$Tag)

    $encodedTag = [System.Uri]::EscapeDataString($Tag)
    $reference = Invoke-GitHubJson -Method GET -Uri "$script:ApiBase/git/ref/tags/$encodedTag" -Body $null
    if (-not $reference.Ok) {
        return @{ Found = $false; Status = $reference.Status }
    }
    $object = $reference.Data.object
    for ($depth = 0; $depth -lt 5 -and $object.type -eq "tag"; $depth++) {
        $tagObject = Invoke-GitHubJson -Method GET -Uri "$script:ApiBase/git/tags/$($object.sha)" -Body $null
        if (-not $tagObject.Ok) {
            throw "Failed to dereference GitHub tag $Tag. HTTP $($tagObject.Status)."
        }
        $object = $tagObject.Data.object
    }
    if ($object.type -ne "commit") {
        throw "GitHub tag $Tag does not resolve to a commit."
    }
    return @{ Found = $true; Status = 200; Commit = $object.sha }
}

function Confirm-CommitAndTag {
    param(
        [Parameter(Mandatory = $true)][string]$Commit,
        [Parameter(Mandatory = $true)][string]$Tag
    )

    $commitResult = Invoke-GitHubJson -Method GET -Uri "$script:ApiBase/commits/$Commit" -Body $null
    if (-not $commitResult.Ok) {
        throw "GitHub source mirror does not contain commit $Commit."
    }
    $mainRef = Invoke-GitHubJson -Method GET -Uri "$script:ApiBase/git/ref/heads/main" -Body $null
    if (-not $mainRef.Ok -or $mainRef.Data.object.sha -ne $Commit) {
        throw "GitHub main does not match release commit $Commit after source-mirror push."
    }
    $resolved = Resolve-TagCommit -Tag $Tag
    if ($resolved.Found) {
        if ($resolved.Commit -ne $Commit) {
            throw "GitHub tag $Tag resolves to $($resolved.Commit), expected $Commit."
        }
        return
    }
    if ($resolved.Status -ne 404) {
        throw "Failed to inspect GitHub tag $Tag. HTTP $($resolved.Status)."
    }
    $created = Invoke-GitHubJson -Method POST -Uri "$script:ApiBase/git/refs" -Body @{
        ref = "refs/tags/$Tag"
        sha = $Commit
    }
    if (-not $created.Ok) {
        throw "Failed to create GitHub tag $Tag. HTTP $($created.Status)."
    }
    $verified = Resolve-TagCommit -Tag $Tag
    if (-not $verified.Found -or $verified.Commit -ne $Commit) {
        throw "GitHub tag $Tag verification failed after creation."
    }
}

function Get-OrCreateRelease {
    param(
        [Parameter(Mandatory = $true)][string]$Tag,
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $encodedTag = [System.Uri]::EscapeDataString($Tag)
    $existing = Invoke-GitHubJson -Method GET -Uri "$script:ApiBase/releases/tags/$encodedTag" -Body $null
    $body = @{ tag_name = $Tag; name = $Name; body = $Description; draft = $true; prerelease = $false }
    if ($existing.Ok) {
        if (-not $existing.Data.draft) {
            if ($existing.Data.name -ne $Name -or $existing.Data.body.Trim() -ne $Description.Trim()) {
                throw "Published GitHub release $Tag metadata differs from the release candidate."
            }
            return $existing.Data
        }
        $updated = Invoke-GitHubJson -Method PATCH -Uri "$script:ApiBase/releases/$($existing.Data.id)" -Body $body
        if (-not $updated.Ok) { throw "Failed to update GitHub release $Tag." }
        return $updated.Data
    }
    if ($existing.Status -ne 404) { throw "Failed to inspect GitHub release $Tag." }
    $created = Invoke-GitHubJson -Method POST -Uri "$script:ApiBase/releases" -Body $body
    if (-not $created.Ok) { throw "Failed to create GitHub release $Tag." }
    return $created.Data
}

function Confirm-ExistingAsset {
    param(
        [Parameter(Mandatory = $true)]$Asset,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $scratch = Join-Path (Get-RequiredEnv "CI_PROJECT_DIR") ".tmp\github-release\$env:CI_JOB_ID"
    [System.IO.Directory]::CreateDirectory($scratch) | Out-Null
    $download = Join-Path $scratch $Asset.name
    try {
        $downloadHeaders = @{}
        foreach ($entry in $script:Headers.GetEnumerator()) {
            $downloadHeaders[$entry.Key] = $entry.Value
        }
        $downloadHeaders["Accept"] = "application/octet-stream"
        Invoke-WebRequest -Uri $Asset.url -Headers $downloadHeaders `
            -OutFile $download -UseBasicParsing
        $local = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
        $remote = (Get-FileHash -LiteralPath $download -Algorithm SHA256).Hash
        if ($local -ne $remote) {
            throw "GitHub release asset $($Asset.name) exists with different bytes."
        }
    } finally {
        if (Test-Path -LiteralPath $scratch) {
            Remove-Item -LiteralPath $scratch -Recurse -Force
        }
    }
}

function Publish-Asset {
    param(
        [Parameter(Mandatory = $true)]$Release,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $name = [System.IO.Path]::GetFileName($Path)
    $existing = @($Release.assets) | Where-Object { $_.name -eq $name }
    if ($existing.Count -gt 1) { throw "GitHub release has duplicate asset $name." }
    if ($existing.Count -eq 1) {
        Confirm-ExistingAsset -Asset $existing[0] -Path $Path
        return
    }
    if (-not $Release.draft) {
        throw "Published GitHub release is missing expected asset $name."
    }
    $uploadBase = $Release.upload_url -replace '\{\?name,label\}$', ''
    $uri = "$uploadBase?name=$([System.Uri]::EscapeDataString($name))"
    Invoke-RestMethod -Method POST -Uri $uri -Headers $script:Headers -InFile $Path `
        -ContentType "application/octet-stream" -ErrorAction Stop | Out-Null
}

function Confirm-ReleaseAssets {
    param(
        [Parameter(Mandatory = $true)]$Release,
        [Parameter(Mandatory = $true)][array]$Assets
    )

    $expectedNames = @($Assets | ForEach-Object { $_.Name } | Sort-Object)
    $actualNames = @($Release.assets | ForEach-Object { $_.name } | Sort-Object)
    if (($expectedNames -join "`n") -ne ($actualNames -join "`n")) {
        throw "GitHub release asset names do not exactly match the release evidence."
    }
    foreach ($asset in $Assets) {
        $remote = @($Release.assets) | Where-Object { $_.name -eq $asset.Name }
        if ($remote.Count -ne 1) {
            throw "GitHub release asset $($asset.Name) is missing or duplicated."
        }
        Confirm-ExistingAsset -Asset $remote[0] -Path $asset.FullName
    }
}

$token = Get-RequiredEnv "GITHUB_RELEASE_TOKEN"
$projectDir = Get-RequiredEnv "CI_PROJECT_DIR"
$tag = Get-RequiredEnv "CI_COMMIT_TAG"
$commit = Get-RequiredEnv "CI_COMMIT_SHA"
if ($tag -notmatch '^v(?<version>(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*))$') {
    throw "CI_COMMIT_TAG must exactly match vX.Y.Z; got '$tag'."
}
$version = $Matches.version
$script:ApiBase = "https://api.github.com/repos/$repository"
$script:Headers = @{
    Authorization = "Bearer $token"
    Accept = "application/vnd.github+json"
    "X-GitHub-Api-Version" = $apiVersion
    "User-Agent" = "rappct-gitlab-release"
}
$packageDir = Join-Path $projectDir "target\package"
$expectedNames = @(
    "rappct-$version.crate"
    "rappct-$version.crate.sha256"
    "rappct-$version.crate.published.sha256"
    "cargo-metadata.json"
    "rappct.cdx.json"
)
$matchingFiles = @(
    Get-ChildItem -LiteralPath $packageDir -File |
        Where-Object { $_.Name -match '\.(crate|sha256|json)$' }
)
$actualNames = @($matchingFiles | ForEach-Object { $_.Name } | Sort-Object)
if ((($expectedNames | Sort-Object) -join "`n") -ne ($actualNames -join "`n")) {
    throw "Release evidence in $packageDir does not exactly match the expected files."
}
$assets = @($expectedNames | ForEach-Object { Get-Item -LiteralPath (Join-Path $packageDir $_) })

Assert-GitLabMainCommit -Commit $commit
Push-GitHubMain -Token $token
Confirm-CommitAndTag -Commit $commit -Tag $tag
$description = Get-ChangelogEntry -Path (Join-Path $projectDir "CHANGELOG.md") -Version $version
$release = Get-OrCreateRelease -Tag $tag -Name "rappct $tag" -Description $description
foreach ($asset in $assets) {
    Publish-Asset -Release $release -Path $asset.FullName
}
$verified = Invoke-GitHubJson -Method GET `
    -Uri "$script:ApiBase/releases/tags/$([System.Uri]::EscapeDataString($tag))" -Body $null
if (-not $verified.Ok) {
    throw "GitHub release $tag verification failed."
}
Confirm-ReleaseAssets -Release $verified.Data -Assets $assets
if ($verified.Data.draft) {
    $published = Invoke-GitHubJson -Method PATCH -Uri "$script:ApiBase/releases/$($verified.Data.id)" -Body @{
        draft = $false
    }
    if (-not $published.Ok -or $published.Data.draft) {
        throw "GitHub release $tag could not be published after asset verification."
    }
}
Write-Host "Verified GitHub release $tag at commit $commit with $($assets.Count) assets."
