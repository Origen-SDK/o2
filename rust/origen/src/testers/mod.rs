pub mod api;
pub mod igxl;
pub mod simulator;
pub mod smt;
mod supported_testers;
pub mod vector_based;

use crate::core::tester::{Interceptor, TesterAPI, TesterID};
use crate::generator::PAT;
use crate::{dut, Result};
use origen_metal::ast::{Node, Processor, Return};
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
        SupportedTester::J750 => Ok(Box::new(igxl::j750::J750::default())),
        SupportedTester::CUSTOM(_) => {
            bail!("Custom testers are not instantiated by this function")
        }
        _ => bail!("The tester driver for {}, is not implemented yet", g),
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

impl TesterID for DummyRenderer {
    fn id(&self) -> SupportedTester {
        SupportedTester::DUMMYRENDERER
    }
}

impl TesterAPI for DummyRenderer {
    fn render_pattern(&mut self, ast: &Node<PAT>) -> crate::Result<Vec<PathBuf>> {
        ast.process(self)?;
        Ok(vec![])
    }
}

impl Processor<PAT> for DummyRenderer {
    fn on_node(&mut self, node: &Node<PAT>) -> crate::Result<Return<PAT>> {
        match &node.attrs {
            PAT::Test(_name) => {
                // Not counting the top node as a node. Only comments and cycles.
                println!("Printing StubAST to console...");
                Ok(Return::ProcessChildren)
            }
            PAT::Comment(_level, msg) => {
                println!(
                    "  ::DummyRenderer Node {}: Comment - Content: {}",
                    self.count, msg
                );
                self.count += 1;
                Ok(Return::Unmodified)
            }
            PAT::Cycle(repeat, _compressable) => {
                let dut = dut();
                let t = &dut.timesets[self.current_timeset_id.unwrap()];
                println!(
                    "  ::DummyRenderer Node {}: Vector - Repeat: {}, Timeset: '{}'",
                    self.count, repeat, t.name
                );
                self.count += 1;
                Ok(Return::Unmodified)
            }
            PAT::SetTimeset(timeset_id) => {
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

impl TesterID for DummyRendererWithInterceptors {
    fn id(&self) -> SupportedTester {
        SupportedTester::DUMMYRENDERERWITHINTERCEPTORS
    }
}

impl TesterAPI for DummyRendererWithInterceptors {
    fn render_pattern(&mut self, ast: &Node<PAT>) -> crate::Result<Vec<PathBuf>> {
        ast.process(self)?;
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
    fn cycle(&mut self, _repeat: u32, _compressable: bool, _node: &Node<PAT>) -> Result<()> {
        println!("Vector intercepted by DummyRendererWithInterceptors!");
        Ok(())
    }

    fn cc(&mut self, _level: u8, _msg: &str, _node: &Node<PAT>) -> Result<()> {
        println!("Comment intercepted by DummyRendererWithInterceptors!");
        Ok(())
    }
}

impl Processor<PAT> for DummyRendererWithInterceptors {
    fn on_node(&mut self, node: &Node<PAT>) -> crate::Result<Return<PAT>> {
        match &node.attrs {
            PAT::Test(_name) => {
                // Not counting the top node as a node. Only comments and cycles.
                println!("Printing StubAST to console...");
                Ok(Return::ProcessChildren)
            }
            PAT::Comment(_level, msg) => {
                println!(
                    "  ::DummyRendererWithInterceptors Node {}: Comment - Content: {}",
                    self.count, msg
                );
                self.count += 1;
                Ok(Return::Unmodified)
            }
            PAT::Cycle(repeat, _compressable) => {
                let dut = dut();
                let t = &dut.timesets[self.current_timeset_id.unwrap()];
                println!(
                    "  ::DummyRendererWithInterceptors Node {}: Vector - Repeat: {}, Timeset: '{}'",
                    self.count, repeat, t.name
                );
                self.count += 1;
                Ok(Return::Unmodified)
            }
            PAT::SetTimeset(timeset_id) => {
                self.current_timeset_id = Some(*timeset_id);
                Ok(Return::Unmodified)
            }
            _ => Ok(Return::ProcessChildren),
        }
    }
}
