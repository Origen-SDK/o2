use super::SMT7;
use crate::core::application::output_directory;
use crate::core::dut::Dut;
use crate::core::file_handler::File;
use crate::core::model::pins::pin::PinActions;
use crate::core::model::pins::StateTracker;
use crate::core::tester::TesterAPI;
use crate::generator::ast::{Attrs, Node};
use crate::generator::processor::{Processor, Return};
use crate::{Result, DUT};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::generator::processors::{CycleCombiner, FlattenText, PinActionCombiner};

#[derive(Debug, Clone)]
pub struct Renderer<'a> {
    tester: &'a SMT7,
    current_timeset_id: Option<usize>,
    path: Option<PathBuf>,
    output_file: Option<File>,
    states: Option<StateTracker>,
    pin_header_printed: bool,
    pin_header_id: Option<usize>,
}

impl<'a> Renderer<'a> {
    pub fn run(tester: &'a SMT7, ast: &Node) -> Result<Vec<PathBuf>> {
        let mut n = PinActionCombiner::run(ast)?;
        n = CycleCombiner::run(&n)?;
        n = FlattenText::run(&n)?;
        let mut p = Self::new(tester);
        n.process(&mut p)?;
        Ok(vec![p.path.unwrap()])
    }

    fn new(tester: &'a SMT7) -> Self {
        Self {
            tester: tester,
            current_timeset_id: None,
            path: None,
            output_file: None,
            states: None,
            pin_header_printed: false,
            pin_header_id: None,
        }
    }

    fn states(&mut self, dut: &Dut) -> &mut StateTracker {
        if self.states.is_none() {
            let model_id;
            if let Some(id) = self.pin_header_id {
                model_id = dut.pin_headers[id].model_id;
            } else {
                model_id = 0;
            }
            self.states = Some(StateTracker::new(model_id, self.pin_header_id, dut));
        }
        self.states.as_mut().unwrap()
    }

    fn update_states(
        &mut self,
        pin_changes: &HashMap<String, (PinActions, u8)>,
        dut: &Dut,
    ) -> Result<Return> {
        let s = self.states(dut);
        for (name, changes) in pin_changes.iter() {
            s.update(name, Some(changes.0), Some(changes.1), dut)?;
        }
        Ok(Return::Unmodified)
    }
}

impl<'a> Processor for Renderer<'a> {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::Test(name) => {
                let mut p = output_directory();
                p.push(self.tester.name());
                p.push(name);
                p.set_extension("avc");
                self.path = Some(p.clone());
                self.output_file = Some(File::create(p));
                Ok(Return::ProcessChildren)
            }
            Attrs::Comment(_level, msg) => {
                self.output_file
                    .as_mut()
                    .unwrap()
                    .write_ln(&format!("# {}", msg));
                Ok(Return::Unmodified)
            }
            Attrs::Text(text) => {
                self.output_file
                    .as_mut()
                    .unwrap()
                    .write_ln(&format!("# {}", text));
                Ok(Return::Unmodified)
            }
            Attrs::PatternHeader => Ok(Return::ProcessChildren),
            Attrs::PinAction(pin_changes) => {
                let dut = DUT.lock().unwrap();
                return self.update_states(pin_changes, &dut);
            }
            Attrs::Cycle(repeat, _compressable) => {
                let dut = DUT.lock().unwrap();
                let t = &dut.timesets[self.current_timeset_id.unwrap()];

                if !self.pin_header_printed {
                    let pins = self.states(&dut).names().join(" ");
                    self.output_file
                        .as_mut()
                        .unwrap()
                        .write_ln(&format!("FORMAT {}", pins));
                    self.pin_header_printed = true;
                }

                if !self.pin_header_printed {
                    let pins = self.states(&dut).names().join(" ");
                    self.output_file
                        .as_mut()
                        .unwrap()
                        .write_ln(&format!("FORMAT {}", pins));
                    self.pin_header_printed = true;
                }

                self.output_file.as_mut().unwrap().write_ln(&format!(
                    "R{} {} {} # <EoL Comment>;",
                    repeat,
                    t.name,
                    // The pin states should have been previously updated from the PinAction node, or just has default values
                    self.states
                        .as_ref()
                        .unwrap()
                        .as_strings()
                        .unwrap()
                        .join(" ")
                ));
                Ok(Return::Unmodified)
            }
            Attrs::SetTimeset(timeset_id) => {
                self.current_timeset_id = Some(*timeset_id);
                Ok(Return::Unmodified)
            }
            Attrs::SetPinHeader(pin_header_id) => {
                self.pin_header_id = Some(*pin_header_id);
                Ok(Return::Unmodified)
            }
            Attrs::PatternEnd => {
                self.output_file.as_mut().unwrap().write_ln("SQPG STOP;");
                Ok(Return::Unmodified)
            }
            _ => Ok(Return::ProcessChildren),
        }
    }
}
