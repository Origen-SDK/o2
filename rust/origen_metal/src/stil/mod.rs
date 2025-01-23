// This file defines the public API for consuming and generating STIL

pub mod nodes;
mod parser;
pub mod processors;
use crate::ast::Node;
use crate::Result as OrigenResult;
pub use nodes::STIL;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn from_file(path: &Path) -> OrigenResult<Node<STIL>> {
    let ast = parser::parse_file(path, true)?;
    let load_path = vec![Path::new(path).parent().unwrap().to_path_buf()];

    let ast = processors::includer::Includer::run(&ast, load_path, HashMap::new(), true)?;
    Ok(ast)
}

pub fn from_file_ignore_includes(path: &Path) -> OrigenResult<Node<STIL>> {
    let ast = parser::parse_file(path, true)?;
    Ok(ast)
}

/// Parse the given STIL file, using the given load path to resolve any include statements
/// that are encountered.
/// Include files can optionally be renamed using the `rename` argument, which
/// is a map of the original include file name to the new name.
pub fn from_file_with_options(
    path: &Path,
    load_path: &Vec<PathBuf>,
    rename: Option<&HashMap<&str, &str>>,
    strict: bool,
) -> OrigenResult<Node<STIL>> {
    let ast = parser::parse_file(path, strict)?;
    let mut load_path_with_current = vec![Path::new(path).parent().unwrap().to_path_buf()];
    for p in load_path {
        load_path_with_current.push(p.clone());
    }
    let rename = match rename {
        Some(r) => {
            let mut rename = HashMap::new();
            for (orig, new) in r {
                rename.insert(orig.to_string(), new.to_string());
            }
            rename
        }
        None => HashMap::new(),
    };
    let ast = processors::includer::Includer::run(&ast, load_path_with_current, rename, strict)?;
    Ok(ast)
}

pub fn from_str(stil: &str, root_dir: Option<&str>) -> OrigenResult<Node<STIL>> {
    let ast = parser::parse_str(stil, None, true)?;
    let load_path = {
        if let Some(p) = root_dir {
            vec![Path::new(p).to_path_buf()]
        } else {
            vec![]
        }
    };
    let ast = processors::includer::Includer::run(&ast, load_path, HashMap::new(), true)?;
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
