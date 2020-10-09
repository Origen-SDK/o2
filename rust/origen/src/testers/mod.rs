pub mod igxl;
pub mod simulator;
pub mod smt;
mod supported_testers;

use crate::core::tester::{Interceptor, TesterAPI};
use crate::generator::ast::{Attrs, Node};
use crate::generator::processor::{Processor, Return};
use crate::{dut, Result};
use std::path::PathBuf;
pub use supported_testers::SupportedTester;

pub fn instantiate_tester(g: &SupportedTester) -> Result<Box<dyn TesterAPI + std::marker::Send>> {
    match g {
        SupportedTester::DUMMYRENDERER => Ok(Box::new(DummyRenderer::default())),
        SupportedTester::DUMMYRENDERERWITHINTERCEPTORS => {
            Ok(Box::new(DummyRendererWithInterceptors::default()))
        }
        SupportedTester::V93KSMT7 => Ok(Box::new(smt::V93K_SMT7::default())),
        SupportedTester::SIMULATOR => Ok(Box::new(simulator::Renderer::default())),
        SupportedTester::ULTRAFLEX => Ok(Box::new(igxl::UltraFlex::default())),
        SupportedTester::CUSTOM(_) => {
            error!("Custom testers are not instantiated by this function")
        }
        _ => error!("The tester driver for {}, is not implemented yet", g),
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

    fn render_pattern(&mut self, ast: &Node) -> crate::Result<Vec<PathBuf>> {
        ast.process(self)?;
        Ok(vec![])
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
                println!(
                    "  ::DummyRenderer Node {}: Comment - Content: {}",
                    self.count, msg
                );
                self.count += 1;
                Ok(Return::Unmodified)
            }
            Attrs::Cycle(repeat, _compressable) => {
                let dut = dut();
                let t = &dut.timesets[self.current_timeset_id.unwrap()];
                println!(
                    "  ::DummyRenderer Node {}: Vector - Repeat: {}, Timeset: '{}'",
                    self.count, repeat, t.name
                );
                self.count += 1;
                Ok(Return::Unmodified)
            }
            Attrs::SetTimeset(timeset_id) => {
                self.current_timeset_id = Some(*timeset_id);
                Ok(Return::Unmodified)
            }
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

    fn render_pattern(&mut self, ast: &Node) -> crate::Result<Vec<PathBuf>> {
        //let mut slf = Self::default();
        ast.process(self)?;
        //node.clone()
        Ok(vec![])
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
    fn cycle(&mut self, _repeat: u32, _compressable: bool, _node: &Node) -> Result<()> {
        println!("Vector intercepted by DummyRendererWithInterceptors!");
        Ok(())
    }

    fn cc(&mut self, _level: u8, _msg: &str, _node: &Node) -> Result<()> {
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
                println!(
                    "  ::DummyRendererWithInterceptors Node {}: Comment - Content: {}",
                    self.count, msg
                );
                self.count += 1;
                Ok(Return::Unmodified)
            }
            Attrs::Cycle(repeat, _compressable) => {
                let dut = dut();
                let t = &dut.timesets[self.current_timeset_id.unwrap()];
                println!(
                    "  ::DummyRendererWithInterceptors Node {}: Vector - Repeat: {}, Timeset: '{}'",
                    self.count, repeat, t.name
                );
                self.count += 1;
                Ok(Return::Unmodified)
            }
            Attrs::SetTimeset(timeset_id) => {
                self.current_timeset_id = Some(*timeset_id);
                Ok(Return::Unmodified)
            }
            _ => Ok(Return::ProcessChildren),
        }
    }
}
