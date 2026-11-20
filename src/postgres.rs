use colored::Colorize;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::PgConfig;
use crate::downloader::{download_file, extract_zip};
use crate::win_utils::{add_to_user_path, find_available_port, is_port_available};

pub fn default_install_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".mini-dbs")
        .join("postgres")
}

pub struct SetupOptions {
    pub version: String,
    pub install_dir: PathBuf,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub add_to_path: bool,
    pub force: bool,
}

pub fn run_setup(opts: SetupOptions) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "==========================================================".cyan());
    println!("{}", "  mini-dbs: Zero-Admin PostgreSQL CLI Setup for Windows   ".cyan().bold());
    println!("{}", "==========================================================".cyan());
    println!("{}:     {}", "Target Version".bright_black(), opts.version.white());
    println!("{}:  {}", "Install Directory".bright_black(), opts.install_dir.display().to_string().white());
    println!("{}:    {}", "PostgreSQL User".bright_black(), opts.username.white());

    let bin_dir = opts.install_dir.join("bin");
    let postgres_exe = bin_dir.join("postgres.exe");

    if postgres_exe.exists() && !opts.force {
        println!("\n{} PostgreSQL is already installed at: {}", "[!]".yellow(), opts.install_dir.display());
        println!("Pass --force to reinstall or overwrite.");
        return Ok(());
    }

    // Determine port
    let target_port = if !is_port_available(opts.port) {
        let alt = find_available_port(opts.port + 1);
        println!(
            "\n{} Port {} is in use. Automatically selecting free port: {}",
            "[!]".yellow(),
            opts.port,
            alt.to_string().green().bold()
        );
        alt
    } else {
        println!("{}:        {} (Available)", "Target Port".bright_black(), opts.port.to_string().green());
        opts.port
    };

    fs::create_dir_all(&opts.install_dir)?;

    // Download
    let temp_dir = env::temp_dir();
    let zip_filename = format!("postgresql-{}-windows-x64-binaries.zip", opts.version);
    let temp_zip = temp_dir.join(&zip_filename);
    let download_url = format!(
        "https://get.enterprisedb.com/postgresql/postgresql-{}-windows-x64-binaries.zip",
        opts.version
    );

    println!("\n{}", "[1/4] Fetching official PostgreSQL binaries...".cyan());
    if temp_zip.exists() {
        println!("{} Using cached archive in TEMP: {}", "[✓]".green(), temp_zip.display());
    } else {
        println!("Downloading from: {}", download_url.bright_black());
        download_file(&download_url, &temp_zip)?;
        println!("{} Download finished.", "[✓]".green());
    }

    // Extract
    println!("\n{}", "[2/4] Extracting binaries...".cyan());
    extract_zip(&temp_zip, &opts.install_dir)?;
    println!("{} Extracted to {}", "[✓]".green(), opts.install_dir.display());

    // Initdb
    let data_dir = opts.install_dir.join("data");
    let initdb_exe = bin_dir.join("initdb.exe");
    println!("\n{}", "[3/4] Initializing database cluster with initdb...".cyan());

    if !initdb_exe.exists() {
        return Err(format!("initdb.exe not found at {}", initdb_exe.display()).into());
    }

    if !data_dir.join("PG_VERSION").exists() {
        let mut cmd = Command::new(&initdb_exe);
        cmd.arg("-D").arg(&data_dir)
            .arg("-U").arg(&opts.username)
            .arg("-E").arg("UTF8")
            .arg("--locale=C");

        let mut pw_file_path = None;
        if let Some(pw) = &opts.password {
            let tmp_pw = temp_dir.join(format!("pg_pw_{}.tmp", uuid_short()));
            fs::write(&tmp_pw, pw)?;
            cmd.arg("-A").arg("scram-sha-256").arg(format!("--pwfile={}", tmp_pw.display()));
            pw_file_path = Some(tmp_pw);
        } else {
            cmd.arg("-A").arg("trust");
            println!("Authentication: trust (frictionless local development)");
        }

        let status = cmd.status()?;
        if let Some(p) = pw_file_path {
            let _ = fs::remove_file(p);
        }

        if !status.success() {
            return Err("initdb command failed".into());
        }

        if target_port != 5432 {
            let conf_path = data_dir.join("postgresql.conf");
            let mut conf_file = OpenOptions::new().append(true).open(&conf_path)?;
            writeln!(conf_file, "\n# Configured by mini-dbs\nport = {}", target_port)?;
        }

        println!("{} Database cluster initialized successfully.", "[✓]".green());
    } else {
        println!("{} Existing database cluster found in data/. Skipping initdb.", "[!]".yellow());
    }

    // Save mini-dbs.json
    println!("\n{}", "[4/4] Writing configuration and scripts...".cyan());
    let cfg = PgConfig {
        version: opts.version.clone(),
        install_dir: opts.install_dir.to_string_lossy().to_string(),
        data_dir: data_dir.to_string_lossy().to_string(),
        port: target_port,
        username: opts.username.clone(),
        installed_at: chrono_like_now(),
    };
    cfg.save(&opts.install_dir)?;

    // Generate start/stop/status batch shortcuts
    write_shortcuts(&opts.install_dir, &data_dir, target_port, &opts.username)?;

    // Register User PATH if requested
    if opts.add_to_path {
        println!("\nRegistering bin directory to User PATH (zero admin)...");
        let bin_str = bin_dir.to_string_lossy().to_string();
        match add_to_user_path(&bin_str) {
            Ok(true) => println!("{} Added {} to User PATH.", "[✓]".green(), bin_str),
            Ok(false) => println!("{} Already present in User PATH.", "[!]".yellow()),
            Err(e) => eprintln!("{} Failed to update User PATH: {}", "[!]".red(), e),
        }
    }

    println!("\n{}", "==========================================================".green());
    println!("{}", "  PostgreSQL installed successfully! (Zero Admin Required) ".green().bold());
    println!("{}", "==========================================================".green());
    println!("{}:       {}", "Binaries", bin_dir.display());
    println!("{}:   {}", "Data Cluster", data_dir.display());
    println!("{}:           {}", "Port", target_port);
    println!("{}:           {}", "User", opts.username);
    println!("\n{}", "Management with mini-dbs:".cyan());
    println!("  mini-dbs pg start     - Start background server");
    println!("  mini-dbs pg status    - Check server status");
    println!("  mini-dbs pg psql      - Open interactive CLI");
    println!("  mini-dbs pg stop      - Stop server cleanly");
    println!("{}", "==========================================================".green());

    Ok(())
}

pub fn run_start(install_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let pg_ctl = install_dir.join("bin").join("pg_ctl.exe");
    let data_dir = install_dir.join("data");
    let log_file = data_dir.join("logfile");

    if !pg_ctl.exists() {
        return Err(format!("pg_ctl.exe not found at {}. Have you run 'setup'?", pg_ctl.display()).into());
    }

    println!("{} Starting PostgreSQL server...", "[*]".cyan());
    let mut cmd = Command::new(&pg_ctl);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // DETACHED_PROCESS (0x08) | CREATE_NEW_PROCESS_GROUP (0x200)
        cmd.creation_flags(0x00000008 | 0x00000200);
    }

    let status = cmd
        .arg("-D").arg(&data_dir)
        .arg("-l").arg(&log_file)
        .arg("start")
        .status()?;

    if status.success() {
        println!("{} PostgreSQL server started.", "[✓]".green());
        if let Ok(cfg) = PgConfig::load(install_dir) {
            println!("   Port: {}", cfg.port);
            println!("   User: {}", cfg.username);
        }
    } else {
        eprintln!("{} Failed to start server. Check log at: {}", "[!]".red(), log_file.display());
    }
    Ok(())
}

pub fn run_stop(install_dir: &Path, mode: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pg_ctl = install_dir.join("bin").join("pg_ctl.exe");
    let data_dir = install_dir.join("data");

    if !pg_ctl.exists() {
        return Err(format!("pg_ctl.exe not found at {}", pg_ctl.display()).into());
    }

    println!("{} Stopping PostgreSQL server (mode: {})...", "[*]".cyan(), mode);
    let status = Command::new(&pg_ctl)
        .arg("-D").arg(&data_dir)
        .arg("-m").arg(mode)
        .arg("stop")
        .status()?;

    if status.success() {
        println!("{} PostgreSQL server stopped.", "[✓]".green());
    } else {
        eprintln!("{} Server was not running or could not be stopped.", "[!]".yellow());
    }
    Ok(())
}

