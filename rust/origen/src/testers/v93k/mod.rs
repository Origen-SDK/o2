use crate::DUT;
use crate::generator::processor::{Return, Processor};
use crate::generator::ast::{Node};
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
  fn on_comment(&mut self, _level: u8, msg: &str, _node: &Node) -> Return {
    println!("# {}", msg);
    Return::Unmodified
  }

  fn on_cycle(&mut self, repeat: u32, _compressable: bool, _node: &Node) -> Return {
    let dut = DUT.lock().unwrap();
    let t = &dut.timesets[self.current_timeset_id.unwrap()];
    println!("R{} {} <Pins> # <EoL Comment>;", repeat, t.name);
    Return::Unmodified
  }

  fn on_set_timeset(&mut self, timeset_id: usize, _node: &Node) -> Return {
    self.current_timeset_id = Some(timeset_id);
    Return::Unmodified
  }
}
