use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::FlowCondition;

pub struct Jobs {
    negative: Vec<Node>,
    conflicting: Vec<(Node, Node)>,
    stack: Vec<(Node, bool)>,
}

pub fn run(node: &Node) -> Result<()> {
    let mut p = Jobs {
        negative: vec![],
        conflicting: vec![],
        stack: vec![],
    };
    let _ = node.process(&mut p)?;

    let mut failed = false;
    let mut msg = "".to_string();

    if !p.conflicting.is_empty() {
        msg += "if_job and unless_job conditions cannot both be applied to the same tests. ";
        msg += "The following conflicts were found:";
        if crate::STATUS.is_debug_enabled() {
            for (a, b) in &p.conflicting {
                if let Attrs::PGMCondition(n) = &a.attrs {
                    let a_condition = match matches!(n, FlowCondition::IfJob(_)) {
                        true => "if_job:    ",
                        false => "unless_job:",
                    };
                    msg += &format!("\n  {} {}", a_condition, a.meta_string());
                }
                if let Attrs::PGMCondition(n) = &b.attrs {
                    let b_condition = match matches!(n, FlowCondition::IfJob(_)) {
                        true => "if_job:    ",
                        false => "unless_job:",
                    };
                    msg += &format!("\n  {} {}", b_condition, b.meta_string());
                }
            }
        } else {
            msg += "\n  run again with the --debug switch to see them";
        }
        failed = true;
    }

    if !p.negative.is_empty() {
        msg += "\nJob names should not be negated, use unless_job if you want to specify !JOB";
        msg += "\nThe following negative job names were found:";
        if crate::STATUS.is_debug_enabled() {
            for node in &p.negative {
                if let Attrs::PGMCondition(n) = &node.attrs {
                    match n {
                        FlowCondition::IfJob(jobs) | FlowCondition::UnlessJob(jobs) => {
                            msg += &format!("\n  {} - {}", jobs.join(", "), node.meta_string());
                        }
                        _ => unreachable!(),
                    }
                }
            }
        } else {
            msg += "\n  run again with the --debug switch to see them";
        }
        failed = true
    }

    if failed {
        error!("{}", msg)
    } else {
        Ok(())
    }
}

impl Processor for Jobs {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMCondition(cond) => match cond {
                FlowCondition::IfJob(jobs) | FlowCondition::UnlessJob(jobs) => {
                    let state = matches!(cond, FlowCondition::IfJob(_));
                    if jobs
                        .into_iter()
                        .any(|j| j.starts_with("!") | j.starts_with("~"))
                    {
                        self.negative.push(node.without_children());
                    }
                    if !self.stack.is_empty() && self.stack.last().unwrap().1 != state {
                        self.conflicting.push((
                            self.stack.last().unwrap().0.clone(),
                            node.without_children(),
                        ));
                    } else {
                        self.stack.push((node.without_children(), state));
                        let _ = node.process_children(self)?;
                        self.stack.pop();
                    }
                    Return::None
                }
                _ => Return::ProcessChildren,
            },
            _ => Return::ProcessChildren,
        })
    }
}
