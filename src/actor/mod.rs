//! Actor module for low-level browser interactions

pub mod element;
pub mod keyboard;
pub mod mouse;
pub mod page;

pub use element::Element;
pub use keyboard::get_key_info;
pub use mouse::Mouse;
pub use page::Page;
