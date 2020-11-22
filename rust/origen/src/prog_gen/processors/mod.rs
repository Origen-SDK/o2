//! Contains program generator processors that are (or are likely to be) applicable
//! to multiple tester targets

mod extract_to_model;
mod nest_on_result_nodes;

pub use extract_to_model::ExtractToModel;
pub use nest_on_result_nodes::NestOnResultNodes;
