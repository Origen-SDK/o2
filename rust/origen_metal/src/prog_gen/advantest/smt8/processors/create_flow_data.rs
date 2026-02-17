//! Creates a flow data structure for each flow/subflow which summarizes its variable
//! usage (input and output variables)
use std::collections::HashSet;

use crate::prog_gen::FlowCondition;
use crate::prog_gen::GroupType;
use crate::prog_gen::PGM;
use crate::ast::*;
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
        let mut vars_vec: Vec<_> = vars.into_iter().collect();
        vars_vec.sort();
        vars_vec
    }
}

#[derive(Default)]
pub struct Collector {
    flows: Vec<FlowData>,
    processing_subflow: bool,
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
                let n = node.process_and_update_children(self)?;
                self.processing_subflow = orig;
                return Ok(Return::Replace(n));
            }
            PGM::Group(_, _, kind, _) => {
                if kind == &GroupType::Flow {
                    self.flows.push(FlowData { 
                        ..Default::default()
                    });
                }
                let orig = self.processing_subflow;
                self.processing_subflow = true;
                let n = node.process_and_update_children(self)?;
                self.processing_subflow = orig;
                return Ok(Return::Replace(n));
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
            PGM::Condition(cond) => match cond {
                FlowCondition::IfJob(_jobs) | FlowCondition::UnlessJob(_jobs) => {
                    self.flows.last_mut().unwrap().references_job = true;
                }
                FlowCondition::IfEnable(flags) | FlowCondition::UnlessEnable(flags) => {
                    for f in flags {
                        let flag = f.to_uppercase();
                        self.flows.last_mut().unwrap().referenced_enables.insert(flag);
                    }
                }
                FlowCondition::IfFlag(flags) | FlowCondition::UnlessFlag(flags) => {
                    for f in flags {
                        let flag = f.to_uppercase();
                        self.flows.last_mut().unwrap().referenced_flags.insert(flag);
                    }
                }
                _ => {}
            },
            // For all other nodes
            _ => {}
        }
        Ok(Return::ProcessChildren)
    }

    fn on_processed_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        match &node.attrs {
            PGM::Flow(_) | PGM::SubFlow(_, _) => {
                let fdata = self.flows.pop().unwrap();
                let mut children = vec![Box::new(node!(PGM::FlowData, fdata))];
                children.extend(node.children.clone());
                Ok(Return::Replace(node.updated(None, Some(children), None)))
            }
            PGM::Group(_, _, kind, _) => {
                if kind == &GroupType::Flow {
                    let fdata = self.flows.pop().unwrap();
                    let mut children = vec![Box::new(node!(PGM::FlowData, fdata))];
                    children.extend(node.children.clone());
                    Ok(Return::Replace(node.updated(None, Some(children), None)))
                } else {
                    Ok(Return::Unmodified)
                }
            }
            _ => {
                Ok(Return::Unmodified)
            }
        }
    }
}

#[derive(Default)]
pub struct PassthroughCollector {
    flow_stack: Vec<FlowData>,
}

impl Processor<PGM> for PassthroughCollector {
    fn on_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        match &node.attrs {
            PGM::Flow(_) | PGM::SubFlow(_, _) => {
                if let Some(first_child) = node.children.first() {
                    if let PGM::FlowData(fdata) = &first_child.attrs {
                        self.flow_stack.push(fdata.clone());
                    }
                }
            }
            PGM::Group(_, _, kind, _) => {
                if kind == &GroupType::Flow {
                    if let Some(first_child) = node.children.first() {
                        if let PGM::FlowData(fdata) = &first_child.attrs {
                            self.flow_stack.push(fdata.clone());
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(Return::ProcessChildren)
    }

    fn on_processed_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        match &node.attrs {
            PGM::Flow(_) | PGM::SubFlow(_, _) => {
                let flow_data = self.finalize_flow_data();

                let mut children = node.children.clone();
                children.remove(0);
                children.insert(0, Box::new(node!(PGM::FlowData, flow_data)));
                Ok(Return::Replace(node.updated(None, Some(children), None)))
            }
            PGM::Group(_, _, kind, _) => {
                if kind == &GroupType::Flow {
                    let flow_data = self.finalize_flow_data();

                    let mut children = node.children.clone();
                    children.remove(0);
                    children.insert(0, Box::new(node!(PGM::FlowData, flow_data)));
                    Ok(Return::Replace(node.updated(None, Some(children), None)))
                } else {
                    Ok(Return::Unmodified)
                }
            }
            _ => Ok(Return::Unmodified)
        }
    }
}

impl PassthroughCollector {
    fn finalize_flow_data(&mut self) -> FlowData{
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
        // Make sure that all referenced flags that originate from an upstream flow are input
        for fl in &flow_data.referenced_flags {
            if flow_data.modified_flags.contains(fl) || flow_data.output_flags.contains(fl) || flow_data.downstream_flags.contains(fl) {
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
            if flow_data.auto_flags.contains(fl) {
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