[CmdletBinding()]
param(
    [string]$InstallDir = "$env:USERPROFILE\.mini-dbs\postgres"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path "$InstallDir\bin\pg_ctl.exe")) {
    Write-Host "[!] PostgreSQL not found at $InstallDir. Run install.ps1 first." -ForegroundColor Red
    exit 1
}

$dataDir = "$InstallDir\data"
$logFile = "$dataDir\logfile"

Write-Host "Starting PostgreSQL server..." -ForegroundColor Cyan
& "$InstallDir\bin\pg_ctl.exe" -D $dataDir -l $logFile start

if ($LASTEXITCODE -eq 0) {
    Write-Host "[✓] PostgreSQL started successfully." -ForegroundColor Green
    if (Test-Path "$InstallDir\mini-dbs.json") {
        $cfg = Get-Content "$InstallDir\mini-dbs.json" | ConvertFrom-Json
        Write-Host "    Port: $($cfg.port)" -ForegroundColor Gray
        Write-Host "    Data: $dataDir" -ForegroundColor Gray
    }
} else {
    Write-Host "[!] Failed to start PostgreSQL. Check log: $logFile" -ForegroundColor Red
}
