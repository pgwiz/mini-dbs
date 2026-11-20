[CmdletBinding()]
param(
    [string]$InstallDir = "$env:USERPROFILE\.mini-dbs\postgres",
    [string]$Database = "postgres"
)

if (-not (Test-Path "$InstallDir\bin\psql.exe")) {
    Write-Host "[!] psql.exe not found at $InstallDir\bin." -ForegroundColor Red
    exit 1
}

$port = 5432
$user = "postgres"
if (Test-Path "$InstallDir\mini-dbs.json") {
    $cfg = Get-Content "$InstallDir\mini-dbs.json" | ConvertFrom-Json
    $port = $cfg.port
    $user = $cfg.username
}

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$env:PGCLIENTENCODING = "utf8"

Write-Host "Connecting to PostgreSQL ($user@127.0.0.1:$port/$Database)..." -ForegroundColor Cyan
& "$InstallDir\bin\psql.exe" -h 127.0.0.1 -p $port -U $user $Database
