use crate::core::tester::{Interceptor, TesterAPI, TesterID};
use crate::testers::vector_based::pattern_renderer::Renderer;
use crate::testers::vector_based::VectorBased;
use crate::testers::SupportedTester;
use crate::{Result, DUT};
use crate::generator::ast::{Attrs, Node};
use crate::generator::processor::{Processor, Return};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SMT7 {}

impl Default for SMT7 {
    fn default() -> Self {
        Self {}
    }
}

impl TesterID for SMT7 {
    fn id(&self) -> SupportedTester {
        SupportedTester::V93KSMT7
    }
}

impl VectorBased for SMT7 {
    fn comment_str(&self) -> &str {
        "#"
    }

    fn file_ext(&self) -> &str {
        "avc"
    }

    fn print_vector(
        &self,
        renderer: &mut Renderer,
        repeat: u32,
        _compressable: bool,
    ) -> Option<Result<String>> {
        Some(Ok(format!(
            "R{} {} {} # <EoL Comment>;",
            repeat,
            {
                match renderer.timeset_name() {
                    Ok(s) => s,
                    Err(e) => return Some(Err(e)),
                }
            },
            // The pin states should have been previously updated from the PinAction node, or just has default values
            {
                match renderer.render_states() {
                    Ok(s) => s,
                    Err(e) => return Some(Err(e)),
                }
            }
        )))
    }

    fn print_pinlist(&self, renderer: &mut Renderer) -> Option<Result<String>> {
        let dut = DUT.lock().unwrap();
        let pins = renderer.states(&dut).names().join(" ");
        Some(Ok(format!("FORMAT {};", pins)))
    }

    fn print_pattern_end(&self, _renderer: &mut Renderer) -> Option<Result<String>> {
        Some(Ok("SQPG STOP;".to_string()))
    }

    fn override_node(&self, renderer: &mut Renderer, node: &Node) -> Option<Result<Return>> {
        match &node.attrs {
            Attrs::Capture(capture, _metadata) => {
                match capture.enabled_capture_pins() {
                    Ok(ids) => {
                        for pin in ids.iter() {
                            if let Some(sym) = capture.symbol.as_ref() {
                                renderer.capturing.insert(
                                    Some(*pin),
                                    capture.symbol.clone()
                                );
                            } else {
                                renderer.capturing.insert(
                                    Some(*pin),
                                    Some(crate::standards::actions::CAPTURE.to_string())
                                );
                            }
                        }
                        Some(Ok(Return::Unmodified))
                    },
                    Err(e) => None
                }
            }
            _ => None
        }
    }
}

impl Interceptor for SMT7 {}

impl TesterAPI for SMT7 {
    fn render_program(&mut self) -> crate::Result<Vec<PathBuf>> {
        crate::prog_gen::advantest::smt7::render_test_program(&self)
    }
}
