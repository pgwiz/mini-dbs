[CmdletBinding()]
param(
    [string]$InstallDir = "$env:USERPROFILE\.mini-dbs\postgres",
    [ValidateSet("fast", "smart", "immediate")]
    [string]$Mode = "fast"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path "$InstallDir\bin\pg_ctl.exe")) {
    Write-Host "[!] PostgreSQL not found at $InstallDir." -ForegroundColor Red
    exit 1
}

$dataDir = "$InstallDir\data"

Write-Host "Stopping PostgreSQL server (mode: $Mode)..." -ForegroundColor Cyan
& "$InstallDir\bin\pg_ctl.exe" -D $dataDir -m $Mode stop

if ($LASTEXITCODE -eq 0) {
    Write-Host "[✓] PostgreSQL server stopped." -ForegroundColor Green
} else {
    Write-Host "[!] Failed to stop PostgreSQL or server was not running." -ForegroundColor Yellow
}