pub fn run_restart(install_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let pg_ctl = install_dir.join("bin").join("pg_ctl.exe");
    let data_dir = install_dir.join("data");
    let log_file = data_dir.join("logfile");

    if !pg_ctl.exists() {
        return Err(format!("pg_ctl.exe not found at {}", pg_ctl.display()).into());
    }

    println!("{} Restarting PostgreSQL server...", "[*]".cyan());
    let mut cmd = Command::new(&pg_ctl);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x00000008 | 0x00000200);
    }

    let status = cmd
        .arg("-D").arg(&data_dir)
        .arg("-l").arg(&log_file)
        .arg("-m").arg("fast")
        .arg("restart")
        .status()?;

    if status.success() {
        println!("{} PostgreSQL server restarted.", "[✓]".green());
    }
    Ok(())
}

pub fn run_status(install_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let pg_ctl = install_dir.join("bin").join("pg_ctl.exe");
    let data_dir = install_dir.join("data");

    if !pg_ctl.exists() {
        return Err(format!("pg_ctl.exe not found at {}", pg_ctl.display()).into());
    }

    println!("{} Checking PostgreSQL status...", "[*]".cyan());
    let _ = Command::new(&pg_ctl)
        .arg("-D").arg(&data_dir)
        .arg("status")
        .status()?;

    if let Ok(cfg) = PgConfig::load(install_dir) {
        if !is_port_available(cfg.port) {
            println!("{} Port {} is actively listening.", "[✓]".green(), cfg.port);
        } else {
            println!("{} Port {} is not listening.", "[-]".yellow(), cfg.port);
        }
    }
    Ok(())
}

pub fn run_psql(install_dir: &Path, db: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let psql_exe = install_dir.join("bin").join("psql.exe");
    if !psql_exe.exists() {
        return Err(format!("psql.exe not found at {}", psql_exe.display()).into());
    }

    let cfg = PgConfig::load(install_dir).unwrap_or(PgConfig {
        version: "unknown".into(),
        install_dir: install_dir.to_string_lossy().into(),
        data_dir: install_dir.join("data").to_string_lossy().into(),
        port: 5432,
        username: "postgres".into(),
        installed_at: "".into(),
    });

    let target_db = db.unwrap_or_else(|| "postgres".to_string());
    println!(
        "{} Connecting to {}@127.0.0.1:{}/{}...",
        "[*]".cyan(),
        cfg.username,
        cfg.port,
        target_db
    );

    let status = Command::new(&psql_exe)
        .arg("-h").arg("127.0.0.1")
        .arg("-p").arg(cfg.port.to_string())
        .arg("-U").arg(&cfg.username)
        .arg(&target_db)
        .env("PGCLIENTENCODING", "utf8")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        eprintln!("psql exited with code: {:?}", status.code());
    }
    Ok(())
}

pub fn run_print_env(install_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let bin_dir = install_dir.join("bin");
    let data_dir = install_dir.join("data");
    let cfg = PgConfig::load(install_dir).ok();
    let port = cfg.as_ref().map(|c| c.port).unwrap_or(5432);
    let user = cfg.as_ref().map(|c| c.username.as_str()).unwrap_or("postgres");

    println!("# PowerShell Environment Activation:");
    println!("$env:PATH = \"{};$env:PATH\"", bin_dir.display());
    println!("$env:PGDATA = \"{}\"", data_dir.display());
    println!("$env:PGPORT = \"{}\"", port);
    println!("$env:PGUSER = \"{}\"", user);
    println!("$env:PGCLIENTENCODING = \"utf8\"");
    println!("\n# Command Prompt (cmd.exe) Activation:");
    println!("set \"PATH={};%PATH%\"", bin_dir.display());
    println!("set \"PGDATA={}\"", data_dir.display());
    println!("set \"PGPORT={}\"", port);
    println!("set \"PGUSER={}\"", user);
    println!("set \"PGCLIENTENCODING=utf8\"");

    Ok(())
}

