//! Resolves all include statements in the given AST

use super::super::nodes::STIL;
use super::super::parser;
use crate::ast::Node;
use crate::ast::{Processor, Return};
use crate::Result;
use shellexpand;
use std::env;
use std::path::{Path, PathBuf};

pub struct Includer {
    dir: PathBuf,
}

impl Includer {
    #[allow(dead_code)]
    pub fn run(node: &Node<STIL>, dir: Option<&Path>) -> Result<Node<STIL>> {
        let mut p = Includer {
            dir: match dir {
                Some(p) => p.to_path_buf(),
                None => env::current_dir()?,
            },
        };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor<STIL> for Includer {
    fn on_node(&mut self, node: &Node<STIL>) -> Result<Return<STIL>> {
        let result = match &node.attrs {
            STIL::Include(file, _) => {
                let expanded = format!("{}", shellexpand::full(file)?);
                let mut path = self.dir.clone();
                // Note that if expanded is absolute then the push method will replace the current
                // path with the given one
                path.push(expanded);
                let ast = parser::parse_file(&path)?;
                Return::Replace(Includer::run(&ast, path.parent())?)
            }
            STIL::Root => Return::ProcessChildren,
            // No need to recurse into other nodes, all includes should be at the top-level
            _ => Return::Unmodified,
        };
        Ok(result)
    }
}
