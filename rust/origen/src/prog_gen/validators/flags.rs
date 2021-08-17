use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::FlowCondition;

pub struct Flags {
    open_if: Vec<Node>,
    open_unless: Vec<Node>,
    conflicting: Vec<(Node, Node)>,
    volatile_flags: Vec<String>,
}

pub fn run(node: &Node) -> Result<()> {
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
        error!("{}", msg)
    } else {
        Ok(())
    }
}

impl Processor for Flags {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        Ok(match &node.attrs {
            Attrs::PGMVolatile(flag) => {
                self.volatile_flags.push(flag.to_owned());
                Return::None
            }
            Attrs::PGMCondition(cond) => match cond {
                FlowCondition::IfFlag(flags) => {
                    let flag = flags.first().unwrap();
                    if self.volatile_flags.iter().any(|f| f == flag) {
                        Return::ProcessChildren
                    } else {
                        for n in &self.open_unless {
                            if let Attrs::PGMCondition(attrs) = &n.attrs {
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
                            if let Attrs::PGMCondition(attrs) = &n.attrs {
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
