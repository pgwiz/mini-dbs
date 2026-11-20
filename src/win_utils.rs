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
#[repr(C)]
#[allow(non_snake_case)]
struct ProcessMemoryCounters {
    cb: u32,
    PageFaultCount: u32,
    PeakWorkingSetSize: usize,
    WorkingSetSize: usize,
    QuotaPeakPagedPoolUsage: usize,
    QuotaPagedPoolUsage: usize,
    QuotaPeakNonPagedPoolUsage: usize,
    QuotaNonPagedPoolUsage: usize,
    PagefileUsage: usize,
    PeakPagefileUsage: usize,
}

extern "system" {
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut std::ffi::c_void;
    fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    fn K32GetProcessMemoryInfo(
        hProcess: *mut std::ffi::c_void,
        ppmc: *mut ProcessMemoryCounters,
        cb: u32,
    ) -> i32;
    #[allow(dead_code)]
    fn TerminateProcess(hProcess: *mut std::ffi::c_void, uExitCode: u32) -> i32;
}

const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
const PROCESS_VM_READ: u32 = 0x0010;
#[allow(dead_code)]
const PROCESS_TERMINATE: u32 = 0x0001;

/// Query the working set memory usage in MB for a specific process ID
pub fn get_process_memory_mb(pid: u32) -> Option<f64> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle.is_null() {
            return None;
        }

        let mut pmc: ProcessMemoryCounters = std::mem::zeroed();
        pmc.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;

        let ok = K32GetProcessMemoryInfo(handle, &mut pmc, pmc.cb);
        CloseHandle(handle);

        if ok != 0 {
            Some(pmc.WorkingSetSize as f64 / (1024.0 * 1024.0))
        } else {
            None
        }
    }
}

/// Check if a process with the given PID is actively running
pub fn is_process_alive(pid: u32) -> bool {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
        if handle.is_null() {
            false
        } else {
            CloseHandle(handle);
            true
        }
    }
}

/// Forcibly terminate a process by PID
#[allow(dead_code)]
pub fn terminate_process_by_pid(pid: u32) -> bool {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return false;
        }
        let res = TerminateProcess(handle, 1);
        CloseHandle(handle);
        res != 0
    }
}
