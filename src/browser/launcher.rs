//! Browser launcher for local browser instances

use crate::browser::profile::BrowserProfile;
use crate::error::{BrowsingError, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{Duration, sleep};

/// Browser launcher for managing local browser processes
pub struct BrowserLauncher {
    profile: BrowserProfile,
    executable_path: Option<PathBuf>,
    process: Option<tokio::process::Child>,
}

impl BrowserLauncher {
    /// Creates a new BrowserLauncher with the given profile
    pub fn new(profile: BrowserProfile) -> Self {
        Self {
            profile,
            executable_path: None,
            process: None,
        }
    }

    /// Sets the executable path for the browser
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

        Err(BrowsingError::Browser(
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
                PathBuf::from(
                    "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
                ),
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
                PathBuf::from(format!(
                    "{}/Google/Chrome/Application/chrome.exe",
                    program_files
                )),
                PathBuf::from(format!(
                    "{}/Google/Chrome/Application/chrome.exe",
                    program_files_x86
                )),
                PathBuf::from(format!(
                    "{}/Google/Chrome/Application/chrome.exe",
                    local_app_data
                )),
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
            if TcpListener::bind(format!("127.0.0.1:{port}")).is_ok() {
                return Ok(port);
            }
        }

        Err(BrowsingError::Browser(
            "No free port found for CDP".to_string(),
        ))
    }

    /// Build launch arguments from profile
    fn build_launch_args(&self, debug_port: u16) -> Vec<String> {
        let mut args = Vec::new();

        // User data directory (required for CDP)
        if let Some(ref user_data_dir) = self.profile.user_data_dir {
            args.push(format!("--user-data-dir={}", user_data_dir.display()));
        } else {
            // Use unique temp directory for each launch to avoid Chrome's single-instance behavior
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let temp_dir = std::env::temp_dir().join(format!("browser-use-{}", timestamp));
            args.push(format!("--user-data-dir={}", temp_dir.display()));
        }

        // Headless mode
        if self.profile.headless.unwrap_or(false) {
            args.push("--headless".to_string());
            args.push("--disable-gpu".to_string());
        }

        // Remote debugging port
        args.push(format!("--remote-debugging-port={debug_port}"));

        // Proxy configuration (if set)
        if let Some(ref proxy) = self.profile.proxy {
            args.push(format!("--proxy-server={}", proxy.server));
            if let Some(ref bypass) = proxy.bypass {
                args.push(format!("--proxy-bypass-list={}", bypass));
            }
        }

        // Additional common args for automation
        // Note: --no-sandbox is not supported on macOS and modern Chrome
        // Note: --disable-blink-features is also not supported on modern Chrome
        args.extend(vec![
            "--disable-dev-shm-usage".to_string(),
        ]);

        args
    }

    /// Launch browser and return CDP WebSocket URL
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
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        tracing::debug!("Launching browser: {:?} with args: {:?}", browser_path, args);

        let mut child = command
            .spawn()
            .map_err(|e| BrowsingError::Browser(format!("Failed to launch browser: {e}")))?;

        // Give the browser a moment to start
        sleep(Duration::from_millis(1000)).await;

        // Check if process is still alive
        if let Ok(Some(status)) = child.try_wait() {
            return Err(BrowsingError::Browser(format!(
                "Browser process exited immediately with status: {:?}",
                status
            )));
        }

        self.process = Some(child);

        // Wait for CDP to be ready
        let cdp_http_url = format!("http://127.0.0.1:{debug_port}");
        Self::wait_for_cdp_ready(&cdp_http_url).await?;

        // Get WebSocket debugger URL
        let ws_url = Self::get_websocket_debugger_url(&cdp_http_url).await?;

        Ok(ws_url)
    }

    /// Wait for CDP endpoint to be ready
    async fn wait_for_cdp_ready(cdp_url: &str) -> Result<()> {
        let max_attempts = 60;
        let delay = Duration::from_millis(500);

        for attempt in 0..max_attempts {
            // Try to connect to CDP endpoint
            match reqwest::get(format!("{cdp_url}/json/version")).await {
                Ok(response) => {
                    if response.status().is_success() {
                        tracing::debug!("CDP endpoint ready after {} attempts", attempt + 1);
                        return Ok(());
                    }
                }
                Err(e) => {
                    tracing::trace!("CDP connection attempt {}/{} failed: {}", attempt + 1, max_attempts, e);
                }
            }

            sleep(delay).await;
        }

        Err(BrowsingError::Browser(
            format!("CDP endpoint did not become ready in time after {} attempts. Make sure Chrome is installed and can be launched.", max_attempts)
        ))
    }

    /// Get WebSocket debugger URL from CDP HTTP endpoint
    async fn get_websocket_debugger_url(cdp_http_url: &str) -> Result<String> {
        let response = reqwest::get(format!("{cdp_http_url}/json"))
            .await
            .map_err(|e| BrowsingError::Browser(format!("Failed to fetch CDP targets: {e}")))?;

        let targets: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| BrowsingError::Browser(format!("Failed to parse CDP targets: {e}")))?;

        // Find the first page target
        for target in targets {
            if target["type"].as_str() == Some("page") {
                if let Some(ws_url) = target["webSocketDebuggerUrl"].as_str() {
                    return Ok(ws_url.to_string());
                }
            }
        }

        Err(BrowsingError::Browser(
            "No WebSocket debugger URL found in CDP targets".to_string(),
        ))
    }

    /// Stop the browser process
    ///
    /// # Safety Policy
    ///
    /// **IMPORTANT**: This method ONLY stops the browser process.
    /// It does NOT delete the user data directory.
    ///
    /// This is intentional for safety reasons:
    /// - Users may specify a custom `user_data_dir` pointing to their real browser profile
    /// - Accidentally deleting user data (bookmarks, history, passwords, etc.) would be catastrophic
    /// - Temporary directories are left in place for debugging and inspection
    ///
    /// Users are responsible for managing their own user data directories.
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
        // NOTE: We do NOT clean up user_data_dir - see stop() method for safety policy
        if let Some(ref mut process) = self.process {
            let _ = process.kill();
        }
    }
}
