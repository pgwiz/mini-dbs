use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::config::{InstanceRegistry, PgConfig};
use crate::postgres;
use crate::win_utils::{get_process_memory_mb, is_process_alive};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InstanceStatus {
    pub name: String,
    pub install_dir: PathBuf,
    pub data_dir: PathBuf,
    pub port: u16,
    pub pid: Option<u32>,
    pub memory_mb: Option<f64>,
    pub is_running: bool,
    pub state_label: String,
}

/// Collect status and memory metrics for all known mini-dbs database instances
pub fn collect_all_statuses() -> Vec<InstanceStatus> {
    let registry = InstanceRegistry::load();
    let mut dirs_to_check: Vec<PathBuf> = Vec::new();

    // 1. Default AppData directory
    let default_appdata = postgres::default_install_dir();
    dirs_to_check.push(default_appdata);

    // 2. Current project directory ./pgsql
    let current_pgsql = PathBuf::from(".\\pgsql");
    if current_pgsql.exists() && !dirs_to_check.contains(&current_pgsql) {
        dirs_to_check.push(current_pgsql);
    }

    // 3. Registered custom directories
    for inst in registry.instances {
        let p = PathBuf::from(inst);
        if !dirs_to_check.contains(&p) {
            dirs_to_check.push(p);
        }
    }

    let mut statuses = Vec::new();

    for dir in dirs_to_check {
        if !dir.exists() {
            continue;
        }

        let cfg = PgConfig::load(&dir).ok();
        let default_port = cfg.as_ref().map(|c| c.port).unwrap_or(5432);
        let data_dir = cfg.as_ref().map(|c| PathBuf::from(&c.data_dir)).unwrap_or_else(|| dir.join("data"));

        let pid_file = data_dir.join("postmaster.pid");
        let mut active_pid = None;
        let mut port = default_port;
        let mut is_running = false;
        let mut state_label = "STOPPED".bright_black().to_string();
        let mut memory_mb = None;

        if pid_file.exists() {
            if let Ok(content) = fs::read_to_string(&pid_file) {
                let lines: Vec<&str> = content.lines().collect();
                if let Some(first_line) = lines.first() {
                    if let Ok(pid) = first_line.trim().parse::<u32>() {
                        if is_process_alive(pid) {
                            is_running = true;
                            active_pid = Some(pid);
                            memory_mb = get_process_memory_mb(pid);
                            state_label = "RUNNING".green().bold().to_string();

                            // Read actual listening port from postmaster.pid line 4 if present
                            if lines.len() >= 4 {
                                if let Ok(p) = lines[3].trim().parse::<u16>() {
                                    port = p;
                                }
                            }
                        } else {
                            state_label = "STALE (DEAD)".yellow().to_string();
                        }
                    }
                }
            }
        }

        let name = dir.file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("postgres")
            .to_string();

        statuses.push(InstanceStatus {
            name,
            install_dir: dir,
            data_dir,
            port,
            pid: active_pid,
            memory_mb,
            is_running,
            state_label,
        });
    }

    statuses
}

/// Print formatted table of active processes and their real-time memory footprint
pub fn print_process_table() {
    let statuses = collect_all_statuses();

    println!("\n{}", "=========================================================================================".cyan());
    println!("{}", "               mini-dbs: Memory-Efficient Database Process Activity                      ".cyan().bold());
    println!("{}", "=========================================================================================".cyan());
    println!(
        "{:<8} {:<12} {:<7} {:<15} {:<14} {:<32}",
        "PID".bright_white().bold(),
        "ENGINE".bright_white().bold(),
        "PORT".bright_white().bold(),
        "STATUS".bright_white().bold(),
        "RAM USAGE".bright_white().bold(),
        "INSTALL DIRECTORY".bright_white().bold()
    );
    println!("{}", "-----------------------------------------------------------------------------------------".bright_black());

    if statuses.is_empty() {
        println!("  No configured database instances found.");
    } else {
        let mut running_count = 0;
        let mut total_ram = 0.0;

        for s in &statuses {
            let pid_str = s.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());
            let ram_str = s.memory_mb
                .map(|mb| {
                    total_ram += mb;
                    format!("{:.1} MB", mb)
                })
                .unwrap_or_else(|| "-".to_string());

            if s.is_running {
                running_count += 1;
            }

            println!(
                "{:<8} {:<12} {:<7} {:<15} {:<14} {:<32}",
                pid_str.white(),
                "PostgreSQL".cyan(),
                s.port.to_string().yellow(),
                s.state_label,
                ram_str.bright_green(),
                s.install_dir.display().to_string().bright_black()
            );
        }

        println!("{}", "-----------------------------------------------------------------------------------------".bright_black());
        println!(
            "Total Active Instances: {} | Total Memory Footprint: {:.1} MB",
            running_count.to_string().green().bold(),
            total_ram.to_string().cyan().bold()
        );
    }
    println!("{}", "=========================================================================================".cyan());
}

/// Stop all active database processes cleanly
pub fn stop_all_processes() -> Result<(), Box<dyn std::error::Error>> {
    let statuses = collect_all_statuses();
    let mut stopped_any = false;

    for s in statuses {
        if s.is_running {
            println!("Stopping instance at {} (Port {})...", s.install_dir.display(), s.port);
            let _ = postgres::run_stop(&s.install_dir, "fast");
            stopped_any = true;
        }
    }

    if !stopped_any {
        println!("No active instances were running.");
    } else {
        println!("{} All active instances stopped.", "[✓]".green());
    }
    Ok(())
}

/// Run live memory-efficient terminal dashboard loop
pub fn run_live_monitor(interval_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting memory-efficient activity supervisor (refresh: {}s)...", interval_secs);
    println!("Press Ctrl+C to exit monitor.\n");

    loop {
        // Clear terminal screen smoothly
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush()?;

        print_process_table();
        println!("\n[Live Monitor] Sleeping {}s... (Minimal CPU/RAM activity)", interval_secs);
        thread::sleep(Duration::from_secs(interval_secs));
    }
}
