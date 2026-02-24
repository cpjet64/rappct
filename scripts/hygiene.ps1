$ErrorActionPreference = 'Stop'

Write-Host "=== Repo Hygiene Check ==="

$errors = 0
$trackedFiles = git ls-files

Write-Host -NoNewline "Large files (>10MB): "
$largeFiles = @()
foreach ($file in $trackedFiles) {
    if (-not (Test-Path $file)) {
        continue
    }

    $item = Get-Item $file
    if ($item.Length -gt 10MB) {
        $mb = [math]::Round($item.Length / 1MB, 2)
        $largeFiles += "  $file ($mb MB)"
    }
}

if ($largeFiles.Count -eq 0) {
    Write-Host "PASS"
} else {
    Write-Host "FAIL"
    $largeFiles | ForEach-Object { Write-Host $_ }
    $errors++
}

Write-Host -NoNewline "Merge conflict markers: "
$conflictFound = $false
foreach ($file in $trackedFiles) {
    if (-not (Test-Path $file)) { continue }
    $ext = [System.IO.Path]::GetExtension($file).ToLowerInvariant()
    if ($ext -in @('.png', '.jpg', '.jpeg', '.gif', '.pdf', '.zip', '.jar', '.ico', '.bin')) {
        continue
    }
    $content = Get-Content -Path $file -Raw -ErrorAction SilentlyContinue
    if ($null -eq $content) { continue }
    if ($content -match '(?m)^(<<<<<<<|>>>>>>>|=======$)') {
        $conflictFound = $true
        break
    }
}

if (-not $conflictFound) {
    Write-Host "PASS"
} else {
    Write-Host "FAIL — conflict markers found"
    $errors++
}

Write-Host -NoNewline "Required files: "
$missingFiles = @()
foreach ($file in @('.gitignore')) {
    if (-not (Test-Path $file)) {
        $missingFiles += $file
    }
}

if ($missingFiles.Count -eq 0) {
    Write-Host "PASS"
} else {
    Write-Host "FAIL — missing:$($missingFiles -join ' ')"
    $errors++
}

if ($errors -gt 0) {
    Write-Host "=== $errors hygiene issue(s) found ==="
    exit 1
}

Write-Host "=== All hygiene checks passed ==="
