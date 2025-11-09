//! Browser launcher for local browser instances

use crate::error::{BrowserUseError, Result};
use crate::browser::profile::BrowserProfile;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

/// Browser launcher for managing local browser processes
pub struct BrowserLauncher {
    profile: BrowserProfile,
    executable_path: Option<PathBuf>,
    process: Option<tokio::process::Child>,
}

impl BrowserLauncher {
    pub fn new(profile: BrowserProfile) -> Self {
        Self {
            profile,
            executable_path: None,
            process: None,
        }
    }

    pub fn with_executable_path(mut self, path: PathBuf) -> Self {
        self.executable_path = Some(path);
        self
    }

    /// Find browser executable
    pub async fn find_browser_executable(&self) -> Result<PathBuf> {
        // If custom path provided, use it
        if let Some(ref path) = self.executable_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // Try to find browser in common locations
        let candidates = Self::get_browser_candidates();
        
        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        Err(BrowserUseError::Browser(
            "No browser executable found. Please install Chrome/Chromium or provide executable_path".to_string(),
        ))
    }

    /// Get list of candidate browser paths based on platform
    fn get_browser_candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        
        #[cfg(target_os = "macos")]
        {
            candidates.extend(vec![
                PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
                PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium"),
                PathBuf::from("/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary"),
            ]);
        }
        
        #[cfg(target_os = "linux")]
        {
            candidates.extend(vec![
                PathBuf::from("/usr/bin/google-chrome-stable"),
                PathBuf::from("/usr/bin/google-chrome"),
                PathBuf::from("/usr/bin/chromium"),
                PathBuf::from("/usr/bin/chromium-browser"),
            ]);
        }
        
        #[cfg(target_os = "windows")]
        {
            use std::env;
            let local_app_data = env::var("LOCALAPPDATA").unwrap_or_default();
            let program_files = env::var("PROGRAMFILES").unwrap_or_default();
            let program_files_x86 = env::var("PROGRAMFILES(X86)").unwrap_or_default();
            
            candidates.extend(vec![
                PathBuf::from(format!("{}/Google/Chrome/Application/chrome.exe", program_files)),
                PathBuf::from(format!("{}/Google/Chrome/Application/chrome.exe", program_files_x86)),
                PathBuf::from(format!("{}/Google/Chrome/Application/chrome.exe", local_app_data)),
                PathBuf::from("C:\\Program Files\\Chromium\\Application\\chrome.exe"),
            ]);
        }
        
        candidates
    }

    /// Find a free port for CDP debugging
    fn find_free_port() -> Result<u16> {
        use std::net::TcpListener;
        
        // Try ports starting from 9222 (Chrome's default)
        for port in 9222..9300 {
            if TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
                return Ok(port);
            }
        }
        
        Err(BrowserUseError::Browser("No free port found for CDP".to_string()))
    }

    /// Build launch arguments from profile
    fn build_launch_args(&self, debug_port: u16) -> Vec<String> {
        let mut args = Vec::new();

        // User data directory (required for CDP)
        if let Some(ref user_data_dir) = self.profile.user_data_dir {
            args.push(format!("--user-data-dir={}", user_data_dir.display()));
        } else {
            // Use temp directory if not provided
            let temp_dir = std::env::temp_dir().join("browser-use-tmp");
            args.push(format!("--user-data-dir={}", temp_dir.display()));
        }

        // Headless mode
        if self.profile.headless.unwrap_or(false) {
            args.push("--headless".to_string());
            args.push("--disable-gpu".to_string());
        }

        // Remote debugging port
        args.push(format!("--remote-debugging-port={}", debug_port));

        // Additional common args for automation
        args.extend(vec![
            "--disable-blink-features=AutomationControlled".to_string(),
            "--disable-dev-shm-usage".to_string(),
            "--no-sandbox".to_string(),
            "--disable-setuid-sandbox".to_string(),
        ]);

        args
    }

    /// Launch browser and return CDP URL
    pub async fn launch(&mut self) -> Result<String> {
        // Find browser executable
        let browser_path = self.find_browser_executable().await?;
        
        // Find free port
        let debug_port = Self::find_free_port()?;
        
        // Build launch arguments
        let args = self.build_launch_args(debug_port);
        
        // Launch browser
        let mut command = Command::new(&browser_path);
        command.args(&args);
        command.stdin(Stdio::null());
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
        
        let child = command.spawn()
            .map_err(|e| BrowserUseError::Browser(format!("Failed to launch browser: {}", e)))?;
        
        self.process = Some(child);
        
        // Wait for CDP to be ready
        let cdp_url = format!("http://127.0.0.1:{}", debug_port);
        Self::wait_for_cdp_ready(&cdp_url).await?;
        
        Ok(cdp_url)
    }

    /// Wait for CDP endpoint to be ready
    async fn wait_for_cdp_ready(cdp_url: &str) -> Result<()> {
        let max_attempts = 30;
        let delay = Duration::from_millis(500);
        
        for _ in 0..max_attempts {
            // Try to connect to CDP endpoint
            if let Ok(response) = reqwest::get(format!("{}/json/version", cdp_url)).await {
                if response.status().is_success() {
                    return Ok(());
                }
            }
            
            sleep(delay).await;
        }
        
        Err(BrowserUseError::Browser(
            "CDP endpoint did not become ready in time".to_string(),
        ))
    }

    /// Stop the browser process
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill().await;
        }
        Ok(())
    }
}

impl Drop for BrowserLauncher {
    fn drop(&mut self) {
        // Try to stop browser on drop
        if let Some(ref mut process) = self.process {
            let _ = process.kill();
        }
    }
}

