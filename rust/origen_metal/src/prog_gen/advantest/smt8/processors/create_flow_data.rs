//! Creates a flow data structure for each flow/subflow which summarizes its variable
//! usage (input and output variables)
use std::collections::{HashMap, HashSet};

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
    /// Flags which this flow sets true itself, unlike modified_flags this does not include the
    /// declared outputs which it does not set, or the flags which it only clears. Only used to
    /// render the flow, it is not serialized with the AST.
    #[serde(skip_serializing)]
    pub set_flags: HashSet<String>,
    pub auto_flags: HashSet<String>,
    pub downstream_flags: HashSet<String>,
    /// Flow variables, case preserved, which are assigned or read by expressions in this flow
    pub referenced_variables: HashSet<String>,
    /// Flow variables, case preserved, which are passed into this flow
    pub input_variables: HashSet<String>,
    /// Flow variables, case preserved, which this flow declares as outputs, these are also
    /// in modified_flags
    pub declared_outputs: HashSet<String>,
    /// Flags which this flow declares local, for itself and its sub-flows
    pub local_flags: HashSet<String>,
    /// Flow variables, case preserved, which this flow declares local, for itself and its
    /// sub-flows. Like the fields which follow, it is not serialized with the AST.
    #[serde(skip_serializing)]
    pub local_variables: HashSet<String>,
    /// Flow variables, case preserved, which this flow assigns
    #[serde(skip_serializing)]
    pub assigned_variables: HashSet<String>,
    /// Flow variables, case preserved, which this flow reads, unlike referenced_variables this
    /// does not include those which it only assigns
    #[serde(skip_serializing)]
    pub read_variables: HashSet<String>,
    /// The local variables which this flow assigns, these are in modified_flags too
    #[serde(skip_serializing)]
    pub held_local_variables: HashSet<String>,
    /// The local variables which a sub-flow assigns and which are passed out to this flow
    #[serde(skip_serializing)]
    pub routed_local_variables: HashSet<String>,
    /// As flag_readers, for the variables read by flows, not counting the flows which only
    /// assign them
    #[serde(skip_serializing)]
    pub variable_readers: HashMap<String, usize>,
    /// The number of flows within this flow, including itself, which read each flag (or a
    /// variable with the same name)
    pub flag_readers: HashMap<String, usize>,
    /// The counter variables of the loops in the parent flow which enclose the call of this
    /// flow, the parent passes their values down
    pub call_site_loop_variables: HashSet<String>,
}

impl FlowData {
    /// Returns the number of flows within this flow, including itself, which read the given flag
    fn readers_of(&self, flag: &str) -> usize {
        self.flag_readers.get(flag).copied().unwrap_or(0)
    }

    /// Returns the number of flows within this flow, including itself, which read the given
    /// variable, not counting those which only assign it
    fn variable_readers_of(&self, variable: &str) -> usize {
        self.variable_readers.get(variable).copied().unwrap_or(0)
    }

    /// Returns true if this flow holds a value for the given flag or variable which it can pass
    /// down: it sets it, or an earlier sub-flow has passed it up to this flow
    fn holds(&self, name: &str) -> bool {
        self.modified_flags.contains(name)
            || self.output_flags.contains(name)
            || self.downstream_flags.contains(name)
    }

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
    /// Counter variables of the loops which enclose the current node within the current flow
    loop_variables: Vec<String>,
    /// The loop counter variables of the enclosing flows, saved while a sub-flow is processed
    saved_loop_variables: Vec<Vec<String>>,
}

impl Collector {
    /// Starts collecting the data of a new flow, a loop counter of its parent flow is not in
    /// scope within it, it is an input passed down by the parent
    fn open_flow(&mut self) {
        let call_site_loop_variables = std::mem::take(&mut self.loop_variables);
        self.flows.push(FlowData {
            call_site_loop_variables: call_site_loop_variables.iter().cloned().collect(),
            ..Default::default()
        });
        self.saved_loop_variables.push(call_site_loop_variables);
    }

    /// Records a variable read by the current flow
    fn reference_variable(&mut self, name: &str) {
        if self.loop_variables.iter().any(|v| v == name) {
            return;
        }
        let current_flow = self.flows.last_mut().unwrap();
        current_flow.referenced_variables.insert(name.to_string());
        current_flow.read_variables.insert(name.to_string());
    }

    /// Records a variable assigned by the current flow, it is referenced by it too
    fn assign_variable(&mut self, name: &str) {
        if self.loop_variables.iter().any(|v| v == name) {
            return;
        }
        let current_flow = self.flows.last_mut().unwrap();
        current_flow.referenced_variables.insert(name.to_string());
        current_flow.assigned_variables.insert(name.to_string());
    }
}

