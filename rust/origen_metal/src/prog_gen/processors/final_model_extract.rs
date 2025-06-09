use crate::prog_gen::{
    Bin, BinType, FlowCondition, LimitSelector, Model, VariableOperation, VariableType, PGM,
};
use crate::Result;
use crate::ast::{Node, Processor, Return};

/// This extracts all definitions for tests, test invocations, pattern sets, bins, etc.
/// and converts them into a program model which is returned.
/// The resultant AST has most of the associated nodes removed but is otherwise unchanged.
/// The model is not considered finalized until after the flow generator for the specific ATE
/// target has run, at that point any ATE-specific extraction into the model will be complete,
/// e.g. to extract pattern references made by test objects.
pub struct ExtractToModel {
    model: Model,
}

pub fn run(node: &Node<PGM>, model: Model) -> Result<(Node<PGM>, Model)> {
    let mut p = ExtractToModel { model: model };
    let ast = node.process(&mut p)?.unwrap();
    Ok((ast, p.model))
}

impl Processor<PGM> for ExtractToModel {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::ResourcesFilename(name, kind) => {
                self.model.set_resources_filename(name.to_owned(), kind);
                Return::Unmodified
            }
            PGM::Condition(cond) => {
                match cond {
                    FlowCondition::IfJob(_ids) | FlowCondition::UnlessJob(_ids) => {
                        self.model.record_variable_reference(
                            "JOB".to_string(),
                            VariableType::Job,
                            VariableOperation::Reference,
                        );
                    }
                    FlowCondition::IfEnable(ids) | FlowCondition::UnlessEnable(ids) => {
                        for id in ids {
                            self.model.record_variable_reference(
                                id.to_owned(),
                                VariableType::Enable,
                                VariableOperation::Reference,
                            );
                        }
                    }
                    FlowCondition::IfFlag(ids) | FlowCondition::UnlessFlag(ids) => {
                        for id in ids {
                            self.model.record_variable_reference(
                                id.to_owned(),
                                VariableType::Flag,
                                VariableOperation::Reference,
                            );
                        }
                    }
                    _ => {}
                }
                Return::ProcessChildren
            }
            PGM::SetFlag(name, _, _) => {
                self.model.record_variable_reference(
                    name.to_owned(),
                    VariableType::Flag,
                    VariableOperation::Set,
                );
                Return::ProcessChildren
            }
            PGM::SetLimit(test_id, inv_id, selector, value) => {
                let t = {
                    if let Some(id) = test_id {
                        self.model.tests.get_mut(id)
                    } else if let Some(id) = inv_id {
                        self.model.test_invocations.get_mut(id)
                    } else {
                        None
                    }
                };
                if let Some(t) = t {
                    match selector {
                        LimitSelector::Hi => t.hi_limit = value.to_owned(),
                        LimitSelector::Lo => t.lo_limit = value.to_owned(),
                    }
                }
                Return::None
            }
            //PGM::PatternGroup(id, name, _, kind) => Ok(Return::None),
            //PGM::PushPattern(id, name, start_label) => Ok(Return::None),
            // These will be left in the AST for later consumption by the flow, but they also function
            // as un-official bin definitions.
            // Any official bin defs in the AST for the same bin number will be combined with the model
            // defintion created from here.
            PGM::Bin(hardbin, softbin, kind) => {
                let flow = self.model.get_flow_mut(None);
                if !flow.hardbins.contains_key(hardbin) {
                    let b = Bin {
                        number: *hardbin,
                        description: None,
                        priority: None,
                        pass: matches!(kind, BinType::Good),
                    };
                    flow.hardbins.insert(*hardbin, b);
                }
                if let Some(softbin) = softbin {
                    if !flow.softbins.contains_key(hardbin) {
                        let b = Bin {
                            number: *softbin,
                            description: None,
                            priority: None,
                            pass: matches!(kind, BinType::Good),
                        };
                        flow.softbins.insert(*softbin, b);
                    }
                }
                Return::Unmodified
            }
            PGM::DefBin(number, is_soft, kind, description, priority) => {
                let flow = self.model.get_flow_mut(None);
                let collection = match is_soft {
                    false => &mut flow.hardbins,
                    true => &mut flow.softbins,
                };
                if !collection.contains_key(number) {
                    let b = Bin {
                        number: *number,
                        description: description.to_owned(),
                        priority: priority.to_owned(),
                        pass: matches!(kind, BinType::Good),
                    };
                    collection.insert(*number, b);
                } else {
                    let b = collection.get_mut(number).unwrap();
                    if let Some(d) = description {
                        b.description = Some(d.to_owned());
                    }
                    if let Some(p) = priority {
                        b.priority = Some(*p);
                    }
                }
                Return::None
            }
            _ => Return::ProcessChildren,
        })
    }
}
