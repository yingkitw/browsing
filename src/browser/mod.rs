//! Browser session management

pub mod session;
pub mod profile;
pub mod views;
pub mod launcher;
pub mod cdp;

pub use session::Browser;
pub use profile::BrowserProfile;
pub use views::*;

