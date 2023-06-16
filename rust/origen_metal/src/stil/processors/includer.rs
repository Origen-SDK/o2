//! Resolves all include statements in the given AST

use super::super::nodes::STIL;
use super::super::parser;
use crate::ast::Node;
use crate::ast::{Processor, Return};
use crate::Result;
use shellexpand;
use std::env;
use std::path::PathBuf;

pub struct Includer {
    load_path: Vec<PathBuf>,
}

impl Includer {
    #[allow(dead_code)]
    pub fn run(node: &Node<STIL>, load_path: Vec<PathBuf>) -> Result<Node<STIL>> {
        let mut full_load_path = vec![env::current_dir()?];
        for p in load_path {
            full_load_path.push(p.to_path_buf());
        }
        let mut p = Includer {
            load_path: full_load_path,
        };
        Ok(node.process(&mut p)?.unwrap())
    }
}

impl Processor<STIL> for Includer {
    fn on_node(&mut self, node: &Node<STIL>) -> Result<Return<STIL>> {
        match &node.attrs {
            STIL::Include(file, _) => {
                let expanded = format!("{}", shellexpand::full(&file)?);
                for p in &self.load_path {
                    let mut path = p.clone();
                    // Note that if expanded is absolute then the push method will replace the current
                    // path with the given one
                    path.push(&expanded);
                    if path.exists() {
                        let ast = parser::parse_file(&path)?;
                        return Ok(Return::Replace(Includer::run(
                            &ast,
                            self.load_path.clone(),
                        )?));
                    }
                }
                bail!("Unable to find include file: {}", expanded)
            }
            STIL::Root => Ok(Return::ProcessChildren),
            // No need to recurse into other nodes, all includes should be at the top-level
            _ => Ok(Return::Unmodified),
        }
    }
}
