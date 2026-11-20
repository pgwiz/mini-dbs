@echo off
setlocal
chcp 65001 > nul

set "INSTALL_DIR=%USERPROFILE%\.mini-dbs\postgres"
if not "%~1"=="" set "INSTALL_DIR=%~1"

if not exist "%INSTALL_DIR%\bin\psql.exe" (
    echo [!] psql.exe not found at %INSTALL_DIR%\bin.
    exit /b 1
)

set "PGCLIENTENCODING=utf8"
echo [*] Launching PostgreSQL CLI (psql)...
"%INSTALL_DIR%\bin\psql.exe" -h 127.0.0.1 -p 5432 -U postgres postgres

endlocal
