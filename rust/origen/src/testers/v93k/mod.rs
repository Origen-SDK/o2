use crate::{DUT, TEST};
use crate::core::dut::Dut;
use crate::core::application::output_directory;
use crate::generator::processor::{Return, Processor};
use crate::generator::ast::{Node, Attrs};
use crate::core::tester::{TesterAPI, Interceptor};
use crate::core::file_handler::{File};
use crate::core::model::pins::StateTracker;
use crate::core::model::pins::pin::PinActions;
use std::collections::HashMap;
use indexmap::IndexMap;
use std::path::PathBuf;

use crate::generator::processors::CycleCombiner;
use crate::generator::processors::PinActionCombiner;

#[derive(Debug, Clone)]
pub struct Renderer {
  current_timeset_id: Option<usize>,
  output_file: Option<File>,
  states: Option<StateTracker>,
  pin_header_printed: bool,
  pin_header_id: Option<usize>,
}

impl Renderer {
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

  fn update_states(&mut self, pin_changes: &HashMap<String, (PinActions, u8)>, dut: &Dut) -> Return {
    let s = self.states(dut);
    for (name, changes) in pin_changes.iter() {
      s.update(name, Some(changes.0), Some(changes.1), dut);
    }
    Return::Unmodified
  }
}

impl Default for Renderer {
  fn default() -> Self {
    Self {
      current_timeset_id: None,
      output_file: None,
      states: None,
      pin_header_printed: false,
      pin_header_id: None,
    }
  }
}

impl Interceptor for Renderer {}
impl TesterAPI for Renderer {
  fn name(&self) -> String {
    "V93K_ST7".to_string()
  }

  fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
    Box::new(std::clone::Clone::clone(self))
  }

  fn run(&mut self, node: &Node) -> Node {
    node.process(self).unwrap()
  }

  fn preprocess(&mut self, node: &Node) -> Node {
    let mut n = PinActionCombiner::run(node);
    n = CycleCombiner::run(&n);
    n
  }
}

impl Processor for Renderer {
  fn on_node(&mut self, node: &Node) -> Return {
    match &node.attrs {
        Attrs::Test(name) => {
          //self.output_file = output_file!("avc");
          println!("{}", name);
          let mut p = output_directory();
          p.push(self.name());
          p.push(name);
          p.set_extension(".avc");
          self.output_file = Some(File::create(p));
          //self.output_file.write(PRODUCER.pattern_header());
          Return::ProcessChildren
        }
        Attrs::Comment(_level, msg) => {
          self.output_file.as_mut().unwrap().write_ln(&format!("# {}", msg));
          Return::Unmodified
        },
        Attrs::PinAction(pin_changes) => {
          let dut = DUT.lock().unwrap();
          return self.update_states(pin_changes, &dut);
        },
        Attrs::Cycle(repeat, _compressable) => {
          let dut = DUT.lock().unwrap();
          let t = &dut.timesets[self.current_timeset_id.unwrap()];

          if !self.pin_header_printed {
            let pins = self.states(&dut).names().join(" ");
            self.output_file.as_mut().unwrap().write_ln(&format!("FORMAT {}", pins));
            self.pin_header_printed = true;
          }

          self.output_file.as_mut().unwrap().write_ln(&format!("R{} {} {} # <EoL Comment>;",
            repeat,
            t.name,

            // The pin states should have been previously updated from the PinAction node, or just has default values
            self.states.as_ref().unwrap().as_strings().unwrap().join(" ")
          ));
          Return::Unmodified
        },
        Attrs::SetTimeset(timeset_id) => {
          self.current_timeset_id = Some(*timeset_id);
          Return::Unmodified
        },
        Attrs::SetPinHeader(pin_header_id) => {
          self.pin_header_id = Some(*pin_header_id);
          Return::Unmodified
        }
        _ => Return::ProcessChildren,
    }
  }
}
