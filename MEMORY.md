# Project Memory: mini-dbs

## Core Mission
Provide Windows developers with a featherweight, zero-admin, CLI-first alternative to heavy database installers. Eliminates UAC elevation prompts, system-wide service locking, and gigabytes of GUI bloat (e.g. pgAdmin 4).

## Architecture & Design Decisions

### 1. Zero Admin / User Space Operation
- PostgreSQL on Windows does not require Administrator privileges to run `initdb.exe` or `postgres.exe` / `pg_ctl.exe`.
- In Unix and Windows, running PostgreSQL under an unprivileged user is the official security recommendation.
- User environment variables (`PATH`) can be modified safely via `HKEY_CURRENT_USER\Environment` using the `winreg` Rust crate or `[Environment]::SetEnvironmentVariable("Path", ..., "User")` in PowerShell, completely bypassing the need for administrative rights.

### 2. Binary Distribution Mechanism
- Binaries are sourced directly from EnterpriseDB's official portable Windows x64 zip packages:
  `https://get.enterprisedb.com/postgresql/postgresql-[VERSION]-windows-x64-binaries.zip`
- The zip package contains the full native toolchain (`pgsql/bin`, `pgsql/lib`, `pgsql/share`).
- By stripping the `pgsql/` root directory on extraction, binaries are cleanly placed into the user's chosen folder (`~/.mini-dbs/postgres`, `./pgsql`, or custom).

### 3. Resumable Chunked Downloads
- EnterpriseDB CDN servers (Amazon S3 + CloudFront) support `Accept-Ranges: bytes`.
- `mini-dbs` uses HTTP `Range: bytes={offset}-` requests targeting a `.part` file.
- If a download is killed, canceled, or drops, it detects the `.part` length and resumes from the exact byte position, avoiding wasted bandwidth.
- Graceful fallback to `curl -fL -C -` ensures rock-solid downloads across various Windows network setups.

### 4. Port Conflict Detection & Multi-Tenancy
- Many Windows machines already have a background `postgres.exe` running on port 5432 (e.g. installed by Docker, another app, or system services).
- `mini-dbs` checks port availability via `TcpListener::bind` in Rust and `Get-NetTCPConnection` in PowerShell.
- If 5432 is occupied, it automatically negotiates the next free port (`5433`, `5434`, etc.) and appends `port = <PORT>` to `postgresql.conf`.

### 5. Multi-Database Roadmap
- The project is named `mini-dbs` to accommodate future lightweight zero-admin engines:
  - Redis (via native Windows port or portable Memurai binaries)
  - SQLite (CLI + extension loader)
  - MariaDB / MySQL (portable unprivileged mysqld)

## Tooling & Build Environment Notes
- Host environment on this system has Windows SDK 10 installed on the `E:\` drive (`E:\Windows Kits\10\Lib\10.0.22621.0` and `E:\Windows Kits\10\Include\10.0.22621.0`).
- Configured `.cargo/config.toml` with `rustflags = ["-L", "native=..."]` to ensure MSVC linking resolves without manual environment variable setup.
