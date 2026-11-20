mod config;
mod downloader;
mod menu;
mod postgres;
mod win_utils;

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mini-dbs",
    author = "pgwiz",
    version = "0.1.0",
    about = "Zero-admin, lightweight, CLI-first database manager for Windows",
    long_about = "mini-dbs gives developers portable, zero-admin database environments on Windows.\nNo heavy installers, no UAC prompts, no pgAdmin bloat - just pure, fast CLI tools."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Launch interactive mini-menu
    #[arg(short, long)]
    menu: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive terminal wizard & manager menu
    Menu,

    /// Manage PostgreSQL database instances
    #[command(alias = "pg")]
    Postgres(PgCommand),

    /// Display system environment, port usage, and mini-dbs status
    Info,
}

#[derive(Args)]
struct PgCommand {
    #[command(subcommand)]
    subcmd: PgSubcommands,
}

#[derive(Subcommand)]
enum PgSubcommands {
    /// Download and set up a zero-admin PostgreSQL CLI cluster
    Setup {
        /// PostgreSQL version archive to download (e.g. 17.4-1, 16.8-1)
        #[arg(short, long, default_value = "17.4-1")]
        version: String,

        /// Destination directory for PostgreSQL installation
        #[arg(short, long)]
        dir: Option<PathBuf>,

        /// TCP Port (auto-detects and increments if already in use)
        #[arg(short, long, default_value_t = 5432)]
        port: u16,

        /// Default database superuser name
        #[arg(short, long, default_value = "postgres")]
        user: String,

        /// Optional password (defaults to trust authentication for frictionless dev)
        #[arg(long)]
        password: Option<String>,

        /// Automatically register bin folder to current user's PATH (no admin required)
        #[arg(long, default_value_t = false)]
        add_to_path: bool,

        /// Force re-installation even if directory exists
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },

    /// Start the PostgreSQL server in the background
    Start {
        /// Target installation directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },

    /// Stop the PostgreSQL server cleanly
    Stop {
        /// Target installation directory
        #[arg(short, long)]
        dir: Option<PathBuf>,

        /// Shutdown mode: fast, smart, or immediate
        #[arg(short, long, default_value = "fast")]
        mode: String,
    },

    /// Restart the PostgreSQL server
    Restart {
        /// Target installation directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },

    /// Check if PostgreSQL server is running and listening
    Status {
        /// Target installation directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },

    /// Open an interactive psql command-line console
    #[command(alias = "shell", alias = "cli")]
    Psql {
        /// Target installation directory
        #[arg(short, long)]
        dir: Option<PathBuf>,

        /// Target database name
        #[arg(long)]
        db: Option<String>,
    },

    /// Output shell environment variables (PowerShell & CMD)
    Env {
        /// Target installation directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },

    /// Register PostgreSQL bin directory to User PATH permanently without admin
    AddPath {
        /// Target installation directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
}

fn resolve_dir(dir: Option<PathBuf>) -> PathBuf {
    dir.unwrap_or_else(postgres::default_install_dir)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.menu || cli.command.is_none() {
        return menu::run_interactive_menu();
    }

    match cli.command.unwrap() {
        Commands::Menu => {
            menu::run_interactive_menu()?;
        }
        Commands::Postgres(pg) => match pg.subcmd {
            PgSubcommands::Setup {
                version,
                dir,
                port,
                user,
                password,
                add_to_path,
                force,
            } => {
                let install_dir = resolve_dir(dir);
                let opts = postgres::SetupOptions {
                    version,
                    install_dir,
                    port,
                    username: user,
                    password,
                    add_to_path,
                    force,
                };
                postgres::run_setup(opts)?;
            }
            PgSubcommands::Start { dir } => {
                let install_dir = resolve_dir(dir);
                postgres::run_start(&install_dir)?;
            }
            PgSubcommands::Stop { dir, mode } => {
                let install_dir = resolve_dir(dir);
                postgres::run_stop(&install_dir, &mode)?;
            }
            PgSubcommands::Restart { dir } => {
                let install_dir = resolve_dir(dir);
                postgres::run_restart(&install_dir)?;
            }
            PgSubcommands::Status { dir } => {
                let install_dir = resolve_dir(dir);
                postgres::run_status(&install_dir)?;
            }
            PgSubcommands::Psql { dir, db } => {
                let install_dir = resolve_dir(dir);
                postgres::run_psql(&install_dir, db)?;
            }
            PgSubcommands::Env { dir } => {
                let install_dir = resolve_dir(dir);
                postgres::run_print_env(&install_dir)?;
            }
            PgSubcommands::AddPath { dir } => {
                let install_dir = resolve_dir(dir);
                let bin_dir = install_dir.join("bin").to_string_lossy().to_string();
                match win_utils::add_to_user_path(&bin_dir) {
                    Ok(true) => println!("{} Added {} to User PATH.", "[✓]".green(), bin_dir),
                    Ok(false) => println!("{} Already present in User PATH.", "[!]".yellow()),
                    Err(e) => eprintln!("{} Failed to update User PATH: {}", "[!]".red(), e),
                }
            }
        },
        Commands::Info => {
            println!("{}", "mini-dbs System Environment:".cyan().bold());
            println!("  Default Install Dir: {}", postgres::default_install_dir().display());
            println!("  User PATH:          {}", win_utils::get_user_path().unwrap_or_default());
            println!("  Port 5432 free?     {}", win_utils::is_port_available(5432));
            println!("  Next free PG port:  {}", win_utils::find_available_port(5432));
            println!("\nSupported Database Engines:");
            println!("  [✓] PostgreSQL (Zero-admin binaries directly from EnterpriseDB CDN)");
            println!("  [...] Redis, SQLite, MariaDB (Planned roadmap)");
        }
    }

    Ok(())
}
