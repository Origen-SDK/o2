use crate::core::tester::{Interceptor, TesterID};
use crate::testers::vector_based::pattern_renderer::Renderer;
use crate::testers::vector_based::VectorBased;
use crate::testers::SupportedTester;
use origen_metal::prog_gen::SupportedTester as ProgGenSupportedTester;
use crate::{Result, DUT};

use crate::core::model::pins::pin::{PinAction, Resolver};
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

impl TesterID for J750 {
    fn id(&self) -> SupportedTester {
        SupportedTester::J750
    }
    
    fn id_prog_gen(&self) -> ProgGenSupportedTester {
        ProgGenSupportedTester::J750
    }
}

impl VectorBased for J750 {
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

    fn print_vector(
        &self,
        renderer: &mut Renderer,
        repeat: u32,
        _compressable: bool,
    ) -> Option<Result<String>> {
        let states = renderer.states.as_ref().unwrap();
        let tname = renderer.timeset_name().unwrap();
        if states.contains_action(PinAction::capture()) {
            return Some(Ok(vec![
                format!(
                    " stv > {} {} ;",
                    tname,
                    renderer.render_states().unwrap()
                );
                repeat as usize
            ]
            .join("\n")));
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
        let pins = format!(
            "vector ($tset, {})",
            renderer.states(&dut).names().join(", ")
        );
        Some(Ok([&pins, "{", "start_label pattern_st:"].join("\n")))
    }

    fn pin_action_resolver(&self) -> Option<Resolver> {
        let mut map = default_resolver();
        map.update_mapping(PinAction::capture(), "X".to_string());
        map.update_mapping(PinAction::highz(), "X".to_string());
        Some(map)
    }
}
