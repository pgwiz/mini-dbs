@echo off
setlocal

set "INSTALL_DIR=%USERPROFILE%\.mini-dbs\postgres"
if not "%~1"=="" set "INSTALL_DIR=%~1"

if not exist "%INSTALL_DIR%\bin\pg_ctl.exe" (
    echo [!] PostgreSQL not found at %INSTALL_DIR%.
    exit /b 1
)

set "PGDATA=%INSTALL_DIR%\data"

echo [*] Stopping PostgreSQL server...
"%INSTALL_DIR%\bin\pg_ctl.exe" -D "%PGDATA%" -m fast stop
if %ERRORLEVEL% EQU 0 (
    echo [✓] PostgreSQL server stopped.
) else (
    echo [!] Server not running or error stopping.
)

endlocal
