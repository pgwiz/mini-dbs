<#
.SYNOPSIS
    Lightweight, zero-admin installer for PostgreSQL on Windows with interactive menu & resumable downloads.
.DESCRIPTION
    Downloads official PostgreSQL Windows x64 binary archive directly from EnterpriseDB,
    supports chunked resumable downloads via Range headers, allows custom install location,
    initializes a UTF-8 cluster with initdb, sets up custom ports, and generates launcher scripts.
.EXAMPLE
    .\install.ps1 -Interactive
    .\install.ps1 -InstallDir "D:\Databases\postgres" -Port 5434
#>

[CmdletBinding()]
param(
    [string]$Version = "17.4-1",
    [string]$InstallDir = "",
    [int]$Port = 0,
    [string]$Username = "postgres",
    [string]$Password = "",
    [switch]$AddToPath = $false,
    [switch]$Force = $false,
    [switch]$Interactive = $false
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "       mini-dbs: Zero-Admin PostgreSQL CLI Setup          " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# Interactive Menu if requested or if run without specific InstallDir
if ($Interactive -or [string]::IsNullOrWhiteSpace($InstallDir)) {
    Write-Host "`n--- Interactive Setup Wizard ---" -ForegroundColor Yellow

    # Location choice
    Write-Host "`nSelect Installation Location:" -ForegroundColor Cyan
    Write-Host "  [1] Default: $env:USERPROFILE\.mini-dbs\postgres"
    Write-Host "  [2] Current Directory: .\pgsql"
    Write-Host "  [3] Custom Directory Path"
    $locChoice = Read-Host "Choose location [1-3] (Default: 1)"
    if ($locChoice -eq "2") {
        $InstallDir = Join-Path (Get-Location) "pgsql"
    } elseif ($locChoice -eq "3") {
        $customPath = Read-Host "Enter full directory path"
        $InstallDir = if ($customPath) { $customPath } else { "$env:USERPROFILE\.mini-dbs\postgres" }
    } else {
        $InstallDir = "$env:USERPROFILE\.mini-dbs\postgres"
    }

    # Version choice
    Write-Host "`nSelect PostgreSQL Version:" -ForegroundColor Cyan
    Write-Host "  [1] 17.4-1 (Latest Stable, Recommended)"
    Write-Host "  [2] 16.8-1"
    $vChoice = Read-Host "Choose version [1-2] (Default: 1)"
    if ($vChoice -eq "2") { $Version = "16.8-1" }

    # Port choice
    if ($Port -eq 0) {
        $defaultPort = 5432
        $inUse = Get-NetTCPConnection -LocalPort 5432 -State Listen -ErrorAction SilentlyContinue
        if ($inUse) {
            $candidate = 5433
            while (Get-NetTCPConnection -LocalPort $candidate -State Listen -ErrorAction SilentlyContinue) {
                $candidate++
            }
            Write-Host "`nPort 5432 is occupied. Next free port is: $candidate" -ForegroundColor Yellow
            $defaultPort = $candidate
        }
        $portInput = Read-Host "Enter TCP Port (Default: $defaultPort)"
        $Port = if ($portInput) { [int]$portInput } else { $defaultPort }
    }

    # Superuser name
    $userInput = Read-Host "Enter Superuser Name (Default: postgres)"
    if ($userInput) { $Username = $userInput }

    # Password
    $pwInput = Read-Host "Enter Password (leave empty for trust/no password)"
    if ($pwInput) { $Password = $pwInput }

    # Add to User PATH
    $pathInput = Read-Host "Add to User PATH permanently? (Zero Admin) [y/N]"
    if ($pathInput -match "^[yY]") { $AddToPath = $true }
}

if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    $InstallDir = "$env:USERPROFILE\.mini-dbs\postgres"
}
if ($Port -eq 0) {
    $Port = 5432
}

Write-Host "`n[Configuration Summary]" -ForegroundColor Cyan
Write-Host "  Version:     $Version" -ForegroundColor Gray
Write-Host "  Install Dir: $InstallDir" -ForegroundColor Gray
Write-Host "  Port:        $Port" -ForegroundColor Gray
Write-Host "  User:        $Username" -ForegroundColor Gray

