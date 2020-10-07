//! This module contains all of the data/model for a test program, including all flows,
//! test templates, test instances, etc.

mod test;
mod test_collection;
mod test_invocation;
mod test_program;

pub use test::Test;
pub use test_collection::TestCollection;
pub use test_invocation::TestInvocation;
pub use test_program::TestProgram;

#[derive(Debug)]
pub enum ParamValue {
    String(String),
    Int(i128),
    UInt(u128),
    Number(f64),
    Current(f64),
    Voltage(f64),
    Time(f64),
    Frequency(f64),
    Bool(bool),
}

impl ParamValue {
    pub fn is_type(&self, kind: &ParamType) -> bool {
        match self {
            ParamValue::String(_) => kind == &ParamType::String,
            ParamValue::Int(_) => kind == &ParamType::Int,
            ParamValue::UInt(_) => kind == &ParamType::UInt,
            ParamValue::Number(_) => kind == &ParamType::Number,
            ParamValue::Current(_) => kind == &ParamType::Current,
            ParamValue::Voltage(_) => kind == &ParamType::Voltage,
            ParamValue::Time(_) => kind == &ParamType::Time,
            ParamValue::Frequency(_) => kind == &ParamType::Frequency,
            ParamValue::Bool(_) => kind == &ParamType::Bool,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParamType {
    String,
    Int,
    UInt,
    Number,
    Current,
    Voltage,
    Time,
    Frequency,
    Bool,
}

#[derive(Debug)]
pub enum Constraint {
    In(Vec<ParamValue>),
    GT(ParamValue),
    GTE(ParamValue),
    LT(ParamValue),
    LTE(ParamValue),
}
