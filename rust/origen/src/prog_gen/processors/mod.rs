//! Contains program generator processors that are (or are likely to be) applicable
//! to multiple tester targets

pub mod adjacent_if_combiner;
pub mod condition;
pub mod continue_implementer;
pub mod final_model_extract;
pub mod flag_optimizer;
pub mod initial_model_extract;
pub mod nest_on_result_nodes;
pub mod relationship;
