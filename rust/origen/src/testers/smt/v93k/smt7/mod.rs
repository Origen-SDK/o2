use crate::testers::vector_based::pattern_renderer::Renderer;
use crate::testers::vector_based::VectorBased;
use crate::core::tester::{Interceptor, TesterAPI};
use crate::{Result, DUT};

#[derive(Debug, Clone)]
pub struct SMT7 {}

impl Default for SMT7 {
    fn default() -> Self {
        Self {}
    }
}

impl VectorBased for SMT7 {
    fn name(&self) -> String {
        "V93K_SMT7".to_string()
    }

    fn id(&self) -> String {
        "::V93K::SMT7".to_string()
    }

    fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
        Box::new(std::clone::Clone::clone(self))
    }

    fn comment_str(&self) -> &str {
        "#"
    }

    fn file_ext(&self) -> &str {
        "avc"
    }

    fn print_vector(&self, renderer: &mut Renderer, repeat: u32, _compressable: bool) -> Option<Result<String>> {
        Some(Ok(format!(
            "R{} {} {} # <EoL Comment>;",
            repeat,
            {
                match renderer.timeset_name() {
                    Ok(s) => s,
                    Err(e) => return Some(Err(e))
                }
            },

            // The pin states should have been previously updated from the PinAction node, or just has default values
            {
                match renderer.render_states() {
                    Ok(s) => s,
                    Err(e) => return Some(Err(e))
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
}

impl Interceptor for SMT7 {}

