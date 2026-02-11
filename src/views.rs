//! Re-exports of view types from module-specific views
//!
//! Single source of truth: each module owns its view types in its own `views` submodule.
//! This module provides convenient re-exports for a unified public API.

pub use crate::browser::views::BrowserStateSummary;
