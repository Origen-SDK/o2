use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::{Model, ParamValue};
use std::collections::HashMap;

/// Makes test suite names unique, no AST changes are made or returned
pub struct CleanNames {
    model: Model,
    test_suite_names: HashMap<String, usize>,
}

pub fn run(node: &Node, model: Model) -> Result<Model> {
    let mut p = CleanNames {
        model: model,
        test_suite_names: HashMap::new(),
    };
    let _ = node.process(&mut p)?;
    Ok(p.model)
}

impl Processor for CleanNames {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMTest(id, _) => {
                let t = self.model.test_invocations.get_mut(id).unwrap();
                let name = t.get("name")?.unwrap().to_string();
                if self.test_suite_names.contains_key(&name) {
                    let count = self.test_suite_names[&name];
                    self.test_suite_names.insert(name.clone(), count + 1);
                    t.set(
                        "name",
                        ParamValue::String(format!("{}_{}", name, count + 1)),
                        false,
                    )?;
                } else {
                    self.test_suite_names.insert(name.clone(), 0);
                }
                Return::ProcessChildren
            }
            _ => Return::ProcessChildren,
        })
    }
}
