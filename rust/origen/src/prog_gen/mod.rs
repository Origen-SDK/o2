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
pub use model::PatternGroupType;
pub use model::Test;

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
}
