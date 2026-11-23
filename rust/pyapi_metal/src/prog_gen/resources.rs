use super::src_caller_meta;
use origen_metal::prog_gen::flow_api;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};

#[pyclass]
#[derive(Debug, Clone)]
pub struct Resources {
    ref_id: usize,
    keep_flows: bool,
}

impl Resources {
    pub fn new() -> Resources {
        Resources::new_with_options(false)
    }

    /// A resources block which keeps the flow-type groups and sub-flows within it as flow
    /// definitions when keep_flows is true, see flow_api::start_resources_keep_flows
    pub fn new_with_options(keep_flows: bool) -> Resources {
        Resources {
            ref_id: 0,
            keep_flows,
        }
    }
}

#[pymethods]
impl Resources {
    fn __enter__(&mut self) -> PyResult<()> {
        self.ref_id = if self.keep_flows {
            flow_api::start_resources_keep_flows(src_caller_meta())?
        } else {
            flow_api::start_resources(src_caller_meta())?
        };
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
