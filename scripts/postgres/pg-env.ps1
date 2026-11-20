<#
.SYNOPSIS
    Configures the current PowerShell environment with PostgreSQL variables.
.EXAMPLE
    . .\scripts\postgres\pg-env.ps1
#>

param(
    [string]$InstallDir = "$env:USERPROFILE\.mini-dbs\postgres"
)

if (-not (Test-Path "$InstallDir\bin")) {
    Write-Host "[!] PostgreSQL binaries not found at $InstallDir\bin." -ForegroundColor Red
    return
}

$env:PATH = "$InstallDir\bin;$env:PATH"
$env:PGDATA = "$InstallDir\data"
$env:PGCLIENTENCODING = "utf8"

if (Test-Path "$InstallDir\mini-dbs.json") {
    $cfg = Get-Content "$InstallDir\mini-dbs.json" | ConvertFrom-Json
    $env:PGPORT = "$($cfg.port)"
    $env:PGUSER = "$($cfg.username)"
}

Write-Host "[✓] Current session environment updated:" -ForegroundColor Green
Write-Host "    PATH:      $InstallDir\bin (prepended)" -ForegroundColor Gray
Write-Host "    PGDATA:    $env:PGDATA" -ForegroundColor Gray
Write-Host "    PGPORT:    $env:PGPORT" -ForegroundColor Gray
Write-Host "    PGUSER:    $env:PGUSER" -ForegroundColor Gray
Write-Host "You can now run 'psql', 'pg_ctl', 'createdb', etc. directly in this window." -ForegroundColor Cyan
