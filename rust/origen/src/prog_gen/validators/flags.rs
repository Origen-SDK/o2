use crate::prog_gen::{FlowCondition, PGM};
use crate::Result;
use origen_metal::ast::{Node, Processor, Return};

pub struct Flags {
    open_if: Vec<Node<PGM>>,
    open_unless: Vec<Node<PGM>>,
    conflicting: Vec<(Node<PGM>, Node<PGM>)>,
    volatile_flags: Vec<String>,
}

pub fn run(node: &Node<PGM>) -> Result<()> {
    let mut p = Flags {
        open_if: vec![],
        open_unless: vec![],
        conflicting: vec![],
        volatile_flags: vec![],
    };
    let _ = node.process(&mut p)?;

    let mut msg = "".to_string();

    if !p.conflicting.is_empty() {
        msg += "\nif_flag and unless_flag conditions cannot be nested and refer to the same flag unless it is declared as volatile.";
        msg += "\nThe following conflicts were found:";
        if crate::STATUS.is_debug_enabled() {
            for (a, b) in &p.conflicting {
                msg += &format!("\n  {} - {}", a, a.meta_string());
                msg += &format!("\n  {} - {}", b, b.meta_string());
            }
        } else {
            msg += "\n  run again with the --debug switch to see them";
        }
        bail!("{}", msg)
    } else {
        Ok(())
    }
}

impl Processor<PGM> for Flags {
    fn on_node(&mut self, node: &Node<PGM>) -> origen_metal::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Volatile(flag) => {
                self.volatile_flags.push(flag.to_owned());
                Return::None
            }
            PGM::Condition(cond) => match cond {
                FlowCondition::IfFlag(flags) => {
                    let flag = flags.first().unwrap();
                    if self.volatile_flags.iter().any(|f| f == flag) {
                        Return::ProcessChildren
                    } else {
                        for n in &self.open_unless {
                            if let PGM::Condition(attrs) = &n.attrs {
                                if let FlowCondition::UnlessFlag(f) = attrs {
                                    if f.first().unwrap() == flag {
                                        self.conflicting.push((n.clone(), node.without_children()));
                                    }
                                }
                            }
                        }
                        self.open_if.push(node.without_children());
                        let _ = node.process_children(self)?;
                        self.open_if.pop();
                        Return::None
                    }
                }
                FlowCondition::UnlessFlag(flags) => {
                    let flag = flags.first().unwrap();
                    if self.volatile_flags.iter().any(|f| f == flag) {
                        Return::ProcessChildren
                    } else {
                        for n in &self.open_if {
                            if let PGM::Condition(attrs) = &n.attrs {
                                if let FlowCondition::IfFlag(f) = attrs {
                                    if f.first().unwrap() == flag {
                                        self.conflicting.push((n.clone(), node.without_children()));
                                    }
                                }
                            }
                        }
                        self.open_unless.push(node.without_children());
                        let _ = node.process_children(self)?;
                        self.open_unless.pop();
                        Return::None
                    }
                }
                _ => Return::ProcessChildren,
            },
            _ => Return::ProcessChildren,
        })
    }
}
