use crate::ast::{Node, Processor, Return};
use crate::prog_gen::processors::{declared_output_variables, flag_reads_output};
use crate::prog_gen::{FlowCondition, PGM};
use crate::Result;

pub struct Flags {
    open_if: Vec<Node<PGM>>,
    open_unless: Vec<Node<PGM>>,
    conflicting: Vec<(Node<PGM>, Node<PGM>)>,
    volatile_flags: Vec<String>,
    /// Variables declared as outputs, which tests may change natively like volatile flags
    outputs: Vec<String>,
}

pub fn run(node: &Node<PGM>) -> Result<()> {
    let mut p = Flags {
        open_if: vec![],
        open_unless: vec![],
        conflicting: vec![],
        volatile_flags: vec![],
        outputs: declared_output_variables(node),
    };
    node.walk(&mut p)?;

    let mut msg = "".to_string();

    if !p.conflicting.is_empty() {
        msg += "\nif_flag and unless_flag conditions cannot be nested and refer to the same flag unless it is declared as volatile or as an output variable.";
        msg += "\nThe following conflicts were found:";
        if crate::PROG_GEN_CONFIG.debug_enabled() {
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

impl Flags {
    /// Returns true if tests may change the given flag, it is volatile or a declared output
    fn is_volatile(&self, flag: &str) -> bool {
        self.volatile_flags.iter().any(|f| f == flag) || flag_reads_output(flag, &self.outputs)
    }
}

impl Processor<PGM> for Flags {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Volatile(flag) => {
                self.volatile_flags.push(flag.to_owned());
                Return::None
            }
            PGM::Condition(cond) => match cond {
                FlowCondition::IfFlag(flags) => {
                    let flag = flags.first().unwrap();
                    if self.is_volatile(flag) {
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
                        node.walk_children(self)?;
                        self.open_if.pop();
                        Return::None
                    }
                }
                FlowCondition::UnlessFlag(flags) => {
                    let flag = flags.first().unwrap();
                    if self.is_volatile(flag) {
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
                        node.walk_children(self)?;
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

#[cfg(test)]
mod tests {
    use super::run;
    use crate::prog_gen::{FlowCondition, FlowID, PGM};

    fn nested(declared: Option<&str>, read_as: &str) -> crate::ast::Node<PGM> {
        let mut flow = node!(PGM::Flow, "f1".to_string() =>
            node!(PGM::Condition, FlowCondition::IfFlag(vec![read_as.to_string()]) =>
                node!(PGM::Test, 1, FlowID::from_int(1)),
                node!(PGM::Condition, FlowCondition::UnlessFlag(vec![read_as.to_string()]) =>
                    node!(PGM::Test, 2, FlowID::from_int(2)),
                ),
            ),
        );
        if let Some(name) = declared {
            flow.add_child(node!(PGM::OutputVariable, name.to_string()));
        }
        flow
    }

    #[test]
    fn nested_opposite_conditions_on_a_declared_output_are_allowed() {
        // A test may change a declared output natively, like a volatile flag
        assert!(run(&nested(None, "DEBUG")).is_err());
        assert!(run(&nested(Some("DEBUG"), "DEBUG")).is_ok());
        // Flags are rendered upper case, so this reads the declared DEBUG
        assert!(run(&nested(Some("DEBUG"), "debug")).is_ok());
        // A mixed-case declaration is a different variable from the upper-case flag
        assert!(run(&nested(Some("debugMode"), "debugmode")).is_err());
    }
}
