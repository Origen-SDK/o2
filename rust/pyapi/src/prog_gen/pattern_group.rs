use crate::utility::caller::src_caller_meta;
use origen_metal::prog_gen::{flow_api, PatternGroupType, SupportedTester};
use origen::Result;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct PatternGroup {
    #[pyo3(get)]
    pub name: String,
    pub tester: SupportedTester,
    pub id: usize,
    pub kind: Option<PatternGroupType>,
}

impl PatternGroup {
    pub fn new(
        name: String,
        tester: SupportedTester,
        kind: Option<PatternGroupType>,
    ) -> Result<PatternGroup> {
        let id = flow_api::define_pattern_group(
            name.clone(),
            tester.clone(),
            kind.clone(),
            src_caller_meta(),
        )?;

        Ok(PatternGroup {
            name: name,
            tester: tester,
            kind: kind,
            id: id,
        })
    }
}

#[pymethods]
impl PatternGroup {
    pub fn append(&self, pattern_path: String, start_label: Option<String>) -> PyResult<()> {
        flow_api::push_pattern_to_group(self.id, pattern_path, start_label, src_caller_meta())?;
        Ok(())
    }
}
