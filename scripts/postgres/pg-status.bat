@echo off
setlocal

set "INSTALL_DIR=%USERPROFILE%\.mini-dbs\postgres"
if not "%~1"=="" set "INSTALL_DIR=%~1"

if not exist "%INSTALL_DIR%\bin\pg_ctl.exe" (
    echo [!] PostgreSQL not found at %INSTALL_DIR%.
    exit /b 1
)

set "PGDATA=%INSTALL_DIR%\data"
"%INSTALL_DIR%\bin\pg_ctl.exe" -D "%PGDATA%" status

endlocal
