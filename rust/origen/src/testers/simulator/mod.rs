use crate::core::tester::{Interceptor, TesterAPI, TesterID};
use crate::error::Error;
use crate::generator::ast::Node;
use crate::generator::processor::Processor;
use crate::testers::SupportedTester;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Renderer {}

impl Default for Renderer {
    fn default() -> Self {
        Self {}
    }
}

impl Processor for Renderer {}

impl Interceptor for Renderer {
    fn clear_timeset(&mut self, _node: &Node) -> Result<(), Error> {
        println!("<Issue command to clear the timeset in the simulator...>");
        Ok(())
    }

    fn set_timeset(&mut self, _timeset_id: usize, _node: &Node) -> Result<(), Error> {
        println!("<Issue command to set the timeset in the simulator...>");
        Ok(())
    }

    fn cycle(&mut self, _repeat: u32, _compressable: bool, _node: &Node) -> Result<(), Error> {
        println!("<Issue command to cycle the simulator...>");
        Ok(())
    }

    fn cc(&mut self, _level: u8, _msg: &str, _node: &Node) -> Result<(), Error> {
        println!("<Issue command to place a comment in the simulator...>");
        Ok(())
    }
}

impl TesterID for Renderer {
    fn id(&self) -> SupportedTester {
        SupportedTester::SIMULATOR
    }
}

impl TesterAPI for Renderer {
    fn render_pattern(&mut self, ast: &Node) -> crate::Result<Vec<PathBuf>> {
        ast.process(self)?;
        Ok(vec![])
    }
}
