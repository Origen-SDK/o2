pub mod api;
pub mod pattern_renderer;

use crate::core::model::pins::pin::Resolver;
use crate::core::tester::{TesterAPI, TesterID};
use crate::generator::PAT;
use crate::prog_gen::Model;
use crate::{Overlay, Result};
use origen_metal::ast::{Node, Return};
use origen_metal::utils::differ::{ASCIIDiffer, Differ};
use pattern_renderer::Renderer;
use std::path::{Path, PathBuf};

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

    fn override_node(
        &self,
        _renderer: &mut Renderer,
        _node: &Node<PAT>,
    ) -> Option<Result<Return<PAT>>> {
        None
    }

    fn start_overlay(
        &self,
        _renderer: &mut pattern_renderer::Renderer,
        overlay: &Overlay,
    ) -> Option<Result<String>> {
        Some(Ok(format!(
            "Start Overlay: {}",
            overlay.label.as_ref().unwrap_or(&"".to_string())
        )))
    }

    fn end_overlay(
        &self,
        _renderer: &mut pattern_renderer::Renderer,
        label: &Option<String>,
        _pin_id: &Option<usize>,
    ) -> Option<Result<String>> {
        Some(Ok(format!(
            "End Overlay: {}",
            label.as_ref().unwrap_or(&"".to_string())
        )))
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

    default fn start_overlay(
        &self,
        renderer: &mut pattern_renderer::Renderer,
        overlay: &Overlay,
    ) -> Option<Result<String>> {
        VectorBased::start_overlay(self, renderer, overlay)
    }

    default fn end_overlay(
        &self,
        renderer: &mut pattern_renderer::Renderer,
        label: &Option<String>,
        pin_id: &Option<usize>,
    ) -> Option<Result<String>> {
        VectorBased::end_overlay(self, renderer, label, pin_id)
    }

    fn override_node(
        &self,
        renderer: &mut Renderer,
        node: &Node<PAT>,
    ) -> Option<Result<Return<PAT>>> {
        VectorBased::override_node(self, renderer, node)
    }
}

impl<T: 'static> TesterAPI for T
where
    T: VectorBased,
{
    default fn render_pattern(&mut self, node: &Node<PAT>) -> Result<Vec<PathBuf>> {
        pattern_renderer::Renderer::run(self, node)
    }

    default fn pattern_differ(&self, pat_a: &Path, pat_b: &Path) -> Option<Box<dyn Differ>> {
        let mut d = ASCIIDiffer::new(pat_a, pat_b);
        let _ = d.ignore_comments(self.comment_str());
        Some(Box::new(d))
    }

    default fn program_differ(&self, pat_a: &Path, pat_b: &Path) -> Option<Box<dyn Differ>> {
        let mut d = ASCIIDiffer::new(pat_a, pat_b);
        let _ = d.ignore_comments(self.comment_str());
        Some(Box::new(d))
    }

    default fn pin_action_resolver(&self) -> Option<Resolver> {
        VectorBased::pin_action_resolver(self)
    }

    default fn render_program(&mut self) -> crate::Result<(Vec<PathBuf>, Model)> {
        log_debug!("Tester '{}' does not implement render_program", &self.id());
        Ok((vec![], Model::new(self.id())))
    }

    default fn output_dir(&self) -> Result<PathBuf> {
        let dir = crate::STATUS.output_dir().join(&self.name().to_lowercase());
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }
}
