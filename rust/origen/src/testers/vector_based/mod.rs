pub mod pattern_renderer;

use crate::utility::differ::Differ;
use crate::Result;
use std::path::{Path, PathBuf};
use crate::core::tester::TesterAPI;
use crate::core::model::pins::pin::Resolver;
use crate::generator::ast::Node;

pub trait VectorBased: std::fmt::Debug + std::default::Default + crate::core::tester::Interceptor {
    fn name(&self) -> String;
    fn id(&self) -> String;
    fn clone(&self) ->  Box<dyn TesterAPI + std::marker::Send>;
    fn comment_str(&self) -> &str;
    fn file_ext(&self) -> &str;
    fn print_vector(&self, renderer: &mut pattern_renderer::Renderer, repeat: u32, compressable: bool) -> Option<Result<String>>;
    fn print_pinlist(&self, renderer: &mut pattern_renderer::Renderer) -> Option<Result<String>>;

    fn print_pattern_end(&self, _renderer: &mut pattern_renderer::Renderer) -> Option<Result<String>> {
        None
    }

    fn pin_action_resolver(&self) -> Option<Resolver> {
        None
    }
}

impl <T> pattern_renderer::RendererAPI for T where T: VectorBased {
    fn name(&self) -> String {
        VectorBased::name(self)
    }

    fn id(&self) -> String {
        VectorBased::id(self)
    }

    fn file_ext(&self) -> &str {
        VectorBased::file_ext(self)
    }

    fn comment_str(&self) -> &str {
        VectorBased::comment_str(self)
    }

    fn print_vector(&self, renderer: &mut pattern_renderer::Renderer, repeat: u32, compressable: bool) -> Option<Result<String>> {
        VectorBased::print_vector(self, renderer, repeat, compressable)
    }

    fn print_pinlist(&self, renderer: &mut pattern_renderer::Renderer) -> Option<Result<String>> {
        VectorBased::print_pinlist(self, renderer)
    }

    fn print_pattern_end(&self, renderer: &mut pattern_renderer::Renderer) -> Option<Result<String>> {
        VectorBased::print_pattern_end(self, renderer)
    }
}

impl <T> crate::core::tester::TesterAPI for T where T: VectorBased {
    fn name(&self) -> String {
        VectorBased::name(self)
    }

    fn id(&self) -> String {
        VectorBased::id(self)
    }

    fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
        VectorBased::clone(self)
    }

    fn render_pattern(&mut self, node: &Node) -> Result<Vec<PathBuf>> {
        pattern_renderer::Renderer::run(self, node)
    }

    fn pattern_differ(&self, pat_a: &Path, pat_b: &Path) -> Option<Differ> {
        let mut d = Differ::new(pat_a, pat_b);
        let _ = d.ignore_comments(self.comment_str());
        Some(d)
    }

    fn pin_action_resolver(&self) -> Option<Resolver> {
        VectorBased::pin_action_resolver(self)
    }
}
