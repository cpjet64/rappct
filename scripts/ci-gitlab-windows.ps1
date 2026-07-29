[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet(
        'stable',
        '1.88.0',
        '1.89.0',
        '1.90.0',
        '1.91.0',
        '1.92.0',
        '1.93.0',
        '1.94.0',
        '1.95.0',
        'beta',
        'nightly'
    )]
    [string]$RustToolchain,

    [Parameter(Mandatory = $true)]
    [ValidateSet('none', 'introspection', 'net', 'introspection,net')]
    [string]$FeatureSet
)

$ErrorActionPreference = 'Stop'
$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$minimumFreeBytes = 5GB

function Assert-FreeSpace {
    $driveRoot = [System.IO.Path]::GetPathRoot($repoRoot)
    $driveInfo = [System.IO.DriveInfo]::new($driveRoot)
    if (-not $driveInfo.IsReady) {
        throw "[ci-gitlab-windows] Repository volume is not ready: $driveRoot"
    }
    if ($driveInfo.AvailableFreeSpace -ge $minimumFreeBytes) {
        return
    }

    throw (
        "[ci-gitlab-windows] Insufficient free space on {0}: {1:N0} bytes available; " +
        "{2:N0} bytes required. Free space before running CI."
    ) -f $driveRoot, $driveInfo.AvailableFreeSpace, $minimumFreeBytes
}

function Disable-InheritedRustcWrapper {
    if (-not $env:RUSTC_WRAPPER) {
        return
    }

    Write-Host "[ci-gitlab-windows] disabling inherited RUSTC_WRAPPER to avoid runner sccache configuration drift"
    Remove-Item Env:RUSTC_WRAPPER -ErrorAction SilentlyContinue
}

function Invoke-NativeChecked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,

        [Parameter(Mandatory = $true)]
        [scriptblock]$Action
    )

    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "[ci-gitlab-windows] $Label failed with exit code $LASTEXITCODE"
    }
}

function Get-TestFeatures {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FeatureSet
    )

    if ($FeatureSet -eq 'none') {
        return '_test_helpers'
    }
    return "$FeatureSet,_test_helpers"
}

function Invoke-ToolchainVersionChecks {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RustToolchain,

        [Parameter(Mandatory = $true)]
        [string]$ToolchainArgument
    )

    Invoke-NativeChecked -Label "rustc $RustToolchain version" -Action {
        rustc $ToolchainArgument -Vv
    }
    Invoke-NativeChecked -Label "cargo $RustToolchain version" -Action {
        cargo $ToolchainArgument -V
    }
}

function Invoke-StableNoFeatureChecks {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ToolchainArgument
    )

    Invoke-NativeChecked -Label 'format check' -Action {
        cargo $ToolchainArgument fmt --all -- --check
    }
    Invoke-NativeChecked -Label 'code size check' -Action {
        python (Join-Path $PSScriptRoot 'check_code_size.py')
    }
    Invoke-NativeChecked -Label 'hygiene' -Action {
        & (Join-Path $PSScriptRoot 'hygiene.ps1')
    }
}

function Invoke-FeatureTests {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RustToolchain,

        [Parameter(Mandatory = $true)]
        [string]$ToolchainArgument,

        [Parameter(Mandatory = $true)]
        [string]$FeatureSet,

        [Parameter(Mandatory = $true)]
        [string]$TestFeatures
    )

    Invoke-NativeChecked -Label "tests ($RustToolchain, $FeatureSet)" -Action {
        cargo $ToolchainArgument test `
            --all-targets `
            --locked `
            --features $TestFeatures
    }
}

function Invoke-DependencyTree {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ToolchainArgument,

        [Parameter(Mandatory = $true)]
        [string]$FeatureSet
    )

    $treeArguments = @($toolchainArgument, 'tree', '-d', '--locked')
    if ($FeatureSet -ne 'none') {
        $treeArguments += @('--features', $FeatureSet)
    }

    Invoke-NativeChecked -Label "dependency tree ($FeatureSet)" -Action {
        cargo @treeArguments
    }
}

function Invoke-Clippy {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RustToolchain,

        [Parameter(Mandatory = $true)]
        [string]$ToolchainArgument,

        [Parameter(Mandatory = $true)]
        [string]$FeatureSet,

        [Parameter(Mandatory = $true)]
        [string]$TestFeatures
    )

    Invoke-NativeChecked -Label "clippy ($RustToolchain, $FeatureSet)" -Action {
        cargo $ToolchainArgument clippy `
            --all-targets `
            --locked `
            --features $TestFeatures `
            -- `
            -D warnings
    }
}

function Invoke-GitLabWindowsCi {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RustToolchain,

        [Parameter(Mandatory = $true)]
        [string]$FeatureSet
    )

    Assert-FreeSpace
    Set-Location -LiteralPath $repoRoot
    $env:RUST_BACKTRACE = '1'
    $env:RUSTFLAGS = '-D warnings'
    Disable-InheritedRustcWrapper
    $toolchainArgument = "+$RustToolchain"
    $testFeatures = Get-TestFeatures -FeatureSet $FeatureSet

    Invoke-ToolchainVersionChecks `
        -RustToolchain $RustToolchain `
        -ToolchainArgument $toolchainArgument

    if ($RustToolchain -eq 'stable' -and $FeatureSet -eq 'none') {
        Invoke-StableNoFeatureChecks -ToolchainArgument $toolchainArgument
    }

    Invoke-FeatureTests `
        -RustToolchain $RustToolchain `
        -ToolchainArgument $toolchainArgument `
        -FeatureSet $FeatureSet `
        -TestFeatures $testFeatures

    if ($RustToolchain -eq 'stable') {
        Invoke-DependencyTree -ToolchainArgument $toolchainArgument -FeatureSet $FeatureSet
    }

    if ($RustToolchain -eq 'stable') {
        Invoke-Clippy `
            -RustToolchain $RustToolchain `
            -ToolchainArgument $toolchainArgument `
            -FeatureSet $FeatureSet `
            -TestFeatures $testFeatures
    }
}

Invoke-GitLabWindowsCi -RustToolchain $RustToolchain -FeatureSet $FeatureSet
