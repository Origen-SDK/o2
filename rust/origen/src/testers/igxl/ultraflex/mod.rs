use crate::core::tester::{Interceptor, TesterAPI, TesterID};
use crate::testers::SupportedTester;
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
}

impl TesterAPI for UltraFlex {}
