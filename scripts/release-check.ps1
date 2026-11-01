[CmdletBinding()]
param(
    [switch]$SkipClippy,
    [switch]$SkipTests,
    [switch]$SkipReleaseBuild
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

function Invoke-Native {
    param(
        [Parameter(Mandatory = $true)][string]$Command,
        [string[]]$Arguments = @()
    )
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "'$Command $($Arguments -join ' ')' failed with exit code $LASTEXITCODE."
    }
}

Invoke-Native "cargo" @("xtask", "source-check")
Invoke-Native "cargo" @("xtask", "vendor-check")
Invoke-Native "cargo" @("fmt", "--all", "--", "--check")
if (-not $SkipClippy) {
    Invoke-Native "cargo" @("clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings")
}
if (-not $SkipTests) {
    Invoke-Native "cargo" @("test", "--workspace", "--locked")
}
Invoke-Native "cargo" @("check", "-p", "gity-web", "--target", "wasm32-unknown-unknown", "--locked")

if (-not $SkipReleaseBuild) {
    if (-not (Get-Command "trunk" -ErrorAction SilentlyContinue)) {
        throw "Trunk 0.21.14 is required. Install it with: cargo install trunk --version 0.21.14 --locked"
    }
    $trunkVersion = (& trunk --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $trunkVersion -notmatch '\b0\.21\.14\b') {
        throw "Expected Trunk 0.21.14, found '$trunkVersion'. Install the pinned version with: cargo install trunk --version 0.21.14 --locked"
    }
    Invoke-Native "cargo" @("xtask", "release")
}

Write-Host "Rust release checks completed. Run .\scripts\blackbox-e2e.ps1 for the Docker smoke check."
