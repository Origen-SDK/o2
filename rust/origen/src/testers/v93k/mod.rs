use crate::DUT;
use crate::generator::processor::{Return, Processor};
use crate::generator::ast::{Node, Attrs};
use crate::core::tester::{TesterAPI, Interceptor};
//use crate::current_job;
//use crate::core::producer::{output_file};
use crate::core::file_handler::{File};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Renderer {
  current_timeset_id: Option<usize>,
  output_file: Option<File>,
}

impl Default for Renderer {
  fn default() -> Self {
    Self {
      current_timeset_id: None,
      output_file: None,
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
}

impl Processor for Renderer {
  fn on_node(&mut self, node: &Node) -> Return {
    match &node.attrs {
        Attrs::Test(name) => {
          //self.output_file = output_file!("avc");
          println!("{}", name);
          let mut p = PathBuf::from(name);
          p.set_extension(".avc");
          self.output_file = Some(File::create(p));
          //self.output_file.write(PRODUCER.pattern_header());
          Return::ProcessChildren
        }
        Attrs::Comment(_level, msg) => {
          self.output_file.as_mut().unwrap().write_ln(&format!("# {}", msg));
          Return::Unmodified
        },
        Attrs::Cycle(repeat, _compressable) => {
          let dut = DUT.lock().unwrap();
          let t = &dut.timesets[self.current_timeset_id.unwrap()];
          self.output_file.as_mut().unwrap().write_ln(&format!("R{} {} <Pins> # <EoL Comment>;", repeat, t.name));
          Return::Unmodified
        },
        Attrs::SetTimeset(timeset_id) => {
          self.current_timeset_id = Some(*timeset_id);
          Return::Unmodified      
        },
        _ => Return::ProcessChildren,
    }
  }
}
