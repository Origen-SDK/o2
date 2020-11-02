use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::model::{Bin, Test};
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

/// Does the final writing of the flow AST to a SMT7 flow file
pub struct WriteToFile {
    output_dir: PathBuf,
    file_path: Option<PathBuf>,
    flow_header: Vec<String>,
    flow_body: Vec<String>,
    flow_footer: Vec<String>,
    indent: usize,
}

pub fn run(ast: &Node, output_dir: &Path) -> Result<PathBuf> {
    let mut p = WriteToFile {
        output_dir: output_dir.to_owned(),
        file_path: None,
        flow_header: vec![],
        flow_body: vec![],
        flow_footer: vec![],
        indent: 0,
    };
    ast.process(&mut p)?;
    Ok(p.file_path.unwrap())
}

impl WriteToFile {
    fn push_body(&mut self, line: &str) {
        self.flow_body
            .push(format!("{:indent$}{}", "", line, indent = self.indent * 2));
    }
}

impl Processor for WriteToFile {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        let result = match &node.attrs {
            Attrs::PGMFlow(name) => {
                {
                    self.file_path = Some(self.output_dir.join(&format!("{}.tf", name)));

                    self.indent += 1;
                    self.push_body("{");
                    self.indent += 1;
                    let _ = node.process_children(self);
                    self.indent -= 1;
                    self.push_body(&format!("}}, open,\"{}\",\"\"", &name.to_uppercase()));
                    self.indent -= 1;

                    let mut tera = Tera::default();
                    let mut context = Context::new();

                    let test_methods: Vec<&Test> = vec![];
                    let test_suites: Vec<&Test> = vec![];
                    let hard_bins: Vec<&Bin> = vec![];

                    context.insert("test_methods", &test_methods);
                    context.insert("test_suites", &test_suites);
                    context.insert("hard_bins", &hard_bins);
                    context.insert("flow_header", &self.flow_header);
                    context.insert("flow_body", &self.flow_body);
                    context.insert("flow_footer", &self.flow_footer);

                    let contents = tera
                        .render_str(include_str!("../templates/flow.tf.tera"), &context)
                        .unwrap();
                    std::fs::write(&self.file_path.as_ref().unwrap(), &contents)?;
                }
                Return::None
            }
            Attrs::PGMTest(name, tester, test_method_id, test_suite_id) => {
                self.push_body(&format!("run({});", name));
                Return::ProcessChildren
            }
            _ => Return::ProcessChildren,
        };
        Ok(result)
    }
}
