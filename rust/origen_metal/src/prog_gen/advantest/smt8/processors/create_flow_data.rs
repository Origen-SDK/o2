//! Creates a flow data structure for each flow/subflow which summarizes its variable
//! usage (input and output variables)
use std::collections::HashSet;

use crate::ast::*;
use crate::prog_gen::expressions::expression_variables;
use crate::prog_gen::FlowCondition;
use crate::prog_gen::GroupType;
use crate::prog_gen::LoopCount;
use crate::prog_gen::PGM;
use crate::Result;

pub fn run(node: &Node<PGM>) -> Result<Node<PGM>> {
    let mut p = Collector {
        ..Default::default()
    };
    let mut node = node.process(&mut p)?.unwrap();

    let mut p2 = PassthroughCollector {
        ..Default::default()
    };
    node = node.process(&mut p2)?.unwrap();

    Ok(node)
}

#[derive(Default, Serialize, Clone, Debug, PartialEq)]
pub struct FlowData {
    pub references_job: bool,
    pub referenced_flags: HashSet<String>,
    pub referenced_enables: HashSet<String>,
    pub input_job: bool,
    pub input_flags: HashSet<String>,
    pub output_flags: HashSet<String>,
    pub input_enables: HashSet<String>,
    pub modified_flags: HashSet<String>,
    pub auto_flags: HashSet<String>,
    pub downstream_flags: HashSet<String>,
    /// Flow variables, case preserved, which are assigned or read by expressions in this flow
    pub referenced_variables: HashSet<String>,
    /// Flow variables, case preserved, which are passed into this flow
    pub input_variables: HashSet<String>,
    /// Flow variables, case preserved, which this flow declares as outputs, these are also
    /// in modified_flags
    pub declared_outputs: HashSet<String>,
}

impl FlowData {
    pub fn sorted_output_flags(&self) -> Vec<String> {
        let mut flags: Vec<_> = self.output_flags.iter().cloned().collect();
        flags.sort();
        flags
    }

    pub fn sorted_input_flags(&self) -> Vec<String> {
        let mut flags: Vec<_> = self.input_flags.iter().cloned().collect();
        flags.sort();
        flags
    }

    /// Returns a sorted list of all output and modified flags
    pub fn sorted_modified_flags(&self) -> Vec<String> {
        let mut flags: HashSet<String> = HashSet::new();
        for f in &self.output_flags {
            flags.insert(f.to_string());
        }
        for f in &self.modified_flags {
            flags.insert(f.to_string());
        }
        for f in &self.downstream_flags {
            flags.insert(f.to_string());
        }
        let mut flags_vec: Vec<_> = flags.into_iter().collect();
        flags_vec.sort();
        flags_vec
    }

    /// Returns a sorted list of all input variables, including enables, flags, and job
    pub fn sorted_input_vars(&self) -> Vec<String> {
        let mut vars: HashSet<String> = HashSet::new();
        if self.input_job {
            vars.insert("JOB".to_string());
        }
        for f in &self.input_enables {
            vars.insert(f.to_string());
        }
        for f in &self.input_flags {
            vars.insert(f.to_string());
        }
        for v in &self.input_variables {
            vars.insert(v.to_string());
        }
        let mut vars_vec: Vec<_> = vars.into_iter().collect();
        vars_vec.sort();
        vars_vec
    }
}

#[derive(Default)]
pub struct Collector {
    flows: Vec<FlowData>,
    processing_subflow: bool,
    /// Counter variables of the loops which enclose the current node
    loop_variables: Vec<String>,
}

impl Collector {
    fn reference_variable(&mut self, name: &str) {
        if self.loop_variables.iter().any(|v| v == name) {
            return;
        }
        self.flows
            .last_mut()
            .unwrap()
            .referenced_variables
            .insert(name.to_string());
    }
}

