use crate::utility::caller::src_caller_meta;
use origen::prog_gen::{flow_api, FlowCondition};
use pyo3::class::PyContextProtocol;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};

#[pyclass]
#[derive(Debug, Clone)]
pub struct Condition {
    kind: FlowCondition,
    ref_id: usize,
}

impl Condition {
    pub fn new(kind: FlowCondition) -> Condition {
        Condition {
            kind: kind,
            ref_id: 0,
        }
    }
}

#[pyproto]
impl PyContextProtocol for Condition {
    fn __enter__(&mut self) -> PyResult<()> {
        self.ref_id = flow_api::start_condition(self.kind.clone(), src_caller_meta())?;
        Ok(())
    }

    fn __exit__(
        &mut self,
        _ty: Option<&'p PyType>,
        _value: Option<&'p PyAny>,
        _traceback: Option<&'p PyAny>,
    ) -> bool {
        flow_api::end_block(self.ref_id).expect(&format!(
            "Something has gone wrong closing condition '{:?}'",
            self.kind
        ));
        true
    }
}
