use crate::core::tester::{Interceptor, TesterAPI};
use crate::error::Error;
use crate::generator::ast::Node;
use crate::generator::processor::Processor;

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
impl TesterAPI for Renderer {
    fn name(&self) -> String {
        "Simulator".to_string()
    }

    fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
        Box::new(std::clone::Clone::clone(self))
    }

    fn render_pattern(&mut self, ast: &Node) -> crate::Result<()> {
        ast.process(self)?;
        Ok(())
    }
}
