use super::src_caller_meta;
use origen_metal::prog_gen::{flow_api, LoopCount};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};

/// A context manager which repeats the flow content generated within it
#[pyclass]
#[derive(Debug, Clone)]
pub struct Loop {
    count: LoopCount,
    variable: String,
    descending: bool,
    start: i64,
    ref_id: usize,
    pub open_conditions: Option<Vec<usize>>,
}

impl Loop {
    pub fn new(count: LoopCount, variable: String, descending: bool, start: i64) -> Loop {
        Loop {
            count,
            variable,
            descending,
            start,
            ref_id: 0,
            open_conditions: None,
        }
    }
}

#[pymethods]
impl Loop {
    fn __enter__(&mut self) -> PyResult<Loop> {
        self.ref_id = flow_api::start_loop(
            Some(self.count.clone()),
            Some(self.variable.clone()),
            self.descending,
            self.start,
            src_caller_meta(),
        )?;
        Ok(self.clone())
    }

    fn __exit__(
        &mut self,
        _ty: Option<&PyType>,
        _value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        flow_api::end_block(self.ref_id)?;
        if let Some(ref_ids) = &self.open_conditions {
            for id in ref_ids {
                flow_api::end_block(*id)?;
            }
        }
        Ok(false)
    }
}
