use crate::DUT;
use crate::generator::processor::{Return, Processor};
use crate::generator::ast::{Node, Attrs};
use crate::core::tester::{TesterAPI, Interceptor};

#[derive(Debug, Clone)]
pub struct Renderer {
  current_timeset_id: Option<usize>
}

impl Default for Renderer {
  fn default() -> Self {
    Self {
      current_timeset_id: None
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
        Attrs::Comment(_level, msg) => {
          println!("# {}", msg);
          Return::Unmodified
        },
        Attrs::Cycle(repeat, _compressable) => {
          let dut = DUT.lock().unwrap();
          let t = &dut.timesets[self.current_timeset_id.unwrap()];
          println!("R{} {} <Pins> # <EoL Comment>;", repeat, t.name);
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
