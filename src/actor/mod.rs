//! Actor module for low-level browser interactions

pub mod page;
pub mod element;
pub mod mouse;
pub mod keyboard;

pub use page::Page;
pub use element::Element;
pub use mouse::Mouse;
pub use keyboard::get_key_info;

