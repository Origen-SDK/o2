use super::IGXL;
use crate::prog_gen::{PatternGroup, Test};
use origen::prog_gen::ParamValue;
use origen::prog_gen::PatternGroupType;
use origen::testers::SupportedTester;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Patset {
    pub name: String,
    pub tester: SupportedTester,
    pub id: usize,
}

#[pymethods]
impl IGXL {
    fn new_test_instance(
        &mut self,
        name: String,
        library: Option<String>,
        template: String,
    ) -> PyResult<Test> {
        let library = match library {
            Some(x) => x,
            None => "std".to_string(),
        };

        let t = Test::new(name.clone(), self.tester.to_owned(), library, template)?;

        t.set_attr("test_name", ParamValue::String(name))?;

        Ok(t)
    }

    fn new_patset(&mut self, name: String) -> PyResult<PatternGroup> {
        let p = PatternGroup::new(name, self.tester.clone(), Some(PatternGroupType::Patset))?;
        Ok(p)
    }
}
