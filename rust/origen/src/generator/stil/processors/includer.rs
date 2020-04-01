//! Resolves all include statements in the given AST

use super::super::parser;
use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::Result;
use shellexpand;
use std::env;
use std::path::{Path, PathBuf};

pub struct Includer {
    dir: PathBuf,
}

impl Includer {
    #[allow(dead_code)]
    pub fn run(node: &Node, dir: Option<&Path>) -> Result<Node> {
        let mut p = Includer {
            dir: match dir {
                Some(p) => p.to_path_buf(),
                None => env::current_dir()?,
            },
        };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor for Includer {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        let result = match &node.attrs {
            Attrs::STILInclude(file, _) => {
                let expanded = format!("{}", shellexpand::full(file)?);
                let mut path = self.dir.clone();
                // Note that if expanded is absolute then the push method will replace the current
                // path with the given one
                path.push(expanded);
                let ast = parser::parse_file(&path)?;
                Return::Replace(Includer::run(&ast, path.parent())?)
            }
            Attrs::STIL => Return::ProcessChildren,
            // No need to recurse into other nodes, all includes should be at the top-level
            _ => Return::Unmodified,
        };
        Ok(result)
    }
}
