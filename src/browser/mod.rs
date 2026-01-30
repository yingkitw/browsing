//! Browser session management

mod navigation;
mod screenshot;
mod session_guard;
mod tab_manager;

pub mod cdp;
pub mod launcher;
pub mod profile;
pub mod session;
pub mod views;

pub use navigation::NavigationManager;
pub use screenshot::ScreenshotManager;
pub use tab_manager::TabManager;

pub use profile::BrowserProfile;
pub use session::Browser;
pub use views::*;
