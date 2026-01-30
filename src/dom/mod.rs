//! DOM parsing and serialization

pub mod enhanced_snapshot;
pub mod serializer;
pub mod service;
pub mod views;

#[cfg(test)]
mod serializer_test;

pub use enhanced_snapshot::build_snapshot_lookup;
pub use serializer::DOMTreeSerializer;
pub use service::DomService;
pub use views::*;
