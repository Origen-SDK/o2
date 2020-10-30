use crate::generator::ast::*;
use crate::generator::processor::*;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

/// Does the final writing of the flow AST to a SMT7 flow file
pub struct WriteToFile {
    output_dir: PathBuf,
    file_path: Option<PathBuf>,
}

pub fn run(ast: &Node, output_dir: &Path) -> Result<PathBuf> {
    let mut p = WriteToFile {
        output_dir: output_dir.to_owned(),
        file_path: None,
    };
    ast.process(&mut p)?;
    Ok(p.file_path.unwrap())
}

impl Processor for WriteToFile {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        let result = match &node.attrs {
            Attrs::PGMFlow(name) => {
                {
                    self.file_path = Some(self.output_dir.join(&format!("{}.tf", name)));
                    let _ = node.process_children(self);

                    let mut tera = Tera::default();
                    let context = Context::new();
                    //context.insert("packages", &packages);
                    let contents = tera
                        .render_str(include_str!("../templates/flow.tf.tera"), &context)
                        .unwrap();
                    std::fs::write(&self.file_path.as_ref().unwrap(), &contents)?;
                }
                Return::None
            }
            _ => Return::ProcessChildren,
        };
        Ok(result)
    }
}
