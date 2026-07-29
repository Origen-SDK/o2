use crate::core::tester::{Interceptor, TesterID};
use crate::testers::SupportedTester;
use origen_metal::prog_gen::SupportedTester as ProgGenSupportedTester;
use super::base::IGXLBase;

#[derive(Debug, Clone)]
pub struct J750 {}

impl J750 {
    pub fn default() -> Self {
        Self {}
    }
}

impl std::default::Default for J750 {
    fn default() -> Self {
        Self::default()
    }
}

/// Uses default interceptor behavior (no command interception).
impl Interceptor for J750 {}

/// Provides IGXL pattern generation via blanket impl in base.rs.
impl IGXLBase for J750 {
    /// J750 requires the end_module statement
    fn requires_end_module(&self) -> bool {
        true
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
