use crate::testers::vector_based::VectorBased;
use crate::core::tester::{Interceptor, TesterAPI};
use crate::testers::vector_based::pattern_renderer::Renderer;
use crate::{Result, DUT};

use crate::core::model::pins::pin::{PinActions, Resolver};
use crate::core::model::timesets::timeset::default_resolver;

#[derive(Debug, Clone)]
pub struct J750 {}

impl J750 {
    pub fn default() -> Self {
        Self {}
    }
}
impl Interceptor for J750 {}
impl std::default::Default for J750 {
    fn default() -> Self {
        Self::default()
    }
}

impl VectorBased for J750 {
    fn name(&self) -> String {
        "Teradyne_J750".to_string()
    }

    fn clone(&self) -> Box<dyn TesterAPI + std::marker::Send> {
        Box::new(std::clone::Clone::clone(self))
    }

    fn comment_str(&self) -> &str {
        "//"
    }

    fn file_ext(&self) -> &str {
        "atp"
    }

    fn print_pattern_end(&self, renderer: &mut Renderer) -> Option<Result<String>> {
        let tname = renderer.timeset_name().unwrap();
        Some(Ok(format!(
            "end_module > {} {} ;\n}}",
            tname,
            renderer.render_states().unwrap()
        )))
    }

    fn print_vector(&self, renderer: &mut Renderer, repeat: u32, _compressable: bool) -> Option<Result<String>> {
        let states = renderer.states.as_ref().unwrap();
        let tname = renderer.timeset_name().unwrap();
        if states.contains_action(PinActions::Capture) {
            return Some(Ok(vec![
                format!(" stv > {} {} ;", tname, renderer.render_states().unwrap());
                repeat as usize
            ].join("\n")))
        }

        if repeat == 1 {
            Some(Ok(format!(
                " > {} {} ;",
                tname,
                renderer.render_states().unwrap()
            )))
        } else {
            Some(Ok(format!(
                "repeat {} > {} {} ;",
                repeat,
                tname,
                renderer.render_states().unwrap()
            )))
        }
    }

    fn print_pinlist(&self, renderer: &mut Renderer) -> Option<Result<String>> {
        let dut = DUT.lock().unwrap();
        let pins = format!("vector ($tset, {})", renderer.states(&dut).names().join(", "));
        Some(Ok([&pins, "{", "start_label pattern_st:"].join("\n")))
    }

    fn pin_action_resolver(&self) -> Option<Resolver> {
        let mut map = default_resolver();
        map.update_mapping(PinActions::Capture, "X".to_string());
        map.update_mapping(PinActions::HighZ, "X".to_string());
        Some(map)
    }
}