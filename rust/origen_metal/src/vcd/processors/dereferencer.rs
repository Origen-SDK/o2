//! Builds a lookup map from identifier_code to signal reference and scope.
//!
//! This processor walks the header section to collect `$var` declarations and
//! builds a map from each variable's identifier_code to its (reference_name, scope)
//! pair. This allows consumers to resolve value changes back to named signals
//! without maintaining their own lookup table.

use super::super::nodes::VCD;
use crate::ast::Node;
use crate::ast::{Processor, Return};
use crate::Result;
use std::collections::HashMap;

/// Maps identifier_code → (reference_name, scope)
pub struct Dereferencer {
    vars: HashMap<String, VarInfo>,
}

/// Information about a variable declaration from the VCD header.
#[derive(Clone, Debug)]
pub struct VarInfo {
    /// The signal reference name (e.g., "clk", "data", "accumulator")
    pub reference: String,
    /// The hierarchical scope path (e.g., "top.m1"), if resolved by the Scoper
    pub scope: Option<String>,
    /// The variable type (e.g., wire, reg, integer)
    pub var_type: String,
    /// The bit width of the variable
    pub size: u32,
}

impl Dereferencer {
    /// Run the Dereferencer on an AST, returning the populated Dereferencer
    /// with a lookup map from identifier_code to variable info.
    pub fn run(node: &Node<VCD>) -> Result<Dereferencer> {
        let mut p = Dereferencer {
            vars: HashMap::new(),
        };
        // Process the tree to collect Var declarations
        node.process(&mut p)?;
        Ok(p)
    }

    /// Look up a variable by its identifier code.
    /// Returns `None` if the identifier code is not found.
    pub fn lookup(&self, identifier_code: &str) -> Option<&VarInfo> {
        self.vars.get(identifier_code)
    }

    /// Get the full lookup map.
    pub fn vars(&self) -> &HashMap<String, VarInfo> {
        &self.vars
    }
}

impl Processor<VCD> for Dereferencer {
    fn on_node(&mut self, node: &Node<VCD>) -> Result<Return<VCD>> {
        let result = match &node.attrs {
            VCD::Root => Return::ProcessChildren,
            VCD::HeaderSection => Return::ProcessChildren,
            VCD::Var(var_type, size, identifier_code, reference, scope) => {
                self.vars.insert(
                    identifier_code.clone(),
                    VarInfo {
                        reference: reference.clone(),
                        scope: scope.clone(),
                        var_type: format!("{:?}", var_type),
                        size: *size,
                    },
                );
                Return::Unmodified
            }
            // Don't recurse into the data section — we only need header declarations
            VCD::DataSection => Return::Unmodified,
            _ => Return::Unmodified,
        };
        Ok(result)
    }
}
