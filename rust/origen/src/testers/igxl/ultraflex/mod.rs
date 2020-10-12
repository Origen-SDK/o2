use crate::core::tester::{Interceptor, TesterAPI};
//use crate::generator::ast::Node;
//use crate::Result;
//use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct UltraFlex {}

impl Default for UltraFlex {
    fn default() -> Self {
        Self {}
    }
}

impl Interceptor for UltraFlex {}

impl TesterAPI for UltraFlex {
    fn name(&self) -> String {
        "ULTRAFLEX".to_string()
    }

    fn id(&self) -> String {
        "UltraFlex".to_string()
    }

    fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
        Box::new(std::clone::Clone::clone(self))
    }

    //fn render_pattern(&mut self, node: &Node) -> Result<Option<PathBuf>> {
    //    pattern_renderer::Renderer::run(self, node)
    //}
}
