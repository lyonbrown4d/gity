[CmdletBinding()]
param(
    [int]$Port = 18080,
    [string]$ProjectName
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$compose = Join-Path $root "docker-compose.blackbox.yaml"
$base = "http://127.0.0.1:$Port"
if ([string]::IsNullOrWhiteSpace($ProjectName)) {
    $suffix = [Guid]::NewGuid().ToString("N").Substring(0, 10)
    $ProjectName = "gity-blackbox-$suffix"
}

$createdByThisRun = $false

function Invoke-Check {
    param([string]$Path)
    $response = Invoke-WebRequest -UseBasicParsing "$base$Path"
    if ($response.StatusCode -ne 200) {
        throw "GET $Path returned HTTP $($response.StatusCode)."
    }
    return $response
}

Push-Location $root
try {
    $resources = @()
    $resources += docker ps -aq --filter "label=com.docker.compose.project=$ProjectName"
    if ($LASTEXITCODE -ne 0) { throw "Could not inspect Docker containers for project '$ProjectName'." }
    $resources += docker volume ls -q --filter "label=com.docker.compose.project=$ProjectName"
    if ($LASTEXITCODE -ne 0) { throw "Could not inspect Docker volumes for project '$ProjectName'." }
    $resources += docker network ls -q --filter "label=com.docker.compose.project=$ProjectName"
    if ($LASTEXITCODE -ne 0) { throw "Could not inspect Docker networks for project '$ProjectName'." }
    if ($resources.Count -gt 0) {
        throw "Docker Compose project '$ProjectName' already has resources. Choose a new project name; existing data will be left untouched."
    }

    $env:GITY_BLACKBOX_WEB_PORT = "$Port"
    docker compose -p $ProjectName -f $compose up -d --build
    if ($LASTEXITCODE -ne 0) {
        docker compose -p $ProjectName -f $compose logs --no-color
        throw "Docker Compose failed to start the Rust blackbox service; inspect '$ProjectName' before removing any partial resources."
    }
    $createdByThisRun = $true

    $ready = $false
    for ($attempt = 0; $attempt -lt 60; $attempt++) {
        try {
            Invoke-Check "/api/health" | Out-Null
            $ready = $true
            break
        } catch {
            if ($attempt -eq 59) {
                docker compose -p $ProjectName -f $compose logs --no-color
                throw "Rust service did not become healthy: $_"
            }
            Start-Sleep -Seconds 2
        }
    }
    if (-not $ready) { throw "Rust service did not become healthy." }

    foreach ($path in @("/openapi.json", "/docs", "/docs/swagger-ui-bundle.js")) {
        $status = 0
        try { $response = Invoke-WebRequest -UseBasicParsing "$base$path"; $status = [int]$response.StatusCode }
        catch { if ($_.Exception.Response) { $status = [int]$_.Exception.Response.StatusCode } else { throw } }
        if ($status -ne 404) { throw "Release documentation endpoint is exposed: $path (HTTP $status)" }
    }
    $home = Invoke-Check "/"
    if ($home.Content -notmatch "<html") {
        throw "The Rust/Trunk Web root did not return an HTML document."
    }
    Write-Host "Blackbox smoke passed: health, disabled documentation, and Rust/Trunk Web root."
} finally {
    Pop-Location
    if ($createdByThisRun) {
        docker compose -p $ProjectName -f $compose down --volumes
    }
}
