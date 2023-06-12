// This file defines the public API for consuming VCD

mod nodes;
mod parser;
mod processors;
use crate::ast::Node;
use crate::Result as OrigenResult;
use nodes::VCD;
use std::path::Path;

pub fn from_file(path: &Path) -> OrigenResult<Node<VCD>> {
    println!("{}", path.display());
    let ast = parser::parse_file(path)?;
    let ast = processors::sectioner::Sectioner::run(&ast)?;
    let ast = processors::scoper::Scoper::run(&ast)?;
    //println!("{}", ast.to_string());
    Ok(ast)
}

pub fn from_str(vcd: &str) -> OrigenResult<Node<VCD>> {
    let ast = parser::parse_str(vcd)?;
    let ast = processors::sectioner::Sectioner::run(&ast)?;
    let ast = processors::scoper::Scoper::run(&ast)?;
    Ok(ast)
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum ScopeType {
    begin,
    fork,
    function,
    module,
    task
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum VarType {
    event,
    integer,
    parameter,
    real,
    reg,
    supply0,
    supply1,
    time,
    triand,
    trior,
    trireg,
    tri0,
    tri1,
    tri,
    wand,
    wire,
    wor
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum TimeUnit {
    fs,
    ps,
    ns,
    us,
    ms,
    s
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
pub enum ValueChangeType {
    Scalar,
    Vector
}