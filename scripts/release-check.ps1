[CmdletBinding()]
param(
    [switch]$SkipGoLint,
    [switch]$SkipGoReleaser,
    [switch]$SkipFrontendInstall,
    [switch]$SkipBetaSmoke
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,

        [Parameter(Mandatory = $true)]
        [scriptblock]$Body
    )

    Write-Host "==> $Name"
    & $Body
}

function Invoke-Native {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Command,

        [string[]]$Arguments = @()
    )

    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "'$Command $($Arguments -join ' ')' failed with exit code $LASTEXITCODE."
    }
}

function Require-Command {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command '$Name' was not found on PATH."
    }
}

Invoke-Step "Go test" {
    Invoke-Native "go" @("test", "./...")
}

if ($SkipGoLint) {
    Write-Host "==> Skipping Go lint"
} else {
    Require-Command golangci-lint

    Invoke-Step "Go lint" {
        Invoke-Native "golangci-lint" @("run")
    }
}

if (-not $SkipFrontendInstall) {
    Invoke-Step "Web dependencies" {
        Invoke-Native "pnpm" @("-C", "web", "install", "--frozen-lockfile")
    }
}

Invoke-Step "Web build" {
    Invoke-Native "pnpm" @("-C", "web", "build")
}

if ($SkipGoReleaser) {
    Write-Host "==> Skipping GoReleaser checks"
    exit 0
}

Require-Command goreleaser
Require-Command upx

Invoke-Step "GoReleaser config" {
    Invoke-Native "goreleaser" @("check")
}

Invoke-Step "GoReleaser snapshot packages" {
    Invoke-Native "goreleaser" @("release", "--snapshot", "--clean", "--skip=publish,docker")
}

if ($SkipBetaSmoke) {
    Write-Host "==> Skipping beta smoke checklist (manual run required)"
    Write-Host "Use .\docs\beta-smoke.md to complete the end-to-end smoke after release checks."
    exit 0
}

Write-Host "==> Beta smoke checklist"
Write-Host "Release checks succeeded. Complete the remaining end-to-end verification from:"
Write-Host ".\docs\beta-smoke.md"
Write-Host "Before tagging, confirm all smoke steps are completed."
