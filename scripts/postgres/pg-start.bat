@echo off
setlocal

set "INSTALL_DIR=%USERPROFILE%\.mini-dbs\postgres"
if not "%~1"=="" set "INSTALL_DIR=%~1"

if not exist "%INSTALL_DIR%\bin\pg_ctl.exe" (
    echo [!] PostgreSQL not found at %INSTALL_DIR%. Run install.ps1 first.
    exit /b 1
)

set "PGDATA=%INSTALL_DIR%\data"
set "PGLOG=%PGDATA%\logfile"

echo [*] Starting PostgreSQL server...
"%INSTALL_DIR%\bin\pg_ctl.exe" -D "%PGDATA%" -l "%PGLOG%" start
if %ERRORLEVEL% EQU 0 (
    echo [✓] PostgreSQL server is running.
) else (
    echo [!] Failed to start PostgreSQL. Check logs at: %PGLOG%
)

endlocal
