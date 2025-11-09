//! Signal handling for graceful shutdown

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};

/// Global flag to track if shutdown was requested
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Check if shutdown was requested
pub fn is_shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::Relaxed)
}

/// Set shutdown flag
pub fn set_shutdown_requested() {
    SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
}

/// Signal handler for graceful shutdown
pub struct SignalHandler {
    shutdown_flag: Arc<AtomicBool>,
}

impl SignalHandler {
    /// Create a new signal handler
    pub fn new() -> Self {
        Self {
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if shutdown was requested
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_flag.load(Ordering::Relaxed)
    }

    /// Set shutdown flag
    pub fn set_shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::Relaxed);
        set_shutdown_requested();
    }

    /// Wait for shutdown signal (SIGINT or SIGTERM)
    /// Returns when a signal is received
    pub async fn wait_for_shutdown(&self) {
        #[cfg(unix)]
        {
            let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                .expect("Failed to register SIGINT handler");
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to register SIGTERM handler");

            tokio::select! {
                _ = sigint.recv() => {
                    info!("🛑 Received SIGINT (Ctrl+C), initiating graceful shutdown...");
                    self.set_shutdown();
                }
                _ = sigterm.recv() => {
                    info!("🛑 Received SIGTERM, initiating graceful shutdown...");
                    self.set_shutdown();
                }
            }
        }

        #[cfg(not(unix))]
        {
            // On Windows, use Ctrl+C handler
            tokio::select! {
                _ = signal::ctrl_c() => {
                    info!("🛑 Received Ctrl+C, initiating graceful shutdown...");
                    self.set_shutdown();
                }
            }
        }
    }

    /// Spawn a background task to listen for shutdown signals
    pub fn spawn_shutdown_listener(&self) -> tokio::task::JoinHandle<()> {
        let flag = Arc::clone(&self.shutdown_flag);
        tokio::spawn(async move {
            #[cfg(unix)]
            {
                let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                    .expect("Failed to register SIGINT handler");
                let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("Failed to register SIGTERM handler");

                tokio::select! {
                    _ = sigint.recv() => {
                        info!("🛑 Received SIGINT (Ctrl+C), initiating graceful shutdown...");
                        flag.store(true, Ordering::Relaxed);
                        set_shutdown_requested();
                    }
                    _ = sigterm.recv() => {
                        info!("🛑 Received SIGTERM, initiating graceful shutdown...");
                        flag.store(true, Ordering::Relaxed);
                        set_shutdown_requested();
                    }
                }
            }

            #[cfg(not(unix))]
            {
                if let Err(e) = signal::ctrl_c().await {
                    warn!("Failed to register Ctrl+C handler: {}", e);
                } else {
                    info!("🛑 Received Ctrl+C, initiating graceful shutdown...");
                    flag.store(true, Ordering::Relaxed);
                    set_shutdown_requested();
                }
            }
        })
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_flag() {
        let handler = SignalHandler::new();
        assert!(!handler.is_shutdown_requested());
        
        handler.set_shutdown();
        assert!(handler.is_shutdown_requested());
        assert!(is_shutdown_requested());
    }

    #[test]
    fn test_global_shutdown_flag() {
        assert!(!is_shutdown_requested());
        set_shutdown_requested();
        assert!(is_shutdown_requested());
    }
}

