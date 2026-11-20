<#
.SYNOPSIS
    Lightweight, memory-efficient process activity monitor for mini-dbs.
.DESCRIPTION
    Scans for active database processes, displays real-time RAM footprint (Working Set MB),
    listening ports, and provides one-click options to stop or manage instances.
#>

[CmdletBinding()]
param(
    [switch]$Watch = $false,
    [int]$Interval = 2
)

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function Show-ProcessActivity {
    Clear-Host
    Write-Host "=========================================================================================" -ForegroundColor Cyan
    Write-Host "               mini-dbs: Memory-Efficient Database Process Activity                      " -ForegroundColor Cyan
    Write-Host "=========================================================================================" -ForegroundColor Cyan
    Write-Host ("{0,-8} {1,-12} {2,-7} {3,-15} {4,-14} {5,-32}" -f "PID", "ENGINE", "PORT", "STATUS", "RAM USAGE", "INSTALL DIRECTORY") -ForegroundColor White
    Write-Host "-----------------------------------------------------------------------------------------" -ForegroundColor DarkGray

    # Candidate directories
    $dirs = @(
        "$env:LOCALAPPDATA\mini-dbs\postgres",
        Join-Path (Get-Location) "pgsql"
    )

    $regPath = "$env:LOCALAPPDATA\mini-dbs\instances.json"
    if (Test-Path $regPath) {
        try {
            $reg = Get-Content $regPath | ConvertFrom-Json
            if ($reg.instances) {
                $dirs += $reg.instances
            }
        } catch {}
    }

    $dirs = $dirs | Select-Object -Unique

    $runningCount = 0
    $totalRamMB = 0

    foreach ($dir in $dirs) {
        if (-not (Test-Path $dir)) { continue }

        $port = 5432
        $cfgFile = Join-Path $dir "mini-dbs.json"
        if (Test-Path $cfgFile) {
            try {
                $cfg = Get-Content $cfgFile | ConvertFrom-Json
                $port = $cfg.port
            } catch {}
        }

        $pidFile = Join-Path $dir "data\postmaster.pid"
        $pidStr = "-"
        $statusStr = "STOPPED"
        $ramStr = "-"
        $statusColor = "DarkGray"

        if (Test-Path $pidFile) {
            $lines = Get-Content $pidFile -ErrorAction SilentlyContinue
            if ($lines -and $lines.Count -ge 1) {
                $candidatePid = [int]$lines[0].Trim()
                $proc = Get-Process -Id $candidatePid -ErrorAction SilentlyContinue
                if ($proc) {
                    $pidStr = "$candidatePid"
                    $statusStr = "RUNNING"
                    $statusColor = "Green"
                    $runningCount++
                    $memMB = [math]::Round($proc.WorkingSet64 / 1MB, 1)
                    $totalRamMB += $memMB
                    $ramStr = "$memMB MB"

                    if ($lines.Count -ge 4) {
                        $port = [int]$lines[3].Trim()
                    }
                } else {
                    $statusStr = "STALE (DEAD)"
                    $statusColor = "Yellow"
                }
            }
        }

        Write-Host ("{0,-8} {1,-12} {2,-7} " -f $pidStr, "PostgreSQL", $port) -NoNewline
        Write-Host ("{0,-15} " -f $statusStr) -ForegroundColor $statusColor -NoNewline
        Write-Host ("{0,-14} " -f $ramStr) -ForegroundColor Green -NoNewline
        Write-Host ("{0,-32}" -f $dir) -ForegroundColor DarkGray
    }

    Write-Host "-----------------------------------------------------------------------------------------" -ForegroundColor DarkGray
    Write-Host "Total Active Instances: $runningCount | Total Memory Footprint: $([math]::Round($totalRamMB, 1)) MB" -ForegroundColor Cyan
    Write-Host "=========================================================================================" -ForegroundColor Cyan
}

if ($Watch) {
    Write-Host "Starting live background activity monitor. Press Ctrl+C to exit..." -ForegroundColor Yellow
    while ($true) {
        Show-ProcessActivity
        Write-Host "`n[Live Monitor] Sleeping ${Interval}s... (Zero CPU/minimal memory activity)" -ForegroundColor DarkGray
        Start-Sleep -Seconds $Interval
    }
} else {
    Show-ProcessActivity
    Write-Host "`nManagement Options:" -ForegroundColor Cyan
    Write-Host "  [1] Refresh table"
    Write-Host "  [2] Start live monitor mode"
    Write-Host "  [3] Stop ALL active instances"
    Write-Host "  [0] Exit"
    $ans = Read-Host "Select option [0-3] (Default: 0)"
    if ($ans -eq "2") {
        & $MyInvocation.MyCommand.Path -Watch -Interval $Interval
    } elseif ($ans -eq "3") {
        Write-Host "Stopping all active database instances..." -ForegroundColor Yellow
        $procs = Get-Process -Name "postgres" -ErrorAction SilentlyContinue
        foreach ($p in $procs) {
            Write-Host "Stopping PID $($p.Id)..." -ForegroundColor DarkGray
            Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
        }
        Write-Host "Done." -ForegroundColor Green
    }
}