impl Processor<PGM> for Collector {
    fn on_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        match &node.attrs {
            PGM::Flow(_) | PGM::SubFlow(_, _) => {
                self.flows.push(FlowData {
                    ..Default::default()
                });
                let orig = self.processing_subflow;
                self.processing_subflow = true;
                let children = node.process_and_box_children(self)?;
                self.processing_subflow = orig;
                // The flow's data, collected from its children, is its first child. This is
                // done here, rather than once the node has been processed, to build the new
                // node from the processed children without cloning them
                return Ok(Return::Replace(self.with_flow_data(node, children)));
            }
            PGM::Group(_, _, kind, _) => {
                if kind == &GroupType::Flow {
                    self.flows.push(FlowData {
                        ..Default::default()
                    });
                }
                let orig = self.processing_subflow;
                self.processing_subflow = true;
                let children = node.process_and_box_children(self)?;
                self.processing_subflow = orig;
                if kind == &GroupType::Flow {
                    return Ok(Return::Replace(self.with_flow_data(node, children)));
                }
                return Ok(Return::Replace(node.replace_children(children)));
            }
            PGM::Test(_, _) | PGM::TestStr(_, _, _, _, _) => {
                let orig = self.processing_subflow;
                self.processing_subflow = false;
                let n = node.process_and_update_children(self)?;
                self.processing_subflow = orig;
                return Ok(Return::Replace(n));
            }
            PGM::OnFailed(_) | PGM::OnPassed(_) => {
                // In this case, the contents of a flow-level on-passed/failed are actually implemented by the parent
                // flow after executing it, so any flags set/referenced here should be applied to the parent flow
                if self.processing_subflow {
                    let f = self.flows.pop().unwrap();
                    let n = node.process_and_update_children(self)?;
                    self.flows.push(f);
                    return Ok(Return::Replace(n));
                }
            }
            PGM::SetFlag(flag, _state, is_auto_generated) => {
                let flag = flag.to_uppercase();
                let current_flow = self.flows.last_mut().unwrap();
                if *is_auto_generated {
                    current_flow.auto_flags.insert(flag.clone());
                }
                current_flow.modified_flags.insert(flag);
            }
            PGM::OutputVariable(name) => {
                // Declared like a flag set by the user, an output of this flow and of every
                // enclosing flow, but keeping the case of its name
                let current_flow = self.flows.last_mut().unwrap();
                current_flow.modified_flags.insert(name.to_string());
                current_flow.declared_outputs.insert(name.to_string());
            }
            PGM::SetVariable(name, value) => {
                self.reference_variable(name);
                for v in expression_variables(value) {
                    self.reference_variable(&v);
                }
            }
            PGM::Loop(count, variable, _, _) => {
                // A count held in a flow variable is read when the loop starts, before its own
                // counter is in scope
                if let Some(LoopCount::Variable(name)) = count {
                    self.reference_variable(name);
                }
                let variable = variable.clone().unwrap_or_else(|| "index".to_string());
                self.loop_variables.push(variable);
                let n = node.process_and_update_children(self)?;
                self.loop_variables.pop();
                return Ok(Return::Replace(n));
            }
            PGM::Condition(cond) => match cond {
                FlowCondition::IfJob(_jobs) | FlowCondition::UnlessJob(_jobs) => {
                    self.flows.last_mut().unwrap().references_job = true;
                }
                FlowCondition::IfEnable(flags) | FlowCondition::UnlessEnable(flags) => {
                    for f in flags {
                        let flag = f.to_uppercase();
                        self.flows
                            .last_mut()
                            .unwrap()
                            .referenced_enables
                            .insert(flag);
                    }
                }
                FlowCondition::IfFlag(flags) | FlowCondition::UnlessFlag(flags) => {
                    for f in flags {
                        let flag = f.to_uppercase();
                        self.flows.last_mut().unwrap().referenced_flags.insert(flag);
                    }
                }
                FlowCondition::IfExpr(expression) | FlowCondition::UnlessExpr(expression) => {
                    for v in expression_variables(expression) {
                        self.reference_variable(&v);
                    }
                }
                _ => {}
            },
            // For all other nodes
            _ => {}
        }
        Ok(Return::ProcessChildren)
    }
}

