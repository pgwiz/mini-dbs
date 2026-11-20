[CmdletBinding()]
param(
    [string]$InstallDir = "$env:USERPROFILE\.mini-dbs\postgres"
)

if (-not (Test-Path "$InstallDir\bin\pg_ctl.exe")) {
    Write-Host "[!] PostgreSQL not found at $InstallDir." -ForegroundColor Red
    exit 1
}

$dataDir = "$InstallDir\data"

Write-Host "PostgreSQL Server Status:" -ForegroundColor Cyan
& "$InstallDir\bin\pg_ctl.exe" -D $dataDir status

if (Test-Path "$InstallDir\mini-dbs.json") {
    $cfg = Get-Content "$InstallDir\mini-dbs.json" | ConvertFrom-Json
    $conn = Get-NetTCPConnection -LocalPort $cfg.port -State Listen -ErrorAction SilentlyContinue
    if ($conn) {
        Write-Host "[✓] Port $($cfg.port) is LISTENING (PID: $($conn.OwningProcess))" -ForegroundColor Green
    } else {
        Write-Host "[-] Port $($cfg.port) is not active." -ForegroundColor Yellow
    }
}
