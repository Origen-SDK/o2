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
}

impl Group {
    pub fn new(name: String, tester: Option<SupportedTester>, kind: GroupType) -> Group {
        Group {
            name: name,
            tester: tester,
            kind: kind,
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
        flow_api::end_group(self.ref_id).expect(&format!(
            "Something has gone wrong closing group '{}'",
            self.name
        ));
        true
    }
}
