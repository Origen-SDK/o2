// This file defines the public API for consuming and generating STIL

pub mod nodes;
mod parser;
pub mod processors;
use crate::ast::Node;
use crate::Result as OrigenResult;
pub use nodes::STIL;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ParseOptions {
    pub(crate) allow_missing_final_waveform_event_semicolon: bool,
}

/// A configurable STIL parser.
///
/// The default parser accepts only standard STIL syntax. Compatibility extensions must be
/// explicitly enabled before parsing.
#[derive(Clone, Debug, Default)]
pub struct Parser {
    options: ParseOptions,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow the final event in a waveform definition to omit its semicolon.
    pub fn allow_missing_final_waveform_event_semicolon(mut self) -> Self {
        self.options.allow_missing_final_waveform_event_semicolon = true;
        self
    }

    pub fn from_file(&self, path: &Path) -> OrigenResult<Node<STIL>> {
        let ast = parser::parse_file_with_options(path, self.options)?;
        let load_path = vec![Path::new(path).parent().unwrap().to_path_buf()];

        processors::includer::Includer::run_with_options(
            &ast,
            load_path,
            HashMap::new(),
            self.options,
        )
    }

    pub fn from_file_ignore_includes(&self, path: &Path) -> OrigenResult<Node<STIL>> {
        parser::parse_file_with_options(path, self.options)
    }

    /// Parse the given STIL file, using the given load path to resolve include statements.
    pub fn from_file_with_options(
        &self,
        path: &Path,
        load_path: &Vec<PathBuf>,
        rename: Option<&HashMap<&str, &str>>,
    ) -> OrigenResult<Node<STIL>> {
        let ast = parser::parse_file_with_options(path, self.options)?;
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
        processors::includer::Includer::run_with_options(
            &ast,
            load_path_with_current,
            rename,
            self.options,
        )
    }

    pub fn from_str(&self, stil: &str, root_dir: Option<&str>) -> OrigenResult<Node<STIL>> {
        let ast = parser::parse_str_with_options(stil, None, self.options)?;
        let load_path = {
            if let Some(p) = root_dir {
                vec![Path::new(p).to_path_buf()]
            } else {
                vec![]
            }
        };
        processors::includer::Includer::run_with_options(
            &ast,
            load_path,
            HashMap::new(),
            self.options,
        )
    }
}

pub fn from_file(path: &Path) -> OrigenResult<Node<STIL>> {
    Parser::new().from_file(path)
}

pub fn from_file_ignore_includes(path: &Path) -> OrigenResult<Node<STIL>> {
    Parser::new().from_file_ignore_includes(path)
}

/// Parse the given STIL file, using the given load path to resolve any include statements
/// that are encountered.
/// Include files can optionally be renamed using the `rename` argument, which
/// is a map of the original include file name to the new name.
pub fn from_file_with_options(
    path: &Path,
    load_path: &Vec<PathBuf>,
    rename: Option<&HashMap<&str, &str>>,
) -> OrigenResult<Node<STIL>> {
    Parser::new().from_file_with_options(path, load_path, rename)
}

pub fn from_str(stil: &str, root_dir: Option<&str>) -> OrigenResult<Node<STIL>> {
    Parser::new().from_str(stil, root_dir)
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
