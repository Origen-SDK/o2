use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::{FlowCondition, Model, ParamValue};
use regex::Regex;
use std::collections::HashMap;

pub struct AddSig {
    sig: Option<String>,
    model: Model,
    test_suite_names: HashMap<String, usize>,
}

pub fn run(node: &Node, model: Model, sig: Option<String>) -> Result<(Node, Model)> {
    let mut p = AddSig {
        sig: sig,
        model: model,
        test_suite_names: HashMap::new(),
    };
    let ast = node.process(&mut p)?.unwrap();

    Ok((ast, p.model))
}

impl Processor for AddSig {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMFlow(name) => {
                let n = node.process_and_update_children(self)?;
                if let Some(sig) = &self.sig {
                    for id in &self.model.flows[name].test_invocations {
                        let t = self.model.test_invocations.get_mut(id).unwrap();
                        let new_name = format!("{}_{}", t.get("name")?.unwrap(), sig);
                        t.set("name", Some(ParamValue::String(new_name)), false)?;
                    }
                }
                Return::Replace(n)
            }
            Attrs::PGMTest(id, _) => {
                let t = self.model.test_invocations.get_mut(id).unwrap();
                let name = t.get("name")?.unwrap().to_string();
                if self.test_suite_names.contains_key(&name) {
                    let count = self.test_suite_names[&name];
                    self.test_suite_names.insert(name.clone(), count + 1);
                    t.set(
                        "name",
                        Some(ParamValue::String(format!("{}_{}", name, count + 1))),
                        false,
                    )?;
                } else {
                    self.test_suite_names.insert(name.clone(), 0);
                }
                Return::ProcessChildren
            }
            Attrs::PGMCondition(cond) => match cond {
                FlowCondition::IfFlag(flags) | FlowCondition::UnlessFlag(flags) => {
                    let flags: Vec<String> = flags
                        .iter()
                        .map(|f| {
                            let f = clean_flag(f);
                            if self.sig.is_some() {
                                self.add_sig_to_flag(&f)
                            } else {
                                f
                            }
                        })
                        .collect();
                    let children = node.process_and_box_children(self)?;
                    let attrs = match cond {
                        FlowCondition::IfFlag(_) => {
                            Attrs::PGMCondition(FlowCondition::IfFlag(flags))
                        }
                        FlowCondition::UnlessFlag(_) => {
                            Attrs::PGMCondition(FlowCondition::UnlessFlag(flags))
                        }
                        _ => unreachable!(),
                    };
                    Return::Replace(node.updated(Some(attrs), Some(children), None))
                }
                FlowCondition::IfEnable(flags) | FlowCondition::UnlessEnable(flags) => {
                    let flags: Vec<String> = flags.iter().map(|f| clean_flag(f)).collect();
                    let children = node.process_and_box_children(self)?;
                    let attrs = match cond {
                        FlowCondition::IfEnable(_) => {
                            Attrs::PGMCondition(FlowCondition::IfEnable(flags))
                        }
                        FlowCondition::UnlessEnable(_) => {
                            Attrs::PGMCondition(FlowCondition::UnlessEnable(flags))
                        }
                        _ => unreachable!(),
                    };
                    Return::Replace(node.updated(Some(attrs), Some(children), None))
                }
                _ => Return::ProcessChildren,
            },
            Attrs::PGMSetFlag(flag, state, is_auto_generated) => {
                let flag = {
                    let f = clean_flag(flag);
                    if *is_auto_generated {
                        if self.sig.is_some() {
                            self.add_sig_to_flag(&f)
                        } else {
                            f
                        }
                    } else {
                        f
                    }
                };
                let children = node.process_and_box_children(self)?;
                Return::Replace(node.updated(
                    Some(Attrs::PGMSetFlag(flag, *state, *is_auto_generated)),
                    Some(children),
                    None,
                ))
            }
            _ => Return::ProcessChildren,
        })
    }
}

impl AddSig {
    fn add_sig_to_flag(&self, flag: &str) -> String {
        let re = Regex::new(r"_(?P<flag>PASSED|FAILED|RAN)$").unwrap();
        let replacement = format!("_{}_$flag", self.sig.as_ref().unwrap());
        let r = re.replace(flag, &*replacement);
        r.to_string()
    }
}

fn clean_flag(flag: &str) -> String {
    if flag.starts_with("$") {
        flag.replacen("$", "", 1)
    } else {
        flag.to_uppercase()
    }
}
