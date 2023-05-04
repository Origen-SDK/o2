// This file defines the public API for consuming and generating STIL

mod nodes;
mod parser;
mod processors;
use crate::ast::Node;
use crate::Result as OrigenResult;
pub use nodes::STIL;
use std::path::Path;

pub fn from_file(path: &Path) -> OrigenResult<Node<STIL>> {
    let ast = parser::parse_file(path)?;
    let ast = processors::includer::Includer::run(&ast, Path::new(path).parent())?;
    Ok(ast)
}

pub fn from_str(stil: &str, root_dir: Option<&str>) -> OrigenResult<Node<STIL>> {
    let ast = parser::parse_str(stil)?;
    let ast = match root_dir {
        Some(p) => processors::includer::Includer::run(&ast, Some(Path::new(p)))?,
        None => processors::includer::Includer::run(&ast, None)?,
    };
    Ok(ast)
}
#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[enumeration(case_insensitive)]
pub enum SignalType {
    InOut,
    Out,
    In,
    Supply,
    Pseudo,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
pub enum Termination {
    TerminateHigh,
    TerminateLow,
    TerminateOff,
    TerminateUnknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
pub enum State {
    U,
    D,
    Z,
    ForceUp,
    ForceDown,
    ForceOff,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
pub enum Base {
    Hex,
    Dec,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
pub enum Alignment {
    MSB,
    LSB,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
pub enum Selector {
    Min,
    Typ,
    Max,
    Meas,
}
