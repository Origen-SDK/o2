use super::src_caller_meta;
use origen_metal::prog_gen::flow_api;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};

/// A context manager which repeats the flow content generated within it
#[pyclass]
#[derive(Debug, Clone)]
pub struct LoopBlock {
    count: u32,
    variable: String,
    descending: bool,
    ref_id: usize,
}

impl LoopBlock {
    pub fn new(count: u32, variable: String, descending: bool) -> LoopBlock {
        LoopBlock {
            count: count,
            variable: variable,
            descending: descending,
            ref_id: 0,
        }
    }
}

#[pymethods]
impl LoopBlock {
    fn __enter__(&mut self) -> PyResult<()> {
        self.ref_id = flow_api::start_loop(
            Some(self.count),
            Some(self.variable.clone()),
            self.descending,
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
