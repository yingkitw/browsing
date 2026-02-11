//! DOM parsing and serialization

mod ax_node;
mod cdp_client;
mod html_converter;
mod processor;
mod tree_builder;

pub mod enhanced_snapshot;
pub mod serializer;
pub mod service;
pub mod views;

#[cfg(test)]
mod serializer_test;

pub use ax_node::build_enhanced_ax_node;
pub use enhanced_snapshot::build_snapshot_lookup;
pub use processor::DOMProcessorImpl;
pub use serializer::DOMTreeSerializer;
pub use service::DomService;
pub use views::*;
