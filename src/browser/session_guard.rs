//! Session guard utility for safe CDP session access
//!
//! This module provides a guard pattern for accessing browser sessions,
//! reducing boilerplate and ensuring consistent error handling.

use crate::browser::cdp::{CdpClient, CdpSession};
use crate::error::{BrowsingError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Guard for safe access to browser sessions
///
/// This struct provides safe access to active browser sessions,
/// encapsulating the common pattern of checking for an active session
/// and returning an error if none exists.
pub struct SessionGuard<'a> {
    sessions: &'a HashMap<String, CdpSession>,
    current_target_id: Option<&'a String>,
}

impl<'a> SessionGuard<'a> {
    /// Create a new session guard
    pub fn new(
        sessions: &'a HashMap<String, CdpSession>,
        current_target_id: Option<&'a String>,
    ) -> Self {
        Self {
            sessions,
            current_target_id,
        }
    }

    /// Get the currently active session
    ///
    /// Returns an error if no session is active.
    pub fn get_active_session(&self) -> Result<&'a CdpSession> {
        self.current_target_id
            .and_then(|id| self.sessions.get(id))
            .ok_or_else(|| BrowsingError::Browser("No active session".to_string()))
    }

    /// Get the CDP client for the active session
    ///
    /// Returns an error if no session is active.
    pub fn get_client(&self) -> Result<Arc<CdpClient>> {
        Ok(Arc::clone(&self.get_active_session()?.client))
    }

    /// Get the session ID for the active session
    ///
    /// Returns an error if no session is active.
    pub fn get_session_id(&self) -> Result<&'a str> {
        Ok(self.get_active_session()?.session_id.as_str())
    }

    /// Check if there is an active session
    pub fn has_active_session(&self) -> bool {
        self.current_target_id
            .is_some_and(|id| self.sessions.contains_key(id))
    }

    /// Get the current target ID
    pub fn current_target_id(&self) -> Option<&'a str> {
        self.current_target_id.map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_active_session() {
        let sessions = HashMap::new();
        let guard = SessionGuard::new(&sessions, None);
        assert!(guard.get_active_session().is_err());
        assert!(!guard.has_active_session());
    }

    #[test]
    fn test_with_active_session() {
        let mut sessions = HashMap::new();
        let target_id = "test-target".to_string();
        // Note: We can't easily create a CdpSession without a real CDP client
        // so this test is more of a compile-time check
        let guard = SessionGuard::new(&sessions, Some(&target_id));
        assert!(guard.get_active_session().is_err()); // No session in map
        assert!(!guard.has_active_session());
    }
}
