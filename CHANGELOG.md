# Changelog

All notable changes to the `mini-dbs` project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-20

### Added
- **Core Engine & Architecture**:
  - Zero-admin PostgreSQL setup architecture for Windows, bypassing the heavy EnterpriseDB GUI installer.
  - Native Rust CLI executable (`mini-dbs.exe`) for fast, single-binary database cluster initialization, management, and process supervision.
  - Multi-database roadmap support (starting with PostgreSQL, architected for Redis, SQLite, MariaDB).
- **Interactive Mini Menu**:
  - Terminal interactive wizard (`mini-dbs menu` or default execution without subcommands).
  - Prompts for version selection, custom location specification, TCP port configuration, credentials, and User PATH setup.
- **Resumable Chunked Downloader**:
  - Range-aware HTTP download manager in Rust utilizing `Range: bytes={offset}-` headers.
  - Partial file (`.part`) support ensuring interrupted downloads resume without restarting from byte 0.
  - Resumable `curl -C -` fallback engine.
- **Location & Collision Intelligence**:
  - Default installation destination set to `%LOCALAPPDATA%\mini-dbs\postgres`.
  - Full support for custom maintainable directories (e.g. project-local `.\pgsql`, `D:\Databases`, etc.) selectable via `-d/--dir` or the interactive menu.
  - Global instance registry tracking (`%LOCALAPPDATA%\mini-dbs\instances.json`) allowing multi-instance management across multiple drives.
  - TCP port conflict detection that automatically detects if `5432` is occupied and increments to the next available free port (e.g. `5433`, `5434`).
- **Memory-Efficient Process Activity & Supervisor**:
  - Direct Win32 `K32GetProcessMemoryInfo` integration measuring real-time Working Set RAM without third-party crate overhead.
  - `mini-dbs ps` command for instant tabular activity reporting.
  - `mini-dbs top` / `mini-dbs monitor` for continuous live activity monitoring (< 2 MB RAM, ~0.0% CPU).
  - `mini-dbs stop-all` command for cleanly shutting down all registered database instances with 1 command.
  - Dedicated PowerShell activity monitor: [`pg-monitor.ps1`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres/pg-monitor.ps1).
- **Automation Scripts**:
  - PowerShell zero-dependency automation: [`install.ps1`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres/install.ps1), [`pg-start.ps1`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres/pg-start.ps1), [`pg-stop.ps1`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres/pg-stop.ps1), [`pg-status.ps1`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres/pg-status.ps1), [`pg-cli.ps1`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres/pg-cli.ps1), [`pg-env.ps1`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres/pg-env.ps1).
  - Double-click Windows Batch launcher shortcuts: `pg-start.bat`, `pg-stop.bat`, `pg-status.bat`, `pg-cli.bat`.
- **System Environment & Extensions**:
  - Safe User PATH registration without elevation via `winreg`.
  - Automatic `UTF-8` code page enforcement (`chcp 65001`).
  - Extension management commands: `mini-dbs pg ext list` to inspect 60+ pre-packaged contrib extensions and `mini-dbs pg ext install <zip>` to automatically install third-party extensions like `pgvector` into user directories without administrator rights.
