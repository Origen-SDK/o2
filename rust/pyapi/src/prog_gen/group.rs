use origen::prog_gen::{flow_api, GroupType};
use origen::testers::SupportedTester;
use pyo3::class::PyContextProtocol;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};

#[pyclass]
#[derive(Debug, Clone)]
pub struct Group {
    #[pyo3(get, set)]
    pub name: String,
    pub tester: Option<SupportedTester>,
    pub kind: GroupType,
    ref_id: usize,
    flow_id: Option<String>,
}

impl Group {
    pub fn new(
        name: String,
        tester: Option<SupportedTester>,
        kind: GroupType,
        flow_id: Option<String>,
    ) -> Group {
        Group {
            name: name,
            tester: tester,
            kind: kind,
            flow_id: flow_id,
            ref_id: 0,
        }
    }
}

#[pyproto]
impl PyContextProtocol for Group {
    fn __enter__(&mut self) -> PyResult<Group> {
        self.ref_id = flow_api::start_group(
            self.name.clone(),
            self.tester.clone(),
            self.kind.clone(),
            self.flow_id.clone(),
            None,
        )?;
        Ok(self.clone())
    }

    fn __exit__(
        &mut self,
        _ty: Option<&'p PyType>,
        _value: Option<&'p PyAny>,
        _traceback: Option<&'p PyAny>,
    ) -> bool {
        flow_api::end_block(self.ref_id).expect(&format!(
            "Something has gone wrong closing group '{}'",
            self.name
        ));
        true
    }
}