# 1. Check if already installed
if ((Test-Path "$InstallDir\bin\postgres.exe") -and -not $Force) {
    Write-Host "`n[!] PostgreSQL is already installed at: $InstallDir" -ForegroundColor Yellow
    Write-Host "Use -Force to reinstall or overwrite." -ForegroundColor Gray
    return
}

# 2. Port conflict detection
$portInUse = (Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue)
if ($portInUse) {
    Write-Host "`n[!] Port $Port is currently in use." -ForegroundColor Yellow
    $candidatePort = $Port + 1
    while (Get-NetTCPConnection -LocalPort $candidatePort -State Listen -ErrorAction SilentlyContinue) {
        $candidatePort++
    }
    Write-Host "[+] Automatically selecting available port: $candidatePort" -ForegroundColor Green
    $Port = $candidatePort
}

# 3. Create target directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$tempZip = Join-Path $env:TEMP "postgresql-$Version-windows-x64-binaries.zip"
$partZip = Join-Path $env:TEMP "postgresql-$Version-windows-x64-binaries.zip.part"
$downloadUrl = "https://get.enterprisedb.com/postgresql/postgresql-$Version-windows-x64-binaries.zip"

# 4. Resumable Chunked Download
Write-Host "`n[1/4] Downloading official PostgreSQL $Version binaries..." -ForegroundColor Cyan

if (Test-Path $tempZip) {
    Write-Host "Found cached complete archive in TEMP ($tempZip)." -ForegroundColor Green
} else {
    Write-Host "Connecting with range resume support to: $downloadUrl" -ForegroundColor DarkGray
    
    # Check if partial download exists
    $existingLength = 0
    if (Test-Path $partZip) {
        $existingLength = (Get-Item $partZip).Length
        Write-Host "Resuming partial download from byte $existingLength ($([math]::Round($existingLength/1MB, 2)) MB)..." -ForegroundColor Yellow
    }

    $downloadSuccess = $false
    try {
        # Using curl.exe with -C - for chunked resumable download if available
        $curl = Get-Command curl.exe -ErrorAction SilentlyContinue
        if ($curl) {
            Write-Host "Using curl with automatic range resume (-C -)..." -ForegroundColor DarkGray
            & curl.exe -fL -C - "$downloadUrl" -o "$partZip"
            if ($LASTEXITCODE -eq 0 -and (Test-Path $partZip)) {
                Move-Item -Path $partZip -Destination $tempZip -Force
                $downloadSuccess = $true
            }
        }
    } catch {
        Write-Host "curl resume attempt failed. Using .NET WebClient fallback..." -ForegroundColor Yellow
    }

    if (-not $downloadSuccess -and -not (Test-Path $tempZip)) {
        # Fallback to standard .NET download
        $wc = New-Object System.Net.WebClient
        $wc.Headers.Add("User-Agent", "mini-dbs-installer/0.1")
        $wc.DownloadFile($downloadUrl, $tempZip)
    }
    Write-Host "Archive ready." -ForegroundColor Green
}

# 5. Extract Binaries
Write-Host "`n[2/4] Extracting binaries to $InstallDir..." -ForegroundColor Cyan
$extractTemp = Join-Path $env:TEMP "pgsql_extract_$([guid]::NewGuid().ToString().Substring(0,8))"
try {
    Expand-Archive -Path $tempZip -DestinationPath $extractTemp -Force
    $sourceDir = if (Test-Path "$extractTemp\pgsql") { "$extractTemp\pgsql" } else { $extractTemp }
    Copy-Item -Path "$sourceDir\*" -Destination $InstallDir -Recurse -Force
    Remove-Item -Path $extractTemp -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "Binaries extracted successfully." -ForegroundColor Green
} catch {
    Write-Error "Failed to extract archive: $_"
    exit 1
}

# 6. Initialize Cluster with initdb
$dataDir = Join-Path $InstallDir "data"
Write-Host "`n[3/4] Initializing database cluster at $dataDir..." -ForegroundColor Cyan

$initdb = Join-Path $InstallDir "bin\initdb.exe"
if (-not (Test-Path $initdb)) {
    Write-Error "Could not find initdb.exe at $initdb"
    exit 1
}

