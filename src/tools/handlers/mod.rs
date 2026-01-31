//! Action handlers for browser automation
//!
//! This module contains individual action handlers organized by functionality.

mod advanced;
mod content;
mod interaction;
mod navigation;
mod tabs;

pub use advanced::AdvancedHandler;
pub use content::ContentHandler;
pub use interaction::InteractionHandler;
pub use navigation::NavigationHandler;
pub use tabs::TabsHandler;

use crate::agent::views::ActionResult;
use crate::error::Result;
use crate::tools::views::{ActionContext, ActionParams};
use async_trait::async_trait;

/// Base trait for action handlers
#[async_trait]
pub trait Handler: Send + Sync {
    /// Handle an action with the given parameters and context
    async fn handle(
        &self,
        params: &ActionParams<'_>,
        context: &mut ActionContext<'_>,
    ) -> Result<ActionResult>;
}
