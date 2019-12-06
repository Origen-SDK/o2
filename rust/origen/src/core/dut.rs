use std::collections::HashMap;
use crate::core::model::Model;

#[derive(Debug)]
pub struct DUT {
    pub id: String,
    /// Model representing the top-level of the DUT
    pub top_level: Model,
    pub sub_blocks: HashMap<String, Model>,
}
