use crate::core::tester::{Interceptor, TesterID};
use crate::testers::SupportedTester;
use origen_metal::prog_gen::SupportedTester as ProgGenSupportedTester;
use super::base::IGXLBase;


/// Teradyne UltraFlex tester implementation.
/// 
/// Inherits IGXL pattern generation from IGXLBase trait.
/// Adds UltraFlex-specific header directives.
#[derive(Debug, Clone)]
pub struct UltraFlex {}

impl UltraFlex {
    pub fn default() -> Self {
        Self {}
    }
}

impl std::default::Default for UltraFlex {
    fn default() -> Self {
        Self::default()
    }
}

/// Uses default interceptor behavior (no command interception).
impl Interceptor for UltraFlex {}

/// Provides IGXL pattern generation via blanket impl in base.rs.
impl IGXLBase for UltraFlex {
    /// UltraFlex-specific header directives (placed before vector declaration)
    fn additional_header_lines(&self) -> Option<Vec<String>> {
        Some(vec![
            "opcode_mode = single;".to_string(),
            "digital_inst = hsdp;".to_string(),
            "compressed = yes;".to_string(),
        ])
    }
}

impl TesterID for UltraFlex {
    fn id(&self) -> SupportedTester {
        SupportedTester::ULTRAFLEX
    }
    
    fn id_prog_gen(&self) -> ProgGenSupportedTester {
        ProgGenSupportedTester::ULTRAFLEX
    }
}
