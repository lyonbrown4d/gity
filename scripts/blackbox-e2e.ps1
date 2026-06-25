param(
    [int]$Port = 18080,
    [string]$ProjectName = "gity-blackbox"
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$compose = Join-Path $root "docker-compose.blackbox.yaml"
$tmpRoot = Join-Path $root ".tmp-e2e\blackbox"
$binRoot = Join-Path $tmpRoot "bin"
$runnerBin = Join-Path $binRoot "gity-runner-e2e.exe"

New-Item -ItemType Directory -Force -Path $binRoot | Out-Null

try {
    Push-Location $root

    go build -o $runnerBin .\cmd\runner

    $env:GITY_BLACKBOX_WEB_PORT = "$Port"
    docker compose -p $ProjectName -f $compose up -d --build

    $base = "http://127.0.0.1:$Port"
    for ($i = 0; $i -lt 60; $i++) {
        try {
            Invoke-WebRequest -UseBasicParsing "$base/api/health" | Out-Null
            Invoke-WebRequest -UseBasicParsing "$base/" | Out-Null
            break
        } catch {
            if ($i -eq 59) {
                docker compose -p $ProjectName -f $compose logs
                throw
            }
            Start-Sleep -Seconds 2
        }
    }

    $env:E2E_WEB_BASE_URL = $base
    $env:E2E_API_BASE_URL = "$base/api/v1"
    $env:GITY_E2E_ROOT = $tmpRoot
    $env:GITY_E2E_RUNNER_BIN = $runnerBin
    $env:GITY_E2E_REPO_ROOT = ""

    pnpm -C web test:e2e:blackbox
} finally {
    Pop-Location
    docker compose -p $ProjectName -f $compose down -v
}
