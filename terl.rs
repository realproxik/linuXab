use std::process::Stdio;
use tokio::process::{Command, Child};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::collections::HashMap;

pub struct TerminalSession {
    cwd: String,
    env: HashMap<String, String>,
    history: Vec<String>,
}

impl TerminalSession {
    pub fn new() -> Self {
        Self {
            cwd: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/".to_string()),
            env: std::env::vars().collect(),
            history: Vec::new(),
        }
    }
    
    pub async fn execute(&mut self, cmd: &str) -> String {
        let cmd = cmd.trim();
        if cmd.is_empty() {
            return String::new();
        }
        
        self.history.push(cmd.to_string());
        
        // Handle built-in commands
        if let Some(output) = self.handle_builtin(cmd).await {
            return output;
        }
        
        // Execute real bash command
        self.run_shell_command(cmd).await
    }
    
    async fn handle_builtin(&mut self, cmd: &str) -> Option<String> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        
        match parts[0] {
            "cd" => {
                let new_dir = parts.get(1).unwrap_or(&"~");
                let path = if new_dir.starts_with('/') {
                    new_dir.to_string()
                } else if new_dir == &"~" {
                    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
                } else {
                    format!("{}/{}", self.cwd, new_dir)
                };
                
                if std::path::Path::new(&path).exists() {
                    self.cwd = std::fs::canonicalize(&path)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or(path);
                    Some(String::new())
                } else {
                    Some(format!("bash: cd: {}: No such file or directory\\n", new_dir))
                }
            }
            "exit" | "logout" => {
                Some("\\x1b[33m[Session closed]\\x1b[0m\\n".to_string())
            }
            "fastfetch" | "quickfetch" | "ff" => {
                Some(crate::fastfetch::generate_fastfetch().await)
            }
            "clear" | "cls" => {
                Some("\\x1b[2J\\x1b[H".to_string())
            }
            "help" => {
                Some(self.help_text())
            }
            _ => None,
        }
    }
    
    async fn run_shell_command(&mut self, cmd: &str) -> String {
        let mut child = match Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .current_dir(&self.cwd)
            .envs(&self.env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return format!("bash: error spawning shell: {}\\n", e),
        };
        
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        
        let mut out_buf = String::new();
        let mut err_buf = String::new();
        
        let (out_res, err_res) = tokio::join!(
            stdout.read_to_string(&mut out_buf),
            stderr.read_to_string(&mut err_buf)
        );
        
        let _ = child.wait().await;
        
        let mut result = String::new();
        if out_res.is_ok() && !out_buf.is_empty() {
            result.push_str(&out_buf);
        }
        if err_res.is_ok() && !err_buf.is_empty() {
            result.push_str(&err_buf);
        }
        
        if result.is_empty() {
            result.push('\\n');
        }
        
        result
    }
    
    fn help_text(&self) -> String {
        r#"\\x1b36m╔══════════════════════════════════════════════════════════════╗
║           🐧 Linuxab OS Terminal - Help                      ║
╠══════════════════════════════════════════════════════════════╣
║ Built-in commands:                                           ║
║   fastfetch, quickfetch, ff  - System info with penguin logo ║
║   cd <dir>                   - Change directory               ║
║   clear, cls                 - Clear screen                   ║
║   exit, logout               - Close session                  ║
║   help                       - Show this help                 ║
║                                                              ║
║ Any other command is passed to bash.                         ║
║ Try: ls, ps, top, nano, gcc, python3, etc.                   ║
╚══════════════════════════════════════════════════════════════╝\\x1b[0m
"#.to_string()
    }
}
