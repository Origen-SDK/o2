use crate::core::tester::{Interceptor, TesterAPI, TesterID};
use crate::testers::SupportedTester;
use origen_metal::prog_gen::SupportedTester as ProgGenSupportedTester;
//use crate::generator::ast::Node;
//use crate::Result;
//use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct UltraFlex {}

impl Default for UltraFlex {
    fn default() -> Self {
        Self {}
    }
}

impl Interceptor for UltraFlex {}

impl TesterID for UltraFlex {
    fn id(&self) -> SupportedTester {
        SupportedTester::ULTRAFLEX
    }
    
    fn id_prog_gen(&self) -> ProgGenSupportedTester {
        ProgGenSupportedTester::ULTRAFLEX
    }
}

impl TesterAPI for UltraFlex {}
