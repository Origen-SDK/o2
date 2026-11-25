use super::src_caller_meta;
use origen_metal::prog_gen::{flow_api, FlowCondition};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};

#[pyclass]
#[derive(Debug, Clone)]
pub struct Condition {
    kind: FlowCondition,
    ref_id: usize,
    /// True when its flags or IDs were given as a list, see Node::list_form()
    list_form: bool,
}

impl Condition {
    pub fn new(kind: FlowCondition) -> Condition {
        Condition::new_listed(false, kind)
    }

    /// A condition whose flags or IDs were given as a list when list_form is true
    pub fn new_listed(list_form: bool, kind: FlowCondition) -> Condition {
        Condition {
            kind: kind,
            ref_id: 0,
            list_form: list_form,
        }
    }
}

#[pymethods]
impl Condition {
    fn __enter__(&mut self) -> PyResult<()> {
        self.ref_id = flow_api::start_condition_with_list_form(
            self.kind.clone(),
            self.list_form,
            src_caller_meta(),
        )?;
        Ok(())
    }

    fn __exit__(
        &mut self,
        _ty: Option<&PyType>,
        _value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        flow_api::end_block(self.ref_id)?;
        Ok(false)
    }
}
