pub mod api;
pub mod pattern_renderer;

use crate::core::model::pins::pin::Resolver;
use crate::core::tester::{TesterAPI, TesterID};
use crate::generator::ast::Node;
use crate::utility::differ::Differ;
use crate::Result;
use std::path::{Path, PathBuf};
use crate::generator::processor::Return;
use pattern_renderer::Renderer;

pub trait VectorBased:
    std::fmt::Debug + std::default::Default + crate::core::tester::Interceptor + TesterID + Clone + Send
{
    fn comment_str(&self) -> &str;
    fn file_ext(&self) -> &str;
    fn print_vector(
        &self,
        renderer: &mut pattern_renderer::Renderer,
        repeat: u32,
        compressable: bool,
    ) -> Option<Result<String>>;
    fn print_pinlist(&self, renderer: &mut pattern_renderer::Renderer) -> Option<Result<String>>;

    fn print_pattern_end(
        &self,
        _renderer: &mut pattern_renderer::Renderer,
    ) -> Option<Result<String>> {
        None
    }

    fn pin_action_resolver(&self) -> Option<Resolver> {
        None
    }

    fn override_node(&self, _renderer: &mut Renderer, _node: &Node) -> Option<Result<Return>> {
        None
    }
}

impl<T: 'static> pattern_renderer::RendererAPI for T
where
    T: VectorBased,
{
    default fn file_ext(&self) -> &str {
        VectorBased::file_ext(self)
    }

    default fn comment_str(&self) -> &str {
        VectorBased::comment_str(self)
    }

    default fn print_vector(
        &self,
        renderer: &mut pattern_renderer::Renderer,
        repeat: u32,
        compressable: bool,
    ) -> Option<Result<String>> {
        VectorBased::print_vector(self, renderer, repeat, compressable)
    }

    default fn print_pinlist(
        &self,
        renderer: &mut pattern_renderer::Renderer,
    ) -> Option<Result<String>> {
        VectorBased::print_pinlist(self, renderer)
    }

    default fn print_pattern_end(
        &self,
        renderer: &mut pattern_renderer::Renderer,
    ) -> Option<Result<String>> {
        VectorBased::print_pattern_end(self, renderer)
    }

    fn override_node(&self, renderer: &mut Renderer, node: &Node) -> Option<Result<Return>> {
        VectorBased::override_node(self, renderer, node)
    }
}

impl<T: 'static> TesterAPI for T
where
    T: VectorBased,
{
    default fn render_pattern(&mut self, node: &Node) -> Result<Vec<PathBuf>> {
        pattern_renderer::Renderer::run(self, node)
    }

    default fn pattern_differ(&self, pat_a: &Path, pat_b: &Path) -> Option<Differ> {
        let mut d = Differ::new(pat_a, pat_b);
        let _ = d.ignore_comments(self.comment_str());
        Some(d)
    }

    default fn pin_action_resolver(&self) -> Option<Resolver> {
        VectorBased::pin_action_resolver(self)
    }
}
