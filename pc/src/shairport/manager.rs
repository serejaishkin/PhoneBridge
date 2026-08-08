use std::process::{Command, Child};
use std::sync::{Arc, Mutex};

pub struct ShairportManager {
    child: Arc<Mutex<Option<Child>>>,
    platform: Platform,
}

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

impl ShairportManager {
    pub fn new() -> Self {
        let platform = if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else {
            Platform::Linux
        };

        Self {
            child: Arc::new(Mutex::new(None)),
            platform,
        }
    }

    pub fn detect_receiver(&self) -> Option<AirPlayReceiver> {
        match self.platform {
            Platform::Linux | Platform::MacOS => {
                if Self::cmd_exists("shairport-sync") {
                    Some(AirPlayReceiver::ShairportSync)
                } else { None }
            }
            Platform::Windows => {
                if Self::cmd_exists("ShairportQt") || Self::file_exists(r"C:\\Program Files\\ShairportQt\\ShairportQt.exe") {
                    Some(AirPlayReceiver::ShairportQt)
                } else if Self::cmd_exists("shairport4w") {
                    Some(AirPlayReceiver::Shairport4w)
                } else { None }
            }
        }
    }

    pub fn start(&self) -> Result<(), String> {
        let receiver = self.detect_receiver().ok_or("No AirPlay receiver found")?;
        let mut child_lock = self.child.lock().unwrap();
        if child_lock.is_some() {
            return Err("Already running".to_string());
        }

        let child = match receiver {
            AirPlayReceiver::ShairportSync => {
                Command::new("shairport-sync")
                    .args(&["--name=PhoneBridge", "--port=5002", "--output=pipe"])
                    .spawn()
                    .map_err(|e| format!("Failed: {}", e))?
            }
            AirPlayReceiver::ShairportQt => {
                let path = if Self::cmd_exists("ShairportQt") {
                    "ShairportQt".to_string()
                } else {
                    r"C:\\Program Files\\ShairportQt\\ShairportQt.exe".to_string()
                };
                Command::new(&path)
                    .args(&["--name", "PhoneBridge", "--port", "5002"])
                    .spawn()
                    .map_err(|e| format!("Failed: {}", e))?
            }
            AirPlayReceiver::Shairport4w => {
                Command::new("shairport4w")
                    .args(&["/name:PhoneBridge", "/port:5002"])
                    .spawn()
                    .map_err(|e| format!("Failed: {}", e))?
            }
        };

        *child_lock = Some(child);
        log::info!("AirPlay receiver '{}' started as 'PhoneBridge'", receiver.name());
        Ok(())
    }

    pub fn stop(&self) {
        let mut child_lock = self.child.lock().unwrap();
        if let Some(mut child) = child_lock.take() {
            let _ = child.kill();
            log::info!("AirPlay receiver stopped");
        }
    }

    pub fn is_running(&self) -> bool {
        self.child.lock().unwrap().is_some()
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    fn cmd_exists(cmd: &str) -> bool {
        Command::new("which").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
            || Command::new("where").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
    }

    fn file_exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

impl Drop for ShairportManager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AirPlayReceiver {
    ShairportSync,
    ShairportQt,
    Shairport4w,
}

impl AirPlayReceiver {
    pub fn name(&self) -> &'static str {
        match self {
            AirPlayReceiver::ShairportSync => "shairport-sync",
            AirPlayReceiver::ShairportQt => "ShairportQt",
            AirPlayReceiver::Shairport4w => "shairport4w",
        }
    }
}
