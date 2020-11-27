//! This module contains all of the data/model for a test program, including all flows,
//! test templates, test instances, etc.

mod bin;
mod flow_id;
mod limit;
mod model;
mod template_loader;
mod test;

use crate::Result as OrigenResult;
pub use bin::Bin;
pub use flow_id::FlowID;
pub use limit::Limit;
pub use model::Model;
use std::fmt;
use std::str::FromStr;
pub use test::Test;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PatternGroupType {
    Patset,
    Patgroup,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum GroupType {
    Flow,
    Test,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BinType {
    Good,
    Bad,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum FlowCondition {
    IfJob(Vec<String>),
    UnlessJob(Vec<String>),
    IfEnable(Vec<String>),
    UnlessEnable(Vec<String>),
    IfPassed(Vec<FlowID>),
    IfAnyPassed(Vec<FlowID>),
    IfAllPassed(Vec<FlowID>),
    IfAnySitesPassed(Vec<FlowID>),
    IfAllSitesPassed(Vec<FlowID>),
    IfFailed(Vec<FlowID>),
    IfAnyFailed(Vec<FlowID>),
    IfAllFailed(Vec<FlowID>),
    IfAnySitesFailed(Vec<FlowID>),
    IfAllSitesFailed(Vec<FlowID>),
    IfRan(Vec<FlowID>),
    UnlessRan(Vec<FlowID>),
    IfFlag(Vec<String>),
    UnlessFlag(Vec<String>),
    IfAnySitesFlag(Vec<String>),
    IfAllSitesFlag(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ParamValue {
    String(String),
    Int(i64),
    UInt(u64),
    Float(f64),
    Current(f64),
    Voltage(f64),
    Time(f64),
    Frequency(f64),
    Bool(bool),
    // Like a string, but any value assigned to such an attribute will be accepted and converted into a string representation
    Any(String),
}

impl ParamValue {
    pub fn is_type(&self, kind: &ParamType) -> bool {
        match self {
            ParamValue::String(_) => kind == &ParamType::String,
            ParamValue::Int(_) => kind == &ParamType::Int,
            ParamValue::UInt(_) => kind == &ParamType::UInt,
            ParamValue::Float(_) => kind == &ParamType::Float,
            ParamValue::Current(_) => kind == &ParamType::Current,
            ParamValue::Voltage(_) => kind == &ParamType::Voltage,
            ParamValue::Time(_) => kind == &ParamType::Time,
            ParamValue::Frequency(_) => kind == &ParamType::Frequency,
            ParamValue::Bool(_) => kind == &ParamType::Bool,
            ParamValue::Any(_) => true,
        }
    }

    pub fn to_bool(&self) -> OrigenResult<bool> {
        if let ParamValue::Bool(v) = self {
            Ok(*v)
        } else {
            error!("Not a boolean value")
        }
    }
}

impl fmt::Display for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // This can probably go, decided to handle the type specific formatting in the testers instead
            ParamValue::String(v) => write!(f, "{}", v),
            ParamValue::Int(v) => write!(f, "{}", v),
            ParamValue::UInt(v) => write!(f, "{}", v),
            ParamValue::Float(v) => write!(f, "{}", v),
            ParamValue::Current(v) => write!(f, "{}", v),
            ParamValue::Voltage(v) => write!(f, "{}", v),
            ParamValue::Time(v) => write!(f, "{}", v),
            ParamValue::Frequency(v) => write!(f, "{}", v),
            ParamValue::Bool(v) => write!(f, "{}", v),
            ParamValue::Any(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum ParamType {
    String,
    Int,
    UInt,
    Float,
    Current,
    Voltage,
    Time,
    Frequency,
    Bool,
    Any,
}

impl FromStr for ParamType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept any case and with or without underscores
        let str = s.to_lowercase();
        match str.trim() {
            "string" => Ok(ParamType::String),
            "int" | "integer" => Ok(ParamType::Int),
            "uint" | "uinteger" => Ok(ParamType::UInt),
            "float" | "number" | "num" => Ok(ParamType::Float),
            "current" | "curr" | "i" => Ok(ParamType::Current),
            "voltage" | "volt" | "v" => Ok(ParamType::Voltage),
            "time" | "t" | "s" => Ok(ParamType::Time),
            "frequency" | "freq" | "hz" => Ok(ParamType::Frequency),
            "boolean" | "bool" => Ok(ParamType::Bool),
            "any" => Ok(ParamType::Any),
            _ => Err(format!("'{}' is not a valid parameter type, the available types are: String, Int, UInt, Number, Current, Voltage, Time, Frequency, Bool, Any", str)),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Constraint {
    In(Vec<ParamValue>),
    GT(ParamValue),
    GTE(ParamValue),
    LT(ParamValue),
    LTE(ParamValue),
}

impl Constraint {
    pub fn is_satisfied(&self, value: &ParamValue) -> OrigenResult<()> {
        match self {
            Constraint::In(values) => {
                if values.iter().any(|v| v == value) {
                    Ok(())
                } else {
                    error!(
                        "'{}' is not one of the permitted values: {}",
                        value,
                        values
                            .iter()
                            .map(|v| format!("'{}'", v))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
            }
            // Unimplemented for now, but placeholders in case such contraints are supported in future
            Constraint::GT(_) => Ok(()),
            Constraint::GTE(_) => Ok(()),
            Constraint::LT(_) => Ok(()),
            Constraint::LTE(_) => Ok(()),
        }
    }
}
