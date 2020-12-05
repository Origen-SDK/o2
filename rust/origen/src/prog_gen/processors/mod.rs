//! Contains program generator processors that are (or are likely to be) applicable
//! to multiple tester targets

pub mod adjacent_if_combiner;
pub mod condition;
pub mod extract_to_model;
pub mod flag_optimizer;
pub mod nest_on_result_nodes;
pub mod relationship;