if (-not (Test-Path "$dataDir\PG_VERSION")) {
    $initArgs = @(
        "-D", "$dataDir",
        "-U", "$Username",
        "-E", "UTF8",
        "--locale=C"
    )

    if ([string]::IsNullOrWhiteSpace($Password)) {
        $initArgs += @("-A", "trust")
        Write-Host "Configuring authentication: trust (frictionless local development)" -ForegroundColor DarkGray
    } else {
        $pwFile = Join-Path $env:TEMP "pg_pw_$([guid]::NewGuid().ToString().Substring(0,8)).tmp"
        Set-Content -Path $pwFile -Value $Password -NoNewline
        $initArgs += @("-A", "scram-sha-256", "--pwfile=$pwFile")
    }

    try {
        & $initdb $initArgs
        if ($LASTEXITCODE -ne 0) {
            Write-Error "initdb failed with exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }
    } finally {
        if ($pwFile -and (Test-Path $pwFile)) {
            Remove-Item $pwFile -Force -ErrorAction SilentlyContinue
        }
    }

    if ($Port -ne 5432) {
        $confFile = Join-Path $dataDir "postgresql.conf"
        Add-Content -Path $confFile -Value "`n# Configured by mini-dbs`nport = $Port`n"
    }
    
    Write-Host "Database cluster initialized successfully." -ForegroundColor Green
} else {
    Write-Host "Existing database cluster found in $dataDir. Skipping initdb." -ForegroundColor Yellow
}

# 7. Generate Helper Scripts in InstallDir
Write-Host "`n[4/4] Generating management scripts..." -ForegroundColor Cyan

@"
@echo off
setlocal
set "PGDATA=$dataDir"
set "PGBIN=$InstallDir\bin"
"%PGBIN%\pg_ctl.exe" -D "%PGDATA%" -l "%PGDATA%\logfile" start
echo PostgreSQL started on port $Port.
endlocal
"@ | Set-Content -Path "$InstallDir\start.bat" -Encoding ASCII

@"
@echo off
setlocal
set "PGDATA=$dataDir"
set "PGBIN=$InstallDir\bin"
"%PGBIN%\pg_ctl.exe" -D "%PGDATA%" stop -m fast
echo PostgreSQL stopped.
endlocal
"@ | Set-Content -Path "$InstallDir\stop.bat" -Encoding ASCII

@"
@echo off
setlocal
set "PGDATA=$dataDir"
set "PGBIN=$InstallDir\bin"
"%PGBIN%\pg_ctl.exe" -D "%PGDATA%" status
endlocal
"@ | Set-Content -Path "$InstallDir\status.bat" -Encoding ASCII

@"
@echo off
chcp 65001 > nul
set "PGBIN=$InstallDir\bin"
"%PGBIN%\psql.exe" -h 127.0.0.1 -p $Port -U $Username postgres
"@ | Set-Content -Path "$InstallDir\psql.bat" -Encoding ASCII

# Config file
@{
    version     = $Version
    installDir  = $InstallDir
    dataDir     = $dataDir
    port        = $Port
    username    = $Username
    installedAt = (Get-Date).ToString("o")
} | ConvertTo-Json -Depth 4 | Set-Content -Path "$InstallDir\mini-dbs.json"

# 8. Add to User PATH if requested (Zero Admin!)
if ($AddToPath) {
    Write-Host "`nRegistering bin directory to User PATH..." -ForegroundColor Cyan
    $binDir = "$InstallDir\bin"
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -split ';' -notcontains $binDir) {
        $newPath = if ($userPath) { "$userPath;$binDir" } else { $binDir }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        $env:PATH = "$env:PATH;$binDir"
        Write-Host "Added $binDir to User PATH environment variable." -ForegroundColor Green
    } else {
        Write-Host "$binDir is already in User PATH." -ForegroundColor Yellow
    }
}

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host "  PostgreSQL installed successfully! (Zero Admin Required) " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "Binaries:       $InstallDir\bin" -ForegroundColor White
Write-Host "Data Cluster:   $dataDir" -ForegroundColor White
Write-Host "Port:           $Port" -ForegroundColor White
Write-Host "User:           $Username" -ForegroundColor White
Write-Host "`nQuick Commands:" -ForegroundColor Cyan
Write-Host "  Start server:   $InstallDir\start.bat" -ForegroundColor White
Write-Host "  Stop server:    $InstallDir\stop.bat" -ForegroundColor White
Write-Host "  Check status:   $InstallDir\status.bat" -ForegroundColor White
Write-Host "  Open CLI (psql):$InstallDir\psql.bat" -ForegroundColor White
Write-Host "==========================================================" -ForegroundColor Green
