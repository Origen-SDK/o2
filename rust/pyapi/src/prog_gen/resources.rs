use crate::utility::caller::src_caller_meta;
use origen_metal::prog_gen::flow_api;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};

#[pyclass]
#[derive(Debug, Clone)]
pub struct Resources {
    ref_id: usize,
}

impl Resources {
    pub fn new() -> Resources {
        Resources { ref_id: 0 }
    }
}

#[pymethods]
impl Resources {
    fn __enter__(&mut self) -> PyResult<()> {
        self.ref_id = flow_api::start_resources(src_caller_meta())?;
        Ok(())
    }

    fn __exit__(
        &mut self,
        ty: Option<&PyType>,
        _value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> bool {
        if ty.is_none() {
            flow_api::end_block(self.ref_id)
                .expect("Something has gone wrong closing resources block");
        }
        false
    }
}
