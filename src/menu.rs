use colored::Colorize;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::postgres::{self, SetupOptions};
use crate::win_utils::{add_to_user_path, find_available_port, is_port_available};

pub fn run_interactive_menu() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("\n{}", "==========================================================".cyan());
        println!("{}", "       mini-dbs: Zero-Admin Database Suite (Windows)      ".cyan().bold());
        println!("{}", "==========================================================".cyan());
        println!("  {} Setup PostgreSQL (Specify Location, Port & Auth)", "[1]".green().bold());
        println!("  {} Start PostgreSQL Server", "[2]".green().bold());
        println!("  {} Stop PostgreSQL Server", "[3]".green().bold());
        println!("  {} Check Server Status & Port Listener", "[4]".green().bold());
        println!("  {} Open psql Interactive CLI Shell", "[5]".green().bold());
        println!("  {} Add PostgreSQL to User PATH (Zero Elevation)", "[6]".green().bold());
        println!("  {} View Shell Environment Configuration (ps1/cmd)", "[7]".green().bold());
        println!("  {} System Diagnostics & Active Ports", "[8]".green().bold());
        println!("  {} Exit", "[0]".yellow());
        println!("{}", "----------------------------------------------------------".bright_black());
        print!("Select an option [0-8]: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        match choice {
            "1" => interactive_setup()?,
            "2" => interactive_start()?,
            "3" => interactive_stop()?,
            "4" => interactive_status()?,
            "5" => interactive_psql()?,
            "6" => interactive_add_path()?,
            "7" => interactive_env()?,
            "8" => interactive_diagnostics()?,
            "0" | "q" | "exit" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("{} Invalid option, please enter a number 0-8.", "[!]".red()),
        }
    }
    Ok(())
}

fn prompt(message: &str, default: Option<&str>) -> io::Result<String> {
    if let Some(def) = default {
        print!("{} [default: {}]: ", message, def.cyan());
    } else {
        print!("{}: ", message);
    }
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.unwrap_or_default().to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn interactive_setup() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "--- PostgreSQL Installation Setup ---".cyan().bold());

    // 1. Version
    println!("\nSelect PostgreSQL Version:");
    println!("  [1] 17.4-1 (Latest Stable, Recommended)");
    println!("  [2] 16.8-1 (PostgreSQL 16 LTS)");
    println!("  [3] Custom Version string");
    let v_choice = prompt("Choose version [1-3]", Some("1"))?;
    let version = match v_choice.as_str() {
        "2" => "16.8-1".to_string(),
        "3" => prompt("Enter exact EDB version string (e.g. 15.12-1)", Some("17.4-1"))?,
        _ => "17.4-1".to_string(),
    };

    // 2. Specify Location
    println!("\nSpecify Installation Location:");
    let default_dir = postgres::default_install_dir();
    let default_dir_str = default_dir.to_string_lossy().to_string();
    println!("  [1] User Home Directory ({})", default_dir_str.bright_black());
    println!("  [2] Current Directory (.\\pgsql)");
    println!("  [3] Custom Location (enter any full path)");
    let loc_choice = prompt("Choose location [1-3]", Some("1"))?;
    let install_dir: PathBuf = match loc_choice.as_str() {
        "2" => PathBuf::from(".\\pgsql"),
        "3" => {
            let custom = prompt("Enter full destination path", Some(&default_dir_str))?;
            PathBuf::from(custom)
        }
        _ => default_dir,
    };

    // 3. Port with conflict detection
    let mut default_port = 5432;
    if !is_port_available(5432) {
        default_port = find_available_port(5433);
        println!(
            "{} Default port 5432 is occupied. Suggesting free port: {}",
            "[!]".yellow(),
            default_port.to_string().green()
        );
    }
    let port_str = prompt("Enter TCP Port", Some(&default_port.to_string()))?;
    let port: u16 = port_str.parse().unwrap_or(default_port);

    // 4. Superuser
    let username = prompt("Enter Superuser Name", Some("postgres"))?;

    // 5. Password
    let password_raw = prompt("Enter Password (leave empty for trust auth/no password)", Some(""))?;
    let password = if password_raw.is_empty() {
        None
    } else {
        Some(password_raw)
    };

    // 6. Add to PATH
    let path_choice = prompt("Register bin directory to User PATH? (Zero Admin) [y/N]", Some("n"))?;
    let add_to_path = path_choice.eq_ignore_ascii_case("y") || path_choice.eq_ignore_ascii_case("yes");

    println!("\nStarting setup with specified configuration...");
    let opts = SetupOptions {
        version,
        install_dir,
        port,
        username,
        password,
        add_to_path,
        force: false,
    };

    postgres::run_setup(opts)?;
    Ok(())
}

fn interactive_start() -> Result<(), Box<dyn std::error::Error>> {
    let loc = prompt_install_dir()?;
    postgres::run_start(&loc)
}

fn interactive_stop() -> Result<(), Box<dyn std::error::Error>> {
    let loc = prompt_install_dir()?;
    postgres::run_stop(&loc, "fast")
}

fn interactive_status() -> Result<(), Box<dyn std::error::Error>> {
    let loc = prompt_install_dir()?;
    postgres::run_status(&loc)
}

fn interactive_psql() -> Result<(), Box<dyn std::error::Error>> {
    let loc = prompt_install_dir()?;
    let db = prompt("Database name to connect", Some("postgres"))?;
    postgres::run_psql(&loc, Some(db))
}

fn interactive_add_path() -> Result<(), Box<dyn std::error::Error>> {
    let loc = prompt_install_dir()?;
    let bin_dir = loc.join("bin").to_string_lossy().to_string();
    match add_to_user_path(&bin_dir) {
        Ok(true) => println!("{} Added {} to User PATH.", "[✓]".green(), bin_dir),
        Ok(false) => println!("{} Already present in User PATH.", "[!]".yellow()),
        Err(e) => eprintln!("{} Failed to update User PATH: {}", "[!]".red(), e),
    }
    Ok(())
}

fn interactive_env() -> Result<(), Box<dyn std::error::Error>> {
    let loc = prompt_install_dir()?;
    postgres::run_print_env(&loc)
}

fn interactive_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "--- System Diagnostics ---".cyan().bold());
    println!("Default Install Dir: {}", postgres::default_install_dir().display());
    println!("Port 5432 free?     {}", is_port_available(5432));
    println!("Next available port: {}", find_available_port(5432));
    Ok(())
}

fn prompt_install_dir() -> Result<PathBuf, io::Error> {
    let default_dir = postgres::default_install_dir();
    let default_str = default_dir.to_string_lossy().to_string();
    println!("\nTarget Installation Location:");
    println!("  [1] Default ({})", default_str.bright_black());
    println!("  [2] Current Directory (.\\pgsql)");
    println!("  [3] Custom Location");
    let c = prompt("Select location [1-3]", Some("1"))?;
    let path = match c.as_str() {
        "2" => PathBuf::from(".\\pgsql"),
        "3" => {
            let custom = prompt("Enter directory path", Some(&default_str))?;
            PathBuf::from(custom)
        }
        _ => default_dir,
    };
    Ok(path)
}
