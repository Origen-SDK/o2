// This file defines the public API for consuming and generating STIL

pub mod nodes;
mod parser;
pub mod processors;
use crate::ast::Node;
use crate::Result as OrigenResult;
pub use nodes::STIL;
use std::path::{Path, PathBuf};

pub fn from_file(path: &Path) -> OrigenResult<Node<STIL>> {
    let ast = parser::parse_file(path)?;
    let load_path = vec![Path::new(path).parent().unwrap().to_path_buf()];

    let ast = processors::includer::Includer::run(&ast, load_path)?;
    Ok(ast)
}

pub fn from_file_with_load_path(path: &Path, load_path: &Vec<PathBuf>) -> OrigenResult<Node<STIL>> {
    let ast = parser::parse_file(path)?;
    let mut load_path_with_current = vec![Path::new(path).parent().unwrap().to_path_buf()];
    for p in load_path {
        load_path_with_current.push(p.clone());
    }
    let ast = processors::includer::Includer::run(&ast, load_path_with_current)?;
    Ok(ast)
}

pub fn from_str(stil: &str, root_dir: Option<&str>) -> OrigenResult<Node<STIL>> {
    let ast = parser::parse_str(stil)?;
    let load_path = {
        if let Some(p) = root_dir {
            vec![Path::new(p).to_path_buf()]
        } else {
            vec![]
        }
    };
    let ast = processors::includer::Includer::run(&ast, load_path)?;
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
