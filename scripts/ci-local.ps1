$ErrorActionPreference = 'Stop'

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$minimumFreeBytes = 5GB
$features = @('', 'introspection', 'net', 'introspection,net')
$msrvList = @('1.88.0', '1.89.0', '1.90.0', '1.91.0', '1.92.0', '1.93.0', '1.94.0', '1.95.0')

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,

        [Parameter(Mandatory = $true)]
        [scriptblock]$Action
    )

    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "[ci-local] $Label failed with exit code $LASTEXITCODE"
    }
}

function Invoke-CargoChecked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,

        [Parameter(Mandatory = $true)]
        [ValidateNotNullOrEmpty()]
        [string]$Toolchain,

        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    $cargoArgs = @()
    if ($Toolchain -ne 'stable') {
        $cargoArgs += "+$Toolchain"
    }
    $cargoArgs += $Arguments

    cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw "[ci-local] $Label failed with exit code $LASTEXITCODE"
    }
}

function Assert-WindowsHost {
    if ([Environment]::OSVersion.Platform -ne [System.PlatformID]::Win32NT) {
        Write-Error '[ci-local] Windows-only checks. Detected non-Windows environment. Aborting.'
        exit 1
    }
}

function Assert-FreeSpace {
    $driveRoot = [System.IO.Path]::GetPathRoot($repoRoot)
    $driveInfo = [System.IO.DriveInfo]::new($driveRoot)
    if (-not $driveInfo.IsReady) {
        throw "[ci-local] Repository volume is not ready: $driveRoot"
    }
    if ($driveInfo.AvailableFreeSpace -ge $minimumFreeBytes) {
        return
    }

    throw (
        "[ci-local] Insufficient free space on {0}: {1:N0} bytes available; " +
        "{2:N0} bytes required. Free space before running local CI."
    ) -f $driveRoot, $driveInfo.AvailableFreeSpace, $minimumFreeBytes
}

function Get-TestFeatures {
    param([Parameter(Mandatory = $true)][AllowEmptyString()][string]$Feature)

    if ($Feature -eq '') {
        return '_test_helpers'
    }
    return "$Feature,_test_helpers"
}

function Get-FeatureLabel {
    param([Parameter(Mandatory = $true)][AllowEmptyString()][string]$Feature)

    if ($Feature -eq '') {
        return 'no product features'
    }
    return "features: $Feature"
}

function Test-RustToolchainInstalled {
    param([Parameter(Mandatory = $true)][string]$Toolchain)

    $escapedToolchain = [regex]::Escape($Toolchain)
    return [bool](rustup toolchain list --verbose | Select-String "^$escapedToolchain(-|\s)")
}

function Invoke-FormatAndStaticChecks {
    Write-Host '[ci-local] hygiene'
    Invoke-Checked -Label 'hygiene' -Action { & (Join-Path $PSScriptRoot 'hygiene.ps1') }

    Write-Host '[ci-local] fmt (stable, workspace)'
    # rustfmt and stable clippy must be pre-installed. Do not mutate shared RUSTUP_HOME during builds.
    Invoke-Checked -Label 'fmt (stable, workspace)' -Action { cargo fmt --all -- --check }

    Write-Host '[ci-local] code size'
    Invoke-Checked -Label 'code size' -Action { python (Join-Path $PSScriptRoot 'check_code_size.py') }
}

function Invoke-RustFeatureGate {
    param(
        [Parameter(Mandatory = $true)][string]$Toolchain,
        [Parameter(Mandatory = $true)][AllowEmptyString()][string]$Feature,
        [switch]$CheckDependencies
    )

    $featureLabel = Get-FeatureLabel -Feature $Feature
    $testFeatures = Get-TestFeatures -Feature $Feature

    Write-Host "[ci-local] test ($Toolchain, $featureLabel)"
    Invoke-CargoChecked `
        -Label "test ($Toolchain, $featureLabel)" `
        -Toolchain $Toolchain `
        -Arguments @('test', '--all-targets', '--locked', '--features', $testFeatures)

    Write-Host "[ci-local] clippy ($Toolchain, $featureLabel)"
    if ($Toolchain -eq 'stable') {
        Invoke-CargoChecked `
            -Label "clippy ($Toolchain, $featureLabel)" `
            -Toolchain $Toolchain `
            -Arguments @('clippy', '--all-targets', '--locked', '--features', $testFeatures, '--', '-D', 'warnings')
    } else {
        Write-Host "[ci-local] clippy ($Toolchain, $featureLabel) skipped; stable clippy is the lint gate"
    }

    if ($CheckDependencies) {
        $treeArgs = @('tree', '-d', '--locked')
        if ($Feature -ne '') { $treeArgs += @('--features', $Feature) }
        Invoke-CargoChecked `
            -Label "duplicate dependency check ($Toolchain, $featureLabel)" `
            -Toolchain $Toolchain `
            -Arguments $treeArgs
    }
}

function Invoke-FeatureMatrix {
    param(
        [Parameter(Mandatory = $true)][string]$Toolchain,
        [switch]$CheckDependencies
    )

    foreach ($feature in $features) {
        Invoke-RustFeatureGate `
            -Toolchain $Toolchain `
            -Feature $feature `
            -CheckDependencies:$CheckDependencies
    }
}

function Invoke-MsrvMatrix {
    foreach ($msrv in $msrvList) {
        Write-Host "[ci-local] toolchain $msrv"
        # Toolchains must be pre-installed. To provision: rustup toolchain install <version>.
        if (-not (Test-RustToolchainInstalled -Toolchain $msrv)) {
            throw "[ci-local] required MSRV toolchain $msrv is not installed. Provision it with: rustup toolchain install $msrv"
        }
        Invoke-FeatureMatrix -Toolchain $msrv
    }
}

function Invoke-OptionalToolchainMatrix {
    foreach ($toolchain in @('beta', 'nightly')) {
        Write-Host "[ci-local] $toolchain toolchain"
        if (-not (Test-RustToolchainInstalled -Toolchain $toolchain)) {
            Write-Warning "[ci-local] $toolchain toolchain not installed, skipping"
            continue
        }
        Invoke-FeatureMatrix -Toolchain $toolchain
    }
}

function Invoke-CiLocal {
    Assert-WindowsHost
    Assert-FreeSpace
    Set-Location -LiteralPath $repoRoot

    $env:RUST_BACKTRACE = '1'
    $env:RUSTFLAGS = '-D warnings'
    Invoke-FormatAndStaticChecks
    Invoke-FeatureMatrix -Toolchain 'stable' -CheckDependencies
    Invoke-MsrvMatrix
    Invoke-OptionalToolchainMatrix
    Write-Host '[ci-local] OK'
}

Invoke-CiLocal
