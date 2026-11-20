# Agent Guidelines: mini-dbs

## Overview
`mini-dbs` is a zero-admin, lightweight, CLI-first database suite and manager for Windows. It provides portable database environments (starting with PostgreSQL) without requiring UAC elevation, GUI installers, or permanent Windows services.

## Repository Rules
1. **Markdown File Policy**:
   - Only maintain `README.md`, `CHANGELOG.md`, `MEMORY.md`, and `AGENT.md`.
   - Do not create superfluous markdown documentation files unless explicitly requested by the user.
2. **Visual & UI Policy**:
   - Never use glowing blobs, ambient blur background circles, or glowing dots/indicators. Keep interfaces clean, tactile, and text/terminal focused.
3. **Links**:
   - Always link referenced files with file:/// URIs and basename link texts, e.g. `[README.md](file:///e:/Backup/pgwiz/pg-win-cli/README.md)`.

## Project Structure
- `src/main.rs`: Clap CLI entry point and dispatch logic.
- `src/menu.rs`: Interactive terminal mini-menu.
- `src/postgres.rs`: Core database initialization, cluster setup, and process lifecycle (`start`, `stop`, `restart`, `status`, `psql`).
- `src/downloader.rs`: Resumable HTTP Range chunked downloader with `.part` management and `curl -C -` fallback.
- `src/win_utils.rs`: Windows-specific helpers: User registry PATH manipulation (`HKEY_CURRENT_USER\Environment`), TCP port checking.
- `src/config.rs`: Metadata schema (`mini-dbs.json`) stored inside instance directories.
- `scripts/postgres/`: Zero-dependency PowerShell and Batch scripts for non-Rust environments.

## Build & Test Commands
```powershell
# Build debug binary
cargo build

# Build optimized release binary
cargo build --release

# Run interactive menu
cargo run --release -- menu

# Run automated headless setup with custom directory and port
cargo run --release -- pg setup --dir ".\test_db" --port 5434

# Check server status
cargo run --release -- pg status --dir ".\test_db"

# Stop server
cargo run --release -- pg stop --dir ".\test_db"
```
