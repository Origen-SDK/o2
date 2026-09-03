//! Parser and AST definitions for IEEE 1687-2014 Instrument Connectivity Language (ICL).

pub mod model;
mod nodes;
mod parser;

use crate::ast::Node;
use crate::Result;
use std::path::Path;

pub use nodes::{AccessLinkStandard, MuxType, PortType, SignalType, ICL};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ParseOptions {
    pub(crate) preserve_comments: bool,
}

/// A configurable IEEE 1687-2014 ICL parser.
///
/// Comments are discarded by default. Call [`Parser::preserve_comments`] when an AST retaining
/// comments is required.
#[derive(Clone, Debug, Default)]
pub struct Parser {
    options: ParseOptions,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Preserve line and block comments as [`ICL::Comment`] nodes.
    pub fn preserve_comments(mut self) -> Self {
        self.options.preserve_comments = true;
        self
    }

    pub fn from_file(&self, path: &Path) -> Result<Node<ICL>> {
        parser::parse_file_with_options(path, self.options)
    }

    pub fn from_str(&self, icl: &str) -> Result<Node<ICL>> {
        parser::parse_str_with_options(icl, None, self.options)
    }
}

pub fn from_file(path: &Path) -> Result<Node<ICL>> {
    Parser::new().from_file(path)
}

pub fn from_str(icl: &str) -> Result<Node<ICL>> {
    Parser::new().from_str(icl)
}
