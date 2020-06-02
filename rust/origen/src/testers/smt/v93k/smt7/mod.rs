pub mod pattern_renderer;

use crate::core::tester::{Interceptor, TesterAPI};
use crate::generator::ast::Node;
use crate::utility::differ::Differ;
use crate::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SMT7 {}

impl Default for SMT7 {
    fn default() -> Self {
        Self {}
    }
}

impl Interceptor for SMT7 {}
impl TesterAPI for SMT7 {
    fn name(&self) -> String {
        "V93K_ST7".to_string()
    }

    fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
        Box::new(std::clone::Clone::clone(self))
    }

    fn render_pattern(&mut self, node: &Node) -> Result<Vec<PathBuf>> {
        pattern_renderer::Renderer::run(self, node)
    }

    fn pattern_differ(&self, pat_a: &Path, pat_b: &Path) -> Option<Differ> {
        let mut d = Differ::new(pat_a, pat_b);
        d.ignore_comments("#");
        Some(d)
    }
}
