use crate::core::tester::Interceptor;
use crate::testers::vector_based::pattern_renderer::Renderer;
use crate::testers::vector_based::VectorBased;
use crate::{Result, DUT};
use crate::core::model::pins::pin::{PinAction, Resolver};
use crate::core::model::timesets::timeset::default_resolver;

/// Base trait for IGXL-based testers (J750, UltraFlex).
/// 
/// This trait provides common pattern generation behavior via a blanket VectorBased impl.
pub trait IGXLBase: VectorBased + Interceptor {
    /// Returns true if this tester requires the end_module statement
    /// Default is false (UltraFlex), J750 overrides to return true
    fn requires_end_module(&self) -> bool {
        false
    }

    /// Returns additional header lines to be inserted before the vector declaration
    /// Default is None (J750), UltraFlex overrides to add opcode_mode, digital_inst, etc.
    fn additional_header_lines(&self) -> Option<Vec<String>> {
        None
    }
}

impl<T: IGXLBase> VectorBased for T {
    fn comment_str(&self) -> &str {
        "//"
    }

    fn file_ext(&self) -> &str {
        "atp"
    }

    fn print_pattern_end(&self, renderer: &mut Renderer) -> Option<Result<String>> {
        if self.requires_end_module() {
            // J750: includes end_module statement
            let tname = renderer.timeset_name().unwrap();
            Some(Ok(format!(
                "end_module > {} {} ;\n}}",
                tname,
                renderer.render_states().unwrap()
            )))
        } else {
            // UltraFlex: just closing brace
            Some(Ok("}".to_string()))
        }
    }

    /// Generates IGXL vector statements with format: [repeat N] > timeset states ;
    /// 
    /// Capture vectors use 'stv' (store vector) and cannot be compressed with repeat.
    /// Non-capture vectors use repeat statement when count > 1 for pattern compactness.
    fn print_vector(
        &self,
        renderer: &mut Renderer,
        repeat: u32,
        _compressable: bool,
    ) -> Option<Result<String>> {
        let states = renderer.states.as_ref().unwrap();
        let tname = renderer.timeset_name().unwrap();
        
         // IGXL requires individual 'stv' statements for each capture cycle
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

    /// Generates the pattern header with pin list declaration.
    /// Format: [additional_header_lines for UltraFlex]
    ///         vector ($tset, pin1, pin2, ...)
    ///         {
    ///         start_label pattern_st:
    fn print_pinlist(&self, renderer: &mut Renderer) -> Option<Result<String>> {
        let dut = DUT.lock().unwrap();
        let pins = format!(
            "vector ($tset, {})",
            renderer.states(&dut).names().join(", ")
        );
        
        let mut lines = vec![];
        
        // Add tester-specific header lines if any (before vector declaration)
        if let Some(additional) = self.additional_header_lines() {
            lines.extend(additional);
        }
        
        lines.push(pins);
        lines.push("{".to_string());
        lines.push("start_label pattern_st:".to_string());
        
        Some(Ok(lines.join("\n")))
    }

    /// Maps Origen pin actions to IGXL pattern characters.
    /// Both capture and highz map to 'X' in IGXL format.
    fn pin_action_resolver(&self) -> Option<Resolver> {
        let mut map = default_resolver();
        map.update_mapping(PinAction::capture(), "X".to_string());
        map.update_mapping(PinAction::highz(), "X".to_string());
        Some(map)
    }
}