fn write_shortcuts(install_dir: &Path, data_dir: &Path, port: u16, username: &str) -> std::io::Result<()> {
    let start_bat = format!(
        "@echo off\nset \"PGDATA={}\"\n\"{}\" -D \"%PGDATA%\" -l \"%PGDATA%\\logfile\" start\necho PostgreSQL started on port {}.\n",
        data_dir.display(),
        install_dir.join("bin").join("pg_ctl.exe").display(),
        port
    );
    fs::write(install_dir.join("start.bat"), start_bat)?;

    let stop_bat = format!(
        "@echo off\nset \"PGDATA={}\"\n\"{}\" -D \"%PGDATA%\" -m fast stop\necho PostgreSQL stopped.\n",
        data_dir.display(),
        install_dir.join("bin").join("pg_ctl.exe").display()
    );
    fs::write(install_dir.join("stop.bat"), stop_bat)?;

    let status_bat = format!(
        "@echo off\nset \"PGDATA={}\"\n\"{}\" -D \"%PGDATA%\" status\n",
        data_dir.display(),
        install_dir.join("bin").join("pg_ctl.exe").display()
    );
    fs::write(install_dir.join("status.bat"), status_bat)?;

    let psql_bat = format!(
        "@echo off\nchcp 65001 > nul\n\"{}\" -h 127.0.0.1 -p {} -U {} postgres\n",
        install_dir.join("bin").join("psql.exe").display(),
        port,
        username
    );
    fs::write(install_dir.join("psql.bat"), psql_bat)?;

    Ok(())
}

fn uuid_short() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:x}", d.as_nanos())
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    format!("timestamp-{}", secs)
}
pub fn list_extensions(install_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ext_dir = install_dir.join("share").join("extension");
    if !ext_dir.exists() {
        return Err(format!("Extension directory not found at {}. Have you run 'setup'?", ext_dir.display()).into());
    }

    println!("\n{}", "--- Available PostgreSQL Extensions (Zero-Admin) ---".cyan().bold());
    println!("Extension Directory: {}\n", ext_dir.display());

    let mut extensions = Vec::new();
    for entry in fs::read_dir(&ext_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "control") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let content = fs::read_to_string(&path).unwrap_or_default();
                let comment = content.lines()
                    .find(|line| line.trim().starts_with("comment"))
                    .and_then(|line| line.split('=').nth(1))
                    .map(|c| c.trim().trim_matches('\''))
                    .unwrap_or("No description available");
                extensions.push((stem.to_string(), comment.to_string()));
            }
        }
    }

    extensions.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, desc) in &extensions {
        println!("  • {:<24} {}", name.green().bold(), desc.bright_black());
    }

    println!("\nTotal available extensions: {}", extensions.len().to_string().cyan());
    println!("To enable any extension in your database, connect with psql and run:");
    println!("  {}", "CREATE EXTENSION <extension_name>;".yellow());
    Ok(())
}

pub fn install_extension_zip(install_dir: &Path, zip_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(zip_file)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))?;

    let lib_dir = install_dir.join("lib");
    let ext_dir = install_dir.join("share").join("extension");
    let bin_dir = install_dir.join("bin");

    fs::create_dir_all(&lib_dir)?;
    fs::create_dir_all(&ext_dir)?;

    println!("{} Installing extension from {}...", "[*]".cyan(), zip_file.display());
    let mut installed_files = 0;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        if entry.is_dir() {
            continue;
        }

        let file_name = Path::new(&name).file_name().and_then(|f| f.to_str()).unwrap_or("");
        if file_name.is_empty() {
            continue;
        }

        let target_path = if file_name.ends_with(".dll") {
            lib_dir.join(file_name)
        } else if file_name.ends_with(".control") || file_name.ends_with(".sql") {
            ext_dir.join(file_name)
        } else {
            bin_dir.join(file_name)
        };

        let mut out = fs::File::create(&target_path)?;
        std::io::copy(&mut entry, &mut out)?;
        println!("  + Extracted: {}", target_path.display());
        installed_files += 1;
    }

    println!("{} Extension installed successfully ({} files placed into user directory).", "[✓]".green(), installed_files);
    println!("You can now enable it via psql:");
    println!("  {}", "CREATE EXTENSION <extension_name>;".yellow());
    Ok(())
}
