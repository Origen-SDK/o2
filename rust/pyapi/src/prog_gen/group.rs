use origen::prog_gen::{flow_api, FlowID, GroupType};
use origen::testers::SupportedTester;
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
    flow_id: Option<FlowID>,
    pub open_conditions: Option<Vec<usize>>,
}

impl Group {
    pub fn new(
        name: String,
        tester: Option<SupportedTester>,
        kind: GroupType,
        flow_id: Option<FlowID>,
    ) -> Group {
        Group {
            name: name,
            tester: tester,
            kind: kind,
            flow_id: flow_id,
            ref_id: 0,
            open_conditions: None,
        }
    }
}

#[pymethods]
impl Group {
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
        ty: Option<&PyType>,
        _value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> bool {
        if ty.is_none() {
            flow_api::end_block(self.ref_id).expect(&format!(
                "Something has gone wrong closing group '{}'",
                self.name
            ));
            if let Some(ref_ids) = &self.open_conditions {
                for id in ref_ids {
                    flow_api::end_block(*id).expect(&format!(
                        "Something has gone wrong closing a condition that is wrapping group '{}'",
                        self.name
                    ));
                }
            }
        }
        false
    }
}
