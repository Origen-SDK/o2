//! Contains program generator processors that are (or are likely to be) applicable
//! to multiple tester targets

pub mod adjacent_if_combiner;
pub mod clean_resources;
pub mod condition;
pub mod continue_implementer;
pub mod final_model_extract;
pub mod flag_optimizer;
pub mod initial_model_extract;
pub mod nest_on_result_nodes;
pub mod relationship;
pub mod target_tester;

use crate::ast::Node;
use crate::prog_gen::PGM;

/// Returns the names of the variables declared as outputs (`PGM::OutputVariable`) anywhere in
/// the given AST. A test method may change such a variable natively, so the condition
/// optimizers treat them like volatile flags.
pub fn declared_output_variables(node: &Node<PGM>) -> Vec<String> {
    let mut names: Vec<String> = vec![];
    let mut pending: Vec<&Node<PGM>> = vec![node];
    while let Some(n) = pending.pop() {
        if let PGM::OutputVariable(name) = &n.attrs {
            if !names.contains(name) {
                names.push(name.to_string());
            }
        }
        pending.extend(n.children.iter().map(|c| c.as_ref()));
    }
    names
}
