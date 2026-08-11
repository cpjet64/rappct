$ErrorActionPreference = "Stop"

function Get-RequiredEnv {
    param([Parameter(Mandatory = $true)][string]$Name)
    $value = [Environment]::GetEnvironmentVariable($Name)
    if ([string]::IsNullOrWhiteSpace($value)) {
        throw "Required environment variable '$Name' is not set."
    }
    return $value
}

function Get-ErrorBody {
    param([System.Net.WebException]$Exception)
    if (-not $Exception.Response) { return $Exception.Message }
    $stream = $Exception.Response.GetResponseStream()
    if (-not $stream) { return $Exception.Message }
    $reader = [System.IO.StreamReader]::new($stream)
    try { return $reader.ReadToEnd() } finally { $reader.Dispose(); $stream.Dispose() }
}

function Invoke-GitLabJson {
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
        $params.Body = $Body | ConvertTo-Json -Depth 10 -Compress
    }

    try {
        return @{ Ok = $true; Status = 200; Data = Invoke-RestMethod @params }
    } catch [System.Net.WebException] {
        $statusCode = if ($_.Exception.Response) { [int]$_.Exception.Response.StatusCode } else { 0 }
        return @{ Ok = $false; Status = $statusCode; Error = Get-ErrorBody $_.Exception }
    }
}

function Assert-PublishedFileMatches {
    param(
        [Parameter(Mandatory = $true)][string]$Uri,
        [Parameter(Mandatory = $true)][string]$Path
    )
    $projectDir = Get-RequiredEnv "CI_PROJECT_DIR"
    $jobScope = Get-RequiredEnv "CI_JOB_ID"
    $scratch = Join-Path $projectDir ".tmp\release-upload-verify\$jobScope"
    $download = Join-Path $scratch ([System.IO.Path]::GetFileName($Path))
    [System.IO.Directory]::CreateDirectory($scratch) | Out-Null
    try {
        Invoke-WebRequest -Uri $Uri -Headers $script:Headers -OutFile $download -UseBasicParsing
        $localHash = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
        $remoteHash = (Get-FileHash -LiteralPath $download -Algorithm SHA256).Hash
        if ($localHash -ne $remoteHash) {
            throw "Existing package file at $Uri does not match the release artifact."
        }
        Write-Host "Existing package file matches release artifact at $Uri"
    } finally {
        if (Test-Path -LiteralPath $scratch) {
            Remove-Item -LiteralPath $scratch -Recurse -Force
        }
    }
}

function Publish-PackageFile {
    param(
        [Parameter(Mandatory = $true)][string]$Uri,
        [Parameter(Mandatory = $true)][string]$Path
    )

    try {
        Invoke-RestMethod -Method Put -Uri $Uri -Headers $script:Headers -InFile $Path -ContentType "application/octet-stream" -ErrorAction Stop | Out-Null
        Write-Host "Uploaded package file $Uri"
    } catch [System.Net.WebException] {
        $statusCode = if ($_.Exception.Response) { [int]$_.Exception.Response.StatusCode } else { 0 }
        $body = Get-ErrorBody $_.Exception
        if (($statusCode -eq 400 -or $statusCode -eq 409) -and $body -match "already|exists|taken") {
            Assert-PublishedFileMatches -Uri $Uri -Path $Path
            return
        }
        throw "Failed to upload package file $Uri. HTTP $($statusCode): $body"
    }
}

function Get-ChangelogEntry {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Version
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "CHANGELOG.md is missing: $Path"
    }

    $content = Get-Content -LiteralPath $Path -Raw
    $escapedVersion = [regex]::Escape($Version)
    $pattern = "(?ms)^##\s+(?:\[$escapedVersion\](?:\([^)]+\))?|$escapedVersion)(?:\s+[^\r\n]*)?\r?\n.*?(?=^##\s+|\z)"
    $match = [regex]::Match($content, $pattern)
    if (-not $match.Success) {
        throw "CHANGELOG.md does not contain a release entry for version $Version."
    }

    $entry = $match.Value.Trim()
    if ([string]::IsNullOrWhiteSpace($entry)) {
        throw "CHANGELOG.md release entry for version $Version is empty."
    }
    return $entry
}