impl Processor<PGM> for Collector {
    fn on_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        match &node.attrs {
            PGM::Flow(_) | PGM::SubFlow(_, _) => {
                self.open_flow();
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
                    self.open_flow();
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
            PGM::SetFlag(flag, state, is_auto_generated) => {
                let flag = flag.to_uppercase();
                let current_flow = self.flows.last_mut().unwrap();
                if *is_auto_generated {
                    current_flow.auto_flags.insert(flag.clone());
                }
                // As Origen 1, which initializes the flags a flow sets, not those it only clears
                if *state {
                    current_flow.set_flags.insert(flag.clone());
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
            PGM::LocalFlag(name) => {
                self.flows
                    .last_mut()
                    .unwrap()
                    .local_flags
                    .insert(name.to_uppercase());
            }
            PGM::LocalVariable(name) => {
                self.flows
                    .last_mut()
                    .unwrap()
                    .local_variables
                    .insert(name.to_string());
            }
            PGM::InputVariable(name) => {
                // Declared like a variable read by an expression, an input of this flow and of
                // every enclosing flow, keeping the case of its name
                self.reference_variable(name);
            }
            PGM::SetVariable(name, value) => {
                self.assign_variable(name);
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
        let mut fdata = self.flows.pop().unwrap();
        self.loop_variables = self.saved_loop_variables.pop().unwrap();
        // Count this flow as a reader of the flags it reads, its sub-flows have already been
        // counted, and add its counts to those of its parent
        let read: HashSet<&String> = fdata
            .referenced_flags
            .iter()
            .chain(fdata.referenced_variables.iter())
            .collect();
        for flag in read {
            *fdata.flag_readers.entry(flag.to_string()).or_insert(0) += 1;
        }
        // And likewise of the variables it reads, including by a flag condition on the same name,
        // not counting those which it only assigns, for local variables
        let read_variables: HashSet<&String> = fdata
            .read_variables
            .iter()
            .chain(fdata.referenced_flags.iter())
            .collect();
        for variable in read_variables {
            *fdata
                .variable_readers
                .entry(variable.to_string())
                .or_insert(0) += 1;
        }
        if let Some(parent) = self.flows.last_mut() {
            for (flag, count) in &fdata.flag_readers {
                *parent.flag_readers.entry(flag.to_string()).or_insert(0) += count;
            }
            for (variable, count) in &fdata.variable_readers {
                *parent
                    .variable_readers
                    .entry(variable.to_string())
                    .or_insert(0) += count;
            }
        }
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
                let mut fdata = (**fdata).clone();
                self.hold_local_variables(&mut fdata);
                self.flow_stack.push(fdata);
            }
        }
        // The flow's finalized data replaces its first child once its children have been
        // processed, the new node is built here from the processed children without cloning
        // them
        let mut children = node.process_and_box_children(self)?;
        let flow_data = self.finalize_flow_data()?;
        children.remove(0);
        children.insert(0, Box::new(node!(PGM::FlowData, Box::new(flow_data))));
        Ok(Return::Replace(node.updated(None, Some(children), None)))
    }
}

impl PassthroughCollector {
    fn finalize_flow_data(&mut self) -> Result<FlowData> {
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
            // A flag this flow sets, or which a sub-flow has passed up to it, is declared as a
            // flag, not as an input variable
            if flow_data.holds(v) {
                continue;
            }
            flow_data.input_variables.insert(v.to_string());
            // Pass it in from upstream, as far as a flow which holds it, or which calls the flows
            // which read it from within a loop counting with it, otherwise all the way through
            let mut from_caller_loop = flow_data.call_site_loop_variables.contains(v);
            for f in self.flow_stack.iter_mut().rev() {
                if from_caller_loop || f.holds(v) {
                    break;
                }
                f.input_variables.insert(v.to_string());
                from_caller_loop = f.call_site_loop_variables.contains(v);
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
            // Need to get it from upstream, work backwards up the stack until we find a flow that
            // holds it, i.e. sets it or has been passed it by an earlier sub-flow, that flow passes
            // its own variable down. Otherwise it is passed all the way through.
            for f in self.flow_stack.iter_mut().rev() {
                if f.holds(fl) {
                    break;
                }
                f.input_flags.insert(fl.to_string());
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
        // Also output any manually set flags, to every enclosing flow unless they are declared local
        for fl in &flow_data.modified_flags {
            // A declared output is output even when a flag generated by Origen has its name
            if flow_data.auto_flags.contains(fl) && !flow_data.declared_outputs.contains(fl) {
                continue;
            }
            flow_data.output_flags.insert(fl.to_string());
            if flow_data.held_local_variables.contains(fl) {
                // A local variable which this flow assigns, passed out to every enclosing flow
                // when it is declared as an output too
                if !self.is_declared_output(&flow_data, fl) {
                    self.pass_out_local(&flow_data, fl, true);
                    continue;
                }
            } else if self.is_local(&flow_data, fl) {
                self.pass_out_local(&flow_data, fl, false);
                continue;
            }
            for f in &mut self.flow_stack {
                f.output_flags.insert(fl.to_string());
            }
        }
        self.check_local_variables(&flow_data)?;
        Ok(flow_data)
    }

    /// Returns true if the given flag, set by the given flow, is declared local by it or by an
    /// enclosing flow, and not declared as an output by any of them
    fn is_local(&self, flow_data: &FlowData, flag: &str) -> bool {
        let flows = || self.flow_stack.iter().chain(std::iter::once(flow_data));
        flows().any(|f| f.local_flags.contains(flag))
            && !flows().any(|f| f.declared_outputs.contains(flag))
    }

    /// A local variable which the given flow assigns is set by it, like a flag, rather than
    /// being its input. This is applied as the flow is entered, before its sub-flows are
    /// finalized, so that a sub-flow which reads the variable gets it from this flow, which
    /// holds it, rather than from further up.
    fn hold_local_variables(&self, flow_data: &mut FlowData) {
        let local_variables: Vec<String> = flow_data
            .assigned_variables
            .iter()
            .filter(|v| self.is_local_variable(flow_data, v))
            .cloned()
            .collect();
        for v in local_variables {
            flow_data.modified_flags.insert(v.clone());
            flow_data.held_local_variables.insert(v);
        }
    }

    /// Returns an error if a local variable which the given flow assigns, or which is passed out
    /// to it, has the same name as an input which it needs, an enable or JOB, which it cannot also
    /// hold
    fn check_local_variables(&self, flow_data: &FlowData) -> Result<()> {
        let mut variables: Vec<&String> = flow_data
            .held_local_variables
            .iter()
            .chain(flow_data.routed_local_variables.iter())
            .collect();
        variables.sort();
        for v in variables {
            if flow_data.input_enables.contains(v) || (flow_data.input_job && v == "JOB") {
                bail!(
                    "The local variable '{}' is held by a flow which also needs an input of the \
                     same name, an enable or JOB, the variable or the input must be renamed",
                    v
                );
            }
        }
        Ok(())
    }

    /// Returns true if the given variable is declared as an output by the given flow or by an
    /// enclosing flow
    fn is_declared_output(&self, flow_data: &FlowData, variable: &str) -> bool {
        self.flow_stack
            .iter()
            .chain(std::iter::once(flow_data))
            .any(|f| f.declared_outputs.contains(variable))
    }

    /// Returns true if the given variable is declared local by the given flow or by an enclosing
    /// flow
    fn is_local_variable(&self, flow_data: &FlowData, variable: &str) -> bool {
        self.flow_stack
            .iter()
            .chain(std::iter::once(flow_data))
            .any(|f| f.local_variables.contains(variable))
    }

    /// Passes a local flag set by the given flow out as far as the highest enclosing flow in
    /// which another flow reads it, that flow holds it and passes it down to its readers
    fn pass_out_local(&mut self, flow_data: &FlowData, flag: &str, variable: bool) {
        let readers_of = |f: &FlowData| {
            if variable {
                f.variable_readers_of(flag)
            } else {
                f.readers_of(flag)
            }
        };
        // Working up from the setter, an enclosing flow has another reader when it contains more
        // readers than its sub-flow which contains the setter
        let mut holder = None;
        let mut readers_on_path = readers_of(flow_data);
        for (i, f) in self.flow_stack.iter().enumerate().rev() {
            let readers = readers_of(f);
            if readers > readers_on_path {
                holder = Some(i);
            }
            readers_on_path = readers;
        }
        if let Some(i) = holder {
            self.flow_stack[i].downstream_flags.insert(flag.to_string());
            for f in &mut self.flow_stack[i + 1..] {
                f.output_flags.insert(flag.to_string());
            }
            if variable {
                for f in &mut self.flow_stack[i..] {
                    f.routed_local_variables.insert(flag.to_string());
                }
            }
        }
    }
}
