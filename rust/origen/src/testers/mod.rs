pub mod v93k;
pub mod simulator;
use crate::error::Error;
use crate::dut;
use crate::generator::processor::{Return, Processor};
use crate::generator::ast::{Node, Attrs};
use crate::core::tester::{TesterAPI, Interceptor};

pub fn available_testers() -> Vec<String> {
  vec![
    "::DummyRenderer".to_string(),
    "::DummyRendererWithInterceptors".to_string(),
    "::V93K::ST7".to_string(),
    "::Simulator".to_string(),
  ]
}

pub fn instantiate_tester(g: &str) -> Option<Box<dyn TesterAPI + std::marker::Send>> {
  match &g {
    &"::DummyRenderer" => Some(Box::new(DummyRenderer::default())),
    &"::DummyRendererWithInterceptors" => Some(Box::new(DummyRendererWithInterceptors::default())),
    &"::V93K::ST7" => Some(Box::new(v93k::Renderer::default())),
    &"::Simulator" => Some(Box::new(simulator::Renderer::default())),
    _ => None
  }
}

#[derive(Debug, Clone)]
pub struct DummyRenderer {
  count: usize,
  current_timeset_id: Option<usize>,
}

impl Default for DummyRenderer {
  fn default() -> Self {
    Self {
      count: 0,
      current_timeset_id: Option::None,
    }
  }
}

impl DummyRenderer {}
impl Interceptor for DummyRenderer {}
impl TesterAPI for DummyRenderer {
  fn name(&self) -> String {
    "DummyRenderer".to_string()
  }

  fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
    Box::new(std::clone::Clone::clone(self))
  }

  fn run(&mut self, node: &Node) -> crate::Result<Node> {
    Ok(node.process(self)?.unwrap())
  }
}

impl Processor for DummyRenderer {
  fn on_node(&mut self, node: &Node) -> crate::Result<Return> {
    match &node.attrs {
        Attrs::Test(_name) => {
          // Not counting the top node as a node. Only comments and cycles.
          println!("Printing StubAST to console...");
          Ok(Return::ProcessChildren)
        }
        Attrs::Comment(_level, msg) => {
          println!("  ::DummyRenderer Node {}: Comment - Content: {}", self.count, msg);
          self.count += 1;
          Ok(Return::Unmodified)
        },
        Attrs::Cycle(repeat, _compressable) => {
          let dut = dut();
          let t = &dut.timesets[self.current_timeset_id.unwrap()];
          println!("  ::DummyRenderer Node {}: Vector - Repeat: {}, Timeset: '{}'", self.count, repeat, t.name);
          self.count += 1;
          Ok(Return::Unmodified)
        },
        Attrs::SetTimeset(timeset_id) => {
          self.current_timeset_id = Some(*timeset_id);
          Ok(Return::Unmodified)
        },
        _ => Ok(Return::ProcessChildren),
    }
  }
}

#[derive(Debug, Clone)]
pub struct DummyRendererWithInterceptors {
  count: usize,
  current_timeset_id: Option<usize>,
}

impl DummyRendererWithInterceptors {}

impl TesterAPI for DummyRendererWithInterceptors {
  fn name(&self) -> String {
    "DummyRendererWithInterceptors".to_string()
  }

  fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
    Box::new(std::clone::Clone::clone(self))
  }

  fn run(&mut self, node: &Node) -> crate::Result<Node> {
    //let mut slf = Self::default();
    Ok(node.process(self)?.unwrap())
    //node.clone()
  }
}

impl Default for DummyRendererWithInterceptors {
  fn default() -> Self {
    Self {
      count: 0,
      current_timeset_id: Option::None,
    }
  }
}

impl Interceptor for DummyRendererWithInterceptors {
  fn cycle(&mut self, _repeat: u32, _compressable: bool, _node: &Node) -> Result<(), Error> {
    println!("Vector intercepted by DummyRendererWithInterceptors!");
    Ok(())
  }

  fn cc(&mut self, _level: u8, _msg: &str, _node: &Node) -> Result<(), Error> {
    println!("Comment intercepted by DummyRendererWithInterceptors!");
    Ok(())
  }
}

impl Processor for DummyRendererWithInterceptors {
  fn on_node(&mut self, node: &Node) -> crate::Result<Return> {
    match &node.attrs {
        Attrs::Test(_name) => {
          // Not counting the top node as a node. Only comments and cycles.
          println!("Printing StubAST to console...");
          Ok(Return::ProcessChildren)
        }
        Attrs::Comment(_level, msg) => {
          println!("  ::DummyRendererWithInterceptors Node {}: Comment - Content: {}", self.count, msg);
          self.count += 1;
          Ok(Return::Unmodified)
        },
        Attrs::Cycle(repeat, _compressable) => {
          let dut = dut();
          let t = &dut.timesets[self.current_timeset_id.unwrap()];
          println!("  ::DummyRendererWithInterceptors Node {}: Vector - Repeat: {}, Timeset: '{}'", self.count, repeat, t.name);
          self.count += 1;
          Ok(Return::Unmodified)
        },
        Attrs::SetTimeset(timeset_id) => {
          self.current_timeset_id = Some(*timeset_id);
          Ok(Return::Unmodified)
        },
        _ => Ok(Return::ProcessChildren),
    }
  }
}
