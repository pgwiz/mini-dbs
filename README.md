# mini-dbs

> **Zero-Admin, Featherweight, CLI-First Database Suite for Windows.**  
> Drop the 1.5 GB installers, avoid UAC administrator prompts, and run clean, isolated database instances straight from your terminal.

---

## ⚡ Executive Summary & Ecosystem Research

### Does this idea already exist?

Yes and no. The pieces exist across fragmented sources, but **a dedicated, zero-admin, CLI-first, portable database suite with automated tooling and an interactive menu did not exist until `mini-dbs`**.

Here is how existing solutions compare:

| Solution | Admin / UAC Required? | Download / Install Size | CLI-Only? | Auto Resumable Downloads? | Interactive Menu / Manager? | Custom Per-Project Paths? |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Official EDB Windows Installer** | ❌ **YES (Mandatory UAC)** | ~1.5 GB+ (Bundles pgAdmin 4, StackBuilder, Python) | ❌ GUI Bloatware | ❌ No | ❌ Installer Wizard | ❌ Fixed System Service |
| **[kanad13/postgresql-windows-no-admin](https://github.com/kanad13/postgresql-windows-no-admin)** | ✅ No | ~300 MB zip | ✅ Manual CLI | ❌ Manual Web Download | ❌ Text Guide Only | ⚠️ Manual Script Editing |
| **Scoop (`scoop install postgresql`)** | ✅ No | ~300 MB zip | ✅ CLI | ❌ Depends on Scoop | ❌ Package Manager Only | ❌ Fixed `~/scoop/apps` |
| **Chocolatey / Winget** | ❌ **YES** | ~1.5 GB | ❌ Wraps EDB Installer | ❌ No | ❌ No | ❌ System Path |
| **Docker Desktop / WSL2** | ❌ **YES (Hyper-V / WSL)** | ~4 GB+ RAM overhead | ⚠️ Virtualized | ❌ Pulls image | ❌ Container CLI | ⚠️ Volume Mounts |
| **`mini-dbs` (This Project)** | ✅ **100% Zero Admin** | **~120 MB extracted** | ✅ **Pure CLI Tools** | ✅ **Resumable HTTP Ranges** | ✅ **Interactive Mini Menu + Fast CLI** | ✅ **Project-Local or Custom Path** |

### Why `mini-dbs`?
1. **Zero Admin Rights**: Runs entirely inside user space (`%USERPROFILE%\.mini-dbs` or any custom folder like `.\pgsql`). Perfect for corporate-managed laptops, university labs, and restricted development machines.
2. **Pure CLI Performance**: Strips away 1.5+ GB of heavy GUI packages (pgAdmin 4, StackBuilder, local web server daemons). Installs only the true command-line database toolchain: `psql`, `pg_ctl`, `initdb`, `createdb`, `dropdb`, `pg_dump`, `pg_restore`.
3. **Resumable Chunked Downloads**: CloudFront/EDB network connections can be throttled or drop. `mini-dbs` utilizes HTTP `Range: bytes={offset}-` chunking (and curl `-C -` fallback) so partial downloads never start over.
4. **Interactive Mini Menu**: Run `mini-dbs` or `mini-dbs menu` for an interactive CLI wizard that guides location selection, port assignment, and server operations.
5. **Conflict-Free Port Allocation**: Automatically checks if port `5432` is listening (e.g. from another Postgres service) and safely negotiates the next free port (e.g. `5433`, `5434`).
6. **Future Multi-Database Architecture**: Designed to expand into a unified lightweight suite for Redis, SQLite, and MariaDB/MySQL.

---

## 🛠️ Features

- 🚀 **Dual Interface**:
  - **Native Rust Binary (`mini-dbs.exe`)**: Single-file, blazingly fast executable written in Rust with built-in resumable download manager, zip extractor, process supervisor, and user registry modifier.
  - **Zero-Dependency Scripts**: Ready-to-run PowerShell (`.ps1`) and Command Prompt (`.bat`) scripts located in [`scripts/postgres/`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres).
- 📂 **Flexible Location Selection**: Install to user home (`~/.mini-dbs/postgres`), current repository/project directory (`./pgsql`), or any custom disk path.
- 🔄 **Safe User PATH Registration**: Easily expose `psql` and `pg_ctl` to your Windows user environment without administrative privileges.
- 🛡️ **UTF-8 By Default**: Configures `UTF-8` database cluster encoding and sets Windows code page `chcp 65001` automatically for seamless international text and symbols in the Windows console.

---

## 🚀 Quick Start

### Option 1: Native Rust CLI (`mini-dbs`)

#### 1. Interactive Mini Menu
Simply run the binary without arguments:
```powershell
.\target\release\mini-dbs.exe
# or
mini-dbs menu
```
You will see the interactive wizard:
```text
==========================================================
       mini-dbs: Zero-Admin Database Suite (Windows)      
==========================================================
  [1] Setup PostgreSQL (Specify Location, Port & Auth)
  [2] Start PostgreSQL Server
  [3] Stop PostgreSQL Server
  [4] Check Server Status & Port Listener
  [5] Open psql Interactive CLI Shell
  [6] Add PostgreSQL to User PATH (Zero Elevation)
  [7] View Shell Environment Configuration (ps1/cmd)
  [8] Manage Extensions (List available or Install .zip)
  [9] Process Activity & RAM Monitor (Live Monitor, Stop All)
  [10] System Diagnostics & Active Ports
  [0] Exit
----------------------------------------------------------
Select an option [0-10]:
```

#### 2. Headless / Automated CLI Setup
```powershell
# Default AppData destination (%LOCALAPPDATA%\mini-dbs\postgres):
mini-dbs pg setup

# Or specify your own maintainable custom directory (e.g. project-local or dedicated drive):
mini-dbs pg setup --dir ".\pgsql" --port 5434
mini-dbs pg setup --dir "D:\Databases\postgres" --port 5434

# Automatically add binaries to User PATH (zero admin):
mini-dbs pg setup --add-to-path
```

#### 3. Memory-Efficient Process Activity & Supervisor
`mini-dbs` includes an ultra-lightweight process supervisor (< 2 MB RAM, ~0.0% CPU) to inspect and manage running databases:
```powershell
# View instant process table with active PIDs, ports, and real-time RAM (Working Set):
mini-dbs ps

# Run live, continuous activity supervisor:
mini-dbs top          # or: mini-dbs monitor

# Stop all active database instances across all folders with 1 command:
mini-dbs stop-all
```

#### 4. Database Lifecycle Management
```powershell
# Start server in background
mini-dbs pg start

# Check status and port
mini-dbs pg status

# Launch interactive psql shell directly
mini-dbs pg psql

# Stop server cleanly
mini-dbs pg stop
```

---

### Option 2: PowerShell Scripts (Zero Install)

If you don't want to build or run the Rust binary, use the PowerShell scripts:

```powershell
# Interactive setup wizard
.\scripts\postgres\install.ps1 -Interactive

# Or silent setup with custom directory & port
.\scripts\postgres\install.ps1 -InstallDir ".\pgsql" -Port 5434 -AddToPath

# Start server
.\scripts\postgres\pg-start.ps1

# Check status
.\scripts\postgres\pg-status.ps1

# Connect with CLI
.\scripts\postgres\pg-cli.ps1

# Stop server
.\scripts\postgres\pg-stop.ps1
```

---

### Option 3: Double-Click Batch Files

For non-PowerShell environments or double-click lovers:
- Double click [`install.ps1`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres/install.ps1) or run `scripts\postgres\pg-start.bat`
- Use [`pg-cli.bat`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres/pg-cli.bat) to launch the `psql` console instantly.
- Use [`pg-stop.bat`](file:///e:/Backup/pgwiz/pg-win-cli/scripts/postgres/pg-stop.bat) to stop.

---

## 🏗️ Repository Layout

```text
mini-dbs/
├── src/
│   ├── main.rs          # CLI entrypoint & subcommands
│   ├── menu.rs          # Interactive terminal mini-menu
│   ├── postgres.rs      # PostgreSQL setup, initdb, process supervisor
│   ├── downloader.rs    # Resumable chunked download engine (Range headers)
│   ├── win_utils.rs     # User registry PATH & port collision detector
│   └── config.rs        # mini-dbs.json schema & persistence
├── scripts/
│   └── postgres/
│       ├── install.ps1  # Automated zero-admin PowerShell installer
│       ├── pg-start.ps1 # Start server (PowerShell)
│       ├── pg-stop.ps1  # Stop server (PowerShell)
│       ├── pg-status.ps1# Check status (PowerShell)
│       ├── pg-cli.ps1   # Connect to psql shell (PowerShell)
│       ├── pg-env.ps1   # Export env vars to current shell
│       ├── pg-start.bat # Start server (Batch)
│       ├── pg-stop.bat  # Stop server (Batch)
│       ├── pg-status.bat# Check status (Batch)
│       └── pg-cli.bat   # Connect to psql shell (Batch)
├── Cargo.toml           # Rust package configuration
├── README.md            # Main documentation
├── CHANGELOG.md         # Release history
├── MEMORY.md            # Architectural notes & design decisions
└── AGENT.md             # Developer & Agent instructions
```

---

## 🧩 Extension Management (pgvector, pgcrypto, etc.)

One of the greatest headaches with standard PostgreSQL on Windows is installing extensions. Because standard installations live in `C:\Program Files`, Windows UAC blocks you from adding `.dll` or `.control` files without administrator rights.

**With `mini-dbs`, you have full write permissions to your database directory with zero administrator rights required.**

### 1. Built-in Extensions (60+ Pre-Installed)
The official binaries archive comes pre-packaged with over 60 contrib extensions:
- **Crypto & Security**: `pgcrypto`, `uuid-ossp`, `sslinfo`
- **Text & Search**: `pg_trgm`, `citext`, `fuzzystrmatch`, `unaccent`
- **Data Types & Analytics**: `hstore`, `ltree`, `cube`, `bloom`, `btree_gist`, `pg_stat_statements`
- **Federation**: `postgres_fdw`, `dblink`

To enable any built-in extension, open `psql` and run:
```sql
CREATE EXTENSION pgcrypto;
CREATE EXTENSION "uuid-ossp";
CREATE EXTENSION pg_trgm;
```

To list all available extensions on your machine:
```powershell
mini-dbs pg ext list
```

### 2. Third-Party Extensions (e.g., `pgvector`)
For extensions that do not ship with PostgreSQL by default:
1. Download a precompiled Windows binary `.zip` for your PostgreSQL major version (for example, community builds from `andreiramani/pgvector_pgsql_windows` or `portalcorp/pgvector_compiled`).
2. Run the automated installer:
   ```powershell
   mini-dbs pg ext install .\path\to\vector.v0.8.0-pg17.zip
   ```
   Or manually drop:
   - `vector.dll` into `<install_dir>\lib\`
   - `vector.control` and `vector--*.sql` into `<install_dir>\share\extension\`
3. Connect with `psql` and enable:
   ```sql
   CREATE EXTENSION vector;
   ```

---

## 🔒 Security & Best Practices

- **Zero Privilege Escalation**: PostgreSQL deliberately discourages running as root/Administrator. Running inside user space is inherently more secure.
- **Local Dev vs. Production**: By default, `mini-dbs` configures `trust` authentication on `127.0.0.1` for frictionless developer ergonomics. For password-protected setups, pass `--password <your_password>`.
- **Multi-Tenant Isolation**: You can run as many independent clusters as you want by passing different `--dir` and `--port` parameters.

---

## 🗺️ Roadmap

- [x] Resumable chunked binary downloads (HTTP Range)
- [x] Interactive terminal mini-menu
- [x] Custom location specification (`--dir` / prompt)
- [x] Auto-port collision detection
- [x] Safe User PATH registration without elevation
- [ ] Mini Redis (Zero-admin Redis server on Windows)
- [ ] Mini SQLite CLI & extensions manager
- [ ] Ephemeral scratch database runner (`mini-dbs pg temp`)