impl Collector {
    /// Returns a copy of the given flow node with the given (processed) children, preceded by
    /// the flow's data
    fn with_flow_data(&mut self, node: &Node<PGM>, children: Vec<Box<Node<PGM>>>) -> Node<PGM> {
        let fdata = self.flows.pop().unwrap();
        let mut new_children = Vec::with_capacity(children.len() + 1);
        new_children.push(Box::new(node!(PGM::FlowData, Box::new(fdata))));
        new_children.extend(children);
        node.updated(None, Some(new_children), None)
    }
}

#[derive(Default)]
pub struct PassthroughCollector {
    flow_stack: Vec<FlowData>,
}

impl Processor<PGM> for PassthroughCollector {
    fn on_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        let is_flow = match &node.attrs {
            PGM::Flow(_) | PGM::SubFlow(_, _) => true,
            PGM::Group(_, _, kind, _) => kind == &GroupType::Flow,
            _ => false,
        };
        if !is_flow {
            return Ok(Return::ProcessChildren);
        }
        if let Some(first_child) = node.children.first() {
            if let PGM::FlowData(fdata) = &first_child.attrs {
                self.flow_stack.push((**fdata).clone());
            }
        }
        // The flow's finalized data replaces its first child once its children have been
        // processed, the new node is built here from the processed children without cloning
        // them
        let mut children = node.process_and_box_children(self)?;
        let flow_data = self.finalize_flow_data();
        children.remove(0);
        children.insert(0, Box::new(node!(PGM::FlowData, Box::new(flow_data))));
        Ok(Return::Replace(node.updated(None, Some(children), None)))
    }
}

impl PassthroughCollector {
    fn finalize_flow_data(&mut self) -> FlowData {
        // Extract the FlowData from the first child
        let mut flow_data = self.flow_stack.pop().unwrap();

        if flow_data.references_job {
            // Ensure every flow in the stack passses through job
            flow_data.input_job = true;
            for f in &mut self.flow_stack {
                f.input_job = true;
            }
        }
        for en in &flow_data.referenced_enables {
            flow_data.input_enables.insert(en.to_string());
            // Ensure every flow in the stack passses through referenced enables
            for f in &mut self.flow_stack {
                f.input_enables.insert(en.to_string());
            }
        }
        for v in &flow_data.referenced_variables {
            // A flag set by this flow is declared as a flag, not as an input variable
            if flow_data.modified_flags.contains(v) {
                continue;
            }
            flow_data.input_variables.insert(v.to_string());
            // Ensure every flow in the stack passes through referenced variables
            for f in &mut self.flow_stack {
                f.input_variables.insert(v.to_string());
            }
        }
        // Make sure that all referenced flags that originate from an upstream flow are input
        for fl in &flow_data.referenced_flags {
            if flow_data.modified_flags.contains(fl)
                || flow_data.output_flags.contains(fl)
                || flow_data.downstream_flags.contains(fl)
            {
                continue;
            }
            flow_data.input_flags.insert(fl.to_string());
            // Need to get it from upstream, work backwards up the stack until we find a flow that sets it,
            // otherwise we need to pass it all the way through
            for f in self.flow_stack.iter_mut().rev() {
                f.input_flags.insert(fl.to_string());
                if f.modified_flags.contains(fl) {
                    break;
                }
            }
        }
        // Make sure that any modified flags that are referenced later by an upstream flow are output
        for fl in &flow_data.modified_flags {
            let mut passthrough = false;
            for f in &mut self.flow_stack {
                if !passthrough && f.referenced_flags.contains(fl) {
                    passthrough = true;
                    f.downstream_flags.insert(fl.to_string());
                    flow_data.output_flags.insert(fl.to_string());
                } else if passthrough {
                    flow_data.output_flags.insert(fl.to_string());
                    f.output_flags.insert(fl.to_string());
                }
            }
        }
        // Also output any manually set flags (maybe this should be opted-in in the future, e.g. via extern=True)
        for fl in &flow_data.modified_flags {
            // A declared output is output even when a flag generated by Origen has its name
            if flow_data.auto_flags.contains(fl) && !flow_data.declared_outputs.contains(fl) {
                continue;
            }
            flow_data.output_flags.insert(fl.to_string());
            for f in &mut self.flow_stack {
                f.output_flags.insert(fl.to_string());
            }
        }
        flow_data
    }
}
