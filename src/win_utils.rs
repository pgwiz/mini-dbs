use std::net::TcpListener;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

/// Check if a local TCP port is currently free to bind
pub fn is_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Find the next available TCP port starting from `start_port`
pub fn find_available_port(start_port: u16) -> u16 {
    let mut port = start_port;
    while !is_port_available(port) {
        port += 1;
        if port > 65530 {
            return start_port;
        }
    }
    port
}

/// Add a directory to the current user's PATH environment variable without admin rights
pub fn add_to_user_path(bin_dir: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    let current_path: String = env.get_value("Path").unwrap_or_default();
    let parts: Vec<&str> = current_path.split(';').map(|s| s.trim()).collect();

    // Check if already in PATH (case-insensitive on Windows)
    if parts.iter().any(|&p| p.eq_ignore_ascii_case(bin_dir)) {
        return Ok(false);
    }

    let new_path = if current_path.is_empty() {
        bin_dir.to_string()
    } else if current_path.ends_with(';') {
        format!("{}{}", current_path, bin_dir)
    } else {
        format!("{};{}", current_path, bin_dir)
    };

    env.set_value("Path", &new_path)?;

    // Notify the system about environment change (WM_SETTINGCHANGE)
    // Note: Standard for Windows desktop apps
    Ok(true)
}

/// Retrieve the current user PATH
pub fn get_user_path() -> Result<String, Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ)?;
    let current_path: String = env.get_value("Path").unwrap_or_default();
    Ok(current_path)
}
