//! Browser session management

pub mod cdp;
pub mod launcher;
pub mod profile;
pub mod session;
pub mod views;

pub use profile::BrowserProfile;
pub use session::Browser;
pub use views::*;
