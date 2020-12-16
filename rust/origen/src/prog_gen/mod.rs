pub mod advantest;
pub mod flow_api;
mod flow_manager;
mod model;
mod processors;
pub mod teradyne;
mod validators;

pub use flow_manager::FlowManager;
pub use model::Bin;
pub use model::BinType;
pub use model::FlowCondition;
pub use model::FlowID;
pub use model::GroupType;
pub use model::Limit;
pub use model::LimitSelector;
pub use model::LimitType;
use model::Model;
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
