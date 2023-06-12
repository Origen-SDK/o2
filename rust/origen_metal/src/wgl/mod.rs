// This file defines the public API for consuming and generating WGL

mod nodes;
mod parser;
mod processors;
use crate::ast::Node;
use crate::Result as OrigenResult;
use nodes::WGL;
use std::path::Path;

pub fn from_file(path: &Path) -> OrigenResult<Node<WGL>> {
    println!("{}", path.display());
    let ast = parser::parse_file(path)?;
    let ast = processors::includer::Includer::run(&ast, Path::new(path).parent())?;
    //println!("{}", ast.to_string());
    Ok(ast)
}

pub fn from_str(wgl: &str, root_dir: Option<&str>) -> OrigenResult<Node<WGL>> {
    let ast = parser::parse_str(wgl)?;
    let ast = match root_dir {
        Some(p) => processors::includer::Includer::run(&ast, Some(Path::new(p)))?,
        None => processors::includer::Includer::run(&ast, None)?,
    };
    Ok(ast)
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum Dir {
    input,
    output,
    bidir,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum DirType {
    reference,
    timing,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
pub enum StrobeDir {
    In,
    Out,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum Radix {
    binary,
    octal,
    decimal,
    hexadecimal,
    hex,
    symbolic,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum TimeUnit {
    ps,
    ns,
    us,
    ms,
    sec,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum ScanDir {
    input,
    output,
    feedback,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum Scale {
    p,
    n,
    u,
    m,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum EqUnit {
    A,
    V,
    S,
    H,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum TimeGenType {
    force,
    compare,
    direction,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
#[allow(non_camel_case_types)]
pub enum PmodeOption {
    dont_care,
    last_force,
    last_drive,
    force_or_z,
    advantest,
    ims,
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
pub enum BuiltInFunc {
    ACOS,
    ASIN,
    ATAN,
    CEIL,
    COS,
    COSH,
    EXP,
    FABS,
    FLOOR,
    LOG,
    LOG10,
    SIN,
    SINH,
    SQRT,
    TAN,
    TANH,
    ATAN2,
    POW
}

#[derive(Clone, Debug, PartialEq, Serialize, enum_utils::FromStr)]
pub enum BuiltInVar {
    PI,
    E,
    DEG,
}