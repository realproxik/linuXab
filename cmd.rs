use sysinfo::{System, SystemExt, get_current_pid, ProcessExt};
use chrono::Local;
use std::fs;

const LINUXAB_LOGO: &str = r#"
        .-.
       (o o)    🐧 Linuxab OS
       | O \\     The Penguin Power
        \\   \\    
         `~~~'   
        /|_|\\
       / |_| \\
      /__|_|__\\
"#;

const COLOR_LOGO: &str = r#"
\\x1b[36m        .-.
\\x1b[36m       \\x1b[37m(\\x1b[33mo o\\x1b[37m)\\x1b[36m    🐧 Linuxab OS
\\x1b[36m       \\x1b[37m| \\x1b[33mO \\x1b[37m\\  \\x1b[36m   The Penguin Power
\\x1b[36m        \\x1b[37m\\   \\  \\x1b[36m  
\\x1b[36m         `~~~'\\x1b[36m   
\\x1b[36m        /|_|\\
\\x1b[36m       / |_| \\
\\x1b[36m      /__|_|__\\
\\x1b[0m"#;

pub async fn generate_banner() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let hostname = get_hostname();
    let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
    let os_name = get_os_name();
    let kernel = get_kernel_version();
    let uptime = format_uptime();
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    let term = std::env::var("TERM").unwrap_or_else(|_| "linuxab-terminal".to_string());
    let cpu = get_cpu_info(&sys);
    let memory = get_memory_info(&sys);
    let resolution = "1920x1080 @ 60Hz";
    
    format!(r#"
\\x1b[35m{user}\\x1b[0m@\\x1b[35m{hostname}\\x1b[0m
{COLOR_LOGO}
\\x1b[35mOS\\x1b[0m: {os_name} x86_64
\\x1b[35mKernel\\x1b[0m: {kernel}
\\x1b[35mUptime\\x1b[0m: {uptime}
\\x1b[35mShell\\x1b[0m: {shell}
\\x1b[35mTerminal\\x1b[0m: {term}
\\x1b[35mCPU\\x1b[0m: {cpu}
\\x1b[35mMemory\\x1b[0m: {memory}
\\x1b[35mResolution\\x1b[0m: {resolution}
\\x1b[35mLocale\\x1b[0m: C.UTF-8

\\x1b[30m██\\x1b[31m██\\x1b[32m██\\x1b[33m██\\x1b[34m██\\x1b[35m██\\x1b[36m██\\x1b[37m██\\x1b[0m
\\x1b[90m██\\x1b[91m██\\x1b[92m██\\x1b[93m██\\x1b[94m██\\x1b[95m██\\x1b[96m██\\x1b[97m██\\x1b[0m
"#)
}

pub async fn generate_fastfetch() -> String {
    generate_banner().await
}

pub async fn get_sysinfo_json() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let hostname = get_hostname();
    let os = get_os_name();
    let kernel = get_kernel_version();
    let uptime = format_uptime();
    let cpu = get_cpu_info(&sys);
    let memory = get_memory_info(&sys);
    
    format!(r#"{{"hostname":"{hostname}","os":"{os}","kernel":"{kernel}","uptime":"{uptime}","cpu":"{cpu}","memory":"{memory}"}}"#)
}

fn get_hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "linuxab".to_string())
        .trim()
        .to_string()
}

fn get_os_name() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line.trim_start_matches("PRETTY_NAME=\"").trim_end_matches('\"').to_string();
            }
        }
    }
    "Linuxab OS 0.2.0".to_string()
}

fn get_kernel_version() -> String {
    unsafe {
        let mut buf = [0u8; 256];
        if libc::uname(buf.as_mut_ptr() as *mut libc::utsname) == 0 {
            let release = std::ffi::CStr::from_ptr(buf.as_ptr() as *const i8)
                .to_string_lossy()
                .to_string();
            if !release.is_empty() {
                return format!("Linuxab {}", release);
            }
        }
    }
    "Linuxab 0.2.0-default".to_string()
}

fn format_uptime() -> String {
    if let Ok(content) = fs::read_to_string("/proc/uptime") {
        if let Some(secs_str) = content.split_whitespace().next() {
            if let Ok(secs) = secs_str.parse::<f64>() {
                let days = (secs / 86400.0) as u64;
                let hours = ((secs % 86400.0) / 3600.0) as u64;
                let mins = ((secs % 3600.0) / 60.0) as u64;
                
                if days > 0 {
                    return format!("{} days, {} hours, {} mins", days, hours, mins);
                } else if hours > 0 {
                    return format!("{} hours, {} mins", hours, mins);
                } else {
                    return format!("{} mins", mins);
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_cpu_info(sys: &System) -> String {
    if let Some(cpu) = sys.cpus().first() {
        format!("{} @ {:.2}GHz", cpu.brand(), cpu.frequency() as f64 / 1000.0)
    } else {
        "Unknown CPU".to_string()
    }
}

fn get_memory_info(sys: &System) -> String {
    let total = sys.total_memory() / 1024 / 1024;
    let used = sys.used_memory() / 1024 / 1024;
    format!("{}MiB / {}MiB ({:.1}%)", used, total, (used as f64 / total as f64) * 100.0)
}