function Get-ReleaseContext {
    $jobToken = Get-RequiredEnv "CI_JOB_TOKEN"
    $projectId = Get-RequiredEnv "CI_PROJECT_ID"
    $apiBaseUrl = Get-RequiredEnv "CI_API_V4_URL"
    $projectDir = Get-RequiredEnv "CI_PROJECT_DIR"
    $tagName = Get-RequiredEnv "CI_COMMIT_TAG"
    $script:Headers = @{ "JOB-TOKEN" = $jobToken }
    if ($tagName -notmatch '^v(?<version>(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*))$') {
        throw "CI_COMMIT_TAG must exactly match vX.Y.Z; got '$tagName'."
    }
    $version = $Matches.version
    $crateFileName = "rappct-$version.crate"
    $packageDir = Join-Path $projectDir "target\package"
    $files = @{
        Crate = Join-Path $packageDir $crateFileName
        Checksum = Join-Path $packageDir "$crateFileName.sha256"
        Published = Join-Path $packageDir "$crateFileName.published.sha256"
    }
    foreach ($evidencePath in $files.Values) {
        if (-not (Test-Path -LiteralPath $evidencePath -PathType Leaf)) {
            throw "Crate release artifact is missing: $evidencePath"
        }
    }
    return @{
        ApiBaseUrl = $apiBaseUrl
        ProjectId = $projectId
        ProjectDir = $projectDir
        TagName = $tagName
        Version = $version
        CrateFileName = $crateFileName
        Files = $files
    }
}

function Publish-ReleaseArtifacts {
    param(
        [Parameter(Mandatory = $true)][hashtable]$Context
    )
    $packageBase = "$($Context.ApiBaseUrl)/projects/$($Context.ProjectId)/packages/generic/rappct/$($Context.Version)"
    $urls = @{
        Crate = "$packageBase/$([System.Uri]::EscapeDataString($Context.CrateFileName))"
        Checksum = "$packageBase/$([System.IO.Path]::GetFileName($Context.Files.Checksum))"
        Published = "$packageBase/$([System.IO.Path]::GetFileName($Context.Files.Published))"
    }
    Publish-PackageFile -Uri $urls.Crate -Path $Context.Files.Crate
    Publish-PackageFile -Uri $urls.Checksum -Path $Context.Files.Checksum
    Publish-PackageFile -Uri $urls.Published -Path $Context.Files.Published
    return $urls
}

$context = Get-ReleaseContext
$apiBaseUrl = $context.ApiBaseUrl
$projectId = $context.ProjectId
$projectDir = $context.ProjectDir
$tagName = $context.TagName
$version = $context.Version
$crateFileName = $context.CrateFileName
$cratePath = $context.Files.Crate
$checksumPath = $context.Files.Checksum
$publishedPath = $context.Files.Published
$checksumFileName = [System.IO.Path]::GetFileName($checksumPath)
$publishedFileName = [System.IO.Path]::GetFileName($publishedPath)
$artifactUrls = Publish-ReleaseArtifacts -Context $context
$cratePackageUrl = $artifactUrls.Crate
$checksumPackageUrl = $artifactUrls.Checksum
$publishedPackageUrl = $artifactUrls.Published

$encodedTag = [System.Uri]::EscapeDataString($tagName)
$releaseName = "rappct $tagName"
$releaseDescription = Get-ChangelogEntry -Path (Join-Path $projectDir "CHANGELOG.md") -Version $version
$releaseApi = "$apiBaseUrl/projects/$projectId/releases/$encodedTag"
$releasesApi = "$apiBaseUrl/projects/$projectId/releases"
$releaseLinks = @(
    @{
        name = "rappct crate package"
        url = $cratePackageUrl
        direct_asset_path = "/crate/$crateFileName"
        link_type = "package"
    },
    @{
        name = "crates.io"
        url = "https://crates.io/crates/rappct/$version"
        link_type = "other"
    },
    @{
        name = "crate SHA-256"
        url = $checksumPackageUrl
        direct_asset_path = "/crate/$checksumFileName"
        link_type = "other"
    },
    @{
        name = "crates.io byte verification"
        url = $publishedPackageUrl
        direct_asset_path = "/crate/$publishedFileName"
        link_type = "other"
    }
)

$existing = Invoke-GitLabJson -Method "GET" -Uri $releaseApi -Body $null
if ($existing.Ok) {
    $updated = Invoke-GitLabJson -Method "PUT" -Uri $releaseApi -Body @{
        name = $releaseName
        description = $releaseDescription
        assets = @{ links = $releaseLinks }
    }
    if (-not $updated.Ok) {
        throw "Failed to update GitLab release $tagName. HTTP $($updated.Status): $($updated.Error)"
    }
    Write-Host "Updated GitLab release $tagName"
} elseif ($existing.Status -eq 404) {
    $created = Invoke-GitLabJson -Method "POST" -Uri $releasesApi -Body @{
        name = $releaseName
        tag_name = $tagName
        description = $releaseDescription
        assets = @{ links = $releaseLinks }
    }
    if (-not $created.Ok) {
        throw "Failed to create GitLab release $tagName. HTTP $($created.Status): $($created.Error)"
    }
    Write-Host "Created GitLab release $tagName"
} else {
    throw "Failed to query GitLab release $tagName. HTTP $($existing.Status): $($existing.Error)"
}
