pub mod v93k;
pub mod simulator;
use crate::error::Error;
use crate::dut;
use crate::generator::processor::{Return, Processor};
use crate::generator::ast::{Node};
use crate::core::tester::{TesterAPI, Interceptor};

pub fn available_testers() -> Vec<String> {
  vec![
    "::DummyGenerator".to_string(),
    "::DummyGeneratorWithInterceptors".to_string(),
    "::V93K::ST7".to_string(),
    "::Simulator".to_string(),
  ]
}

pub fn instantiate_tester(g: &str) -> Option<Box<dyn TesterAPI + std::marker::Send>> {
  match &g {
    &"::DummyGenerator" => Some(Box::new(DummyGenerator::default())),
    &"::DummyGeneratorWithInterceptors" => Some(Box::new(DummyGeneratorWithInterceptors::default())),
    &"::V93K::ST7" => Some(Box::new(v93k::Generator::default())),
    &"::Simulator" => Some(Box::new(simulator::Generator::default())),
    _ => None
  }
}

#[derive(Debug, Clone)]
pub struct DummyGenerator {
  count: usize,
  current_timeset_id: Option<usize>,
}

impl Default for DummyGenerator {
  fn default() -> Self {
    Self {
      count: 0,
      current_timeset_id: Option::None,
    }
  }
}

impl DummyGenerator {}
impl Interceptor for DummyGenerator {}
impl TesterAPI for DummyGenerator {
  fn name(&self) -> String {
    "DummyGenerator".to_string()
  }

  fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
    Box::new(std::clone::Clone::clone(self))
  }

  fn run(&mut self, node: &Node) -> Node {
    node.process(self).unwrap()
  }
}

impl Processor for DummyGenerator {
  fn on_test(&mut self, _name: &str, _node: &Node) -> Return {
    // Not counting the top node as a node. Only comments and cycles.
    println!("Printing StubAST to console...");
    Return::ProcessChildren
  }
  
  fn on_comment(&mut self, _level: u8, msg: &str, _node: &Node) -> Return {
    println!("  ::DummyGenerator Node {}: Comment - Content: {}", self.count, msg);
    self.count += 1;
    Return::Unmodified
  }

  fn on_cycle(&mut self, repeat: u32, _compressable: bool, _node: &Node) -> Return {
    let dut = dut();
    let t = &dut.timesets[self.current_timeset_id.unwrap()];
    println!("  ::DummyGenerator Node {}: Vector - Repeat: {}, Timeset: '{}'", self.count, repeat, t.name);
    self.count += 1;
    Return::Unmodified
  }

  fn on_set_timeset(&mut self, timeset_id: usize, _node: &Node) -> Return {
    self.current_timeset_id = Some(timeset_id);
    Return::Unmodified
  }
}

#[derive(Debug, Clone)]
pub struct DummyGeneratorWithInterceptors {
  count: usize,
  current_timeset_id: Option<usize>,
}

impl DummyGeneratorWithInterceptors {}

impl TesterAPI for DummyGeneratorWithInterceptors {
  fn name(&self) -> String {
    "DummyGeneratorWithInterceptors".to_string()
  }

  fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
    Box::new(std::clone::Clone::clone(self))
  }

  fn run(&mut self, node: &Node) -> Node {
    //let mut slf = Self::default();
    node.process(self).unwrap()
    //node.clone()
  }
}

impl Default for DummyGeneratorWithInterceptors {
  fn default() -> Self {
    Self {
      count: 0,
      current_timeset_id: Option::None,
    }
  }
}

impl Interceptor for DummyGeneratorWithInterceptors {
  fn cycle(&mut self, _repeat: u32, _compressable: bool, _node: &Node) -> Result<(), Error> {
    println!("Vector intercepted by DummyGeneratorWithInterceptors!");
    Ok(())
  }

  fn cc(&mut self, _level: u8, _msg: &str, _node: &Node) -> Result<(), Error> {
    println!("Comment intercepted by DummyGeneratorWithInterceptors!");
    Ok(())
  }
}

impl Processor for DummyGeneratorWithInterceptors {

  fn on_test(&mut self, _name: &str, _node: &Node) -> Return {
    // Not counting the top node as a node. Only comments and cycles.
    println!("Printing StubAST to console...");
    Return::ProcessChildren
  }

  fn on_comment(&mut self, _level: u8, msg: &str, _node: &Node) -> Return {
    println!("  ::DummyGeneratorWithInterceptors Node {}: Comment - Content: {}", self.count, msg);
    self.count += 1;
    Return::Unmodified
  }

  fn on_cycle(&mut self, repeat: u32, _compressable: bool, _node: &Node) -> Return {
    let dut = dut();
    let t = &dut.timesets[self.current_timeset_id.unwrap()];
    println!("  ::DummyGeneratorWithInterceptors Node {}: Vector - Repeat: {}, Timeset: '{}'", self.count, repeat, t.name);
    self.count += 1;
    Return::Unmodified
  }

  fn on_set_timeset(&mut self, timeset_id: usize, _node: &Node) -> Return {
    self.current_timeset_id = Some(timeset_id);
    Return::Unmodified
  }
}
