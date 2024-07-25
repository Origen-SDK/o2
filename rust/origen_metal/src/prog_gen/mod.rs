pub mod advantest;
pub mod flow_api;
mod flow_manager;
mod model;
mod nodes;
mod processors;
pub mod teradyne;
mod validators;
pub mod config;
mod supported_testers;

use std::path::Path;
use std::path::PathBuf;

pub use flow_manager::FlowManager;
pub use model::Bin;
pub use model::BinType;
pub use model::FlowCondition;
pub use model::FlowID;
pub use model::GroupType;
pub use model::Limit;
pub use model::LimitSelector;
pub use model::LimitType;
pub use model::Model;
pub use model::ParamType;
pub use model::ParamValue;
pub use model::Pattern;
pub use model::PatternGroupType;
pub use model::PatternReferenceType;
pub use model::PatternType;
pub use model::Test;
pub use model::Variable;
pub use model::VariableOperation;
pub use model::VariableType;
pub use nodes::PGM;
pub use supported_testers::SupportedTester;

use crate::ast::{Attrs, Node};

#[derive(Debug, PartialEq, EnumString, Clone, Serialize)]
pub enum ResourcesType {
    #[strum(serialize = "All", serialize = "all", serialize = "ALL")]
    All,
    #[strum(
        serialize = "Pattern",
        serialize = "pattern",
        serialize = "PATTERN",
        serialize = "Patterns",
        serialize = "patterns",
        serialize = "PATTERNS"
    )]
    Patterns,
    #[strum(
        serialize = "Variable",
        serialize = "variable",
        serialize = "VARIABLE",
        serialize = "Variables",
        serialize = "variables",
        serialize = "VARIABLES"
    )]
    Variables,
}

/// The type of unique signature to append to test names and similar
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum UniquenessOption {
    /// No unique identitier
    None,
    /// Add an automatically generated signature
    Signature,
    /// Add the flow name
    Flowname,
    /// Add the given string
    String(String),
}

pub trait ProgramGenerator {
    
}

pub fn trace_error<T: Attrs>(node: &Node<T>, error: crate::Error) -> crate::Result<()> {
    let help = {
        let s = node.meta_string();
        if s != "" {
            s
        } else {
            if crate::PROG_GEN_CONFIG.debug_enabled() {
                // Don't display children since it's potentially huge
                let n = node.replace_children(vec![]);
                format!("Sorry, no flow source information was found, here is the flow node that failed if it helps:\n{}", n)
            } else {
                "Run again with the --debug switch to try and trace this back to a flow source file location".to_string()
            }
        }
    };
    bail!("{}\n{}", error, &help)
}

pub fn render_program(tester: SupportedTester, output_dir: &Path) -> crate::Result<(Vec<PathBuf>, Model)> {
    match tester {
        SupportedTester::V93KSMT7 => advantest::smt7::render(output_dir),
        _ => unimplemented!("Tester {:?} is not yet supported for render_program", tester),
    }
}