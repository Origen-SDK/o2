use crate::prog_gen::{FlowCondition, Model, ParamValue, UniquenessOption, PGM};
use crate::Result;
use md5::{Digest, Md5};
use crate::ast::{Node, Processor, Return};
use regex::Regex;
use std::collections::HashMap;

pub struct AddSig {
    uniq_option: UniquenessOption,
    sig: Option<String>,
    flow_name: Option<String>,
    model: Model,
    test_suite_names: HashMap<String, usize>,
}

pub fn run(node: &Node<PGM>, model: Model) -> Result<(Node<PGM>, Model)> {
    let mut p = AddSig {
        uniq_option: UniquenessOption::Signature,
        sig: None,
        flow_name: None,
        model: model,
        test_suite_names: HashMap::new(),
    };
    let ast = node.process(&mut p)?.unwrap();

    Ok((ast, p.model))
}

impl Processor<PGM> for AddSig {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Flow(name) => {
                self.flow_name = Some(name.to_owned());
                let mut hasher = Md5::new();
                hasher.update(name);
                // Just to match what O1 did for regression testing
                hasher.update(format!("{}.tf", name));
                if let Some(app_name) = crate::PROG_GEN_CONFIG.app_name() {
                    // Again, to match O1 for regression testing
                    if app_name == "example" {
                        hasher.update("origen_testers");
                    } else {
                        hasher.update(app_name);
                    }
                }
                let hash = hasher.finalize();
                self.sig = Some((&format!("{:X}", hash)[0..7]).to_string());

                let n = node.process_and_update_children(self)?;
                if let Some(sig) = self.uniqueness() {
                    let sig = sig.to_owned();
                    for id in &self.model.flows[name].test_invocations {
                        let t = self.model.test_invocations.get_mut(id).unwrap();
                        let new_name = format!("{}_{}", t.get("name")?.unwrap(), sig);
                        t.set("name", Some(ParamValue::String(new_name)), false)?;
                    }
                }
                Return::Replace(n)
            }
            PGM::Test(id, _) => {
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
            PGM::Condition(cond) => match cond {
                FlowCondition::IfFlag(flags) | FlowCondition::UnlessFlag(flags) => {
                    let flags: Vec<String> = flags
                        .iter()
                        .map(|f| {
                            let f = clean_flag(f);
                            self.add_sig_to_flag(f)
                        })
                        .collect();
                    let children = node.process_and_box_children(self)?;
                    let attrs = match cond {
                        FlowCondition::IfFlag(_) => PGM::Condition(FlowCondition::IfFlag(flags)),
                        FlowCondition::UnlessFlag(_) => {
                            PGM::Condition(FlowCondition::UnlessFlag(flags))
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
                            PGM::Condition(FlowCondition::IfEnable(flags))
                        }
                        FlowCondition::UnlessEnable(_) => {
                            PGM::Condition(FlowCondition::UnlessEnable(flags))
                        }
                        _ => unreachable!(),
                    };
                    Return::Replace(node.updated(Some(attrs), Some(children), None))
                }
                _ => Return::ProcessChildren,
            },
            PGM::SetFlag(flag, state, is_auto_generated) => {
                let flag = {
                    let f = clean_flag(flag);
                    if *is_auto_generated {
                        self.add_sig_to_flag(f)
                    } else {
                        f
                    }
                };
                let children = node.process_and_box_children(self)?;
                Return::Replace(node.updated(
                    Some(PGM::SetFlag(flag, *state, *is_auto_generated)),
                    Some(children),
                    None,
                ))
            }
            _ => Return::ProcessChildren,
        })
    }
}

impl AddSig {
    fn add_sig_to_flag(&self, flag: String) -> String {
        if let Some(sig) = self.uniqueness() {
            let re = Regex::new(r"_(?P<flag>PASSED|FAILED|RAN)$").unwrap();
            let replacement = format!("_{}_$flag", sig);
            let r = re.replace(&flag, &*replacement);
            r.to_string()
        } else {
            flag
        }
    }

    fn uniqueness(&self) -> Option<&str> {
        match &self.uniq_option {
            UniquenessOption::None => None,
            UniquenessOption::Flowname => Some(self.flow_name.as_ref().unwrap()),
            UniquenessOption::Signature => Some(self.sig.as_ref().unwrap()),
            UniquenessOption::String(s) => Some(s),
        }
    }
}

fn clean_flag(flag: &str) -> String {
    if flag.starts_with("$") {
        flag.replacen("$", "", 1)
    } else {
        flag.to_uppercase()
    }
}
