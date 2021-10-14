use super::{with_py_frontend, PyFrontend};
use origen_metal::frontend::RevisionControlFrontendAPI;
use origen_metal::Result as OMResult;
use pyo3::prelude::*;
use origen_metal::log_trace;

pub struct Frontend {
    rc: crate::utils::revision_control::_frontend::RevisionControlFrontend,
}

impl Frontend {
    pub fn new() -> PyResult<Self> {
        log_trace!("PyAPI Metal: Creating new frontend");
        PyFrontend::initialize()?;
        Ok(Self {
            rc: crate::utils::revision_control::_frontend::RevisionControlFrontend {},
        })
    }
}

impl origen_metal::frontend::FrontendAPI for Frontend {
    fn revision_control(&self) -> OMResult<Option<&dyn RevisionControlFrontendAPI>> {
        Ok(with_py_frontend(|_py, py_frontend| {
            if py_frontend.rc.is_some() {
                Ok(Some(&self.rc as &dyn RevisionControlFrontendAPI))
            } else {
                Ok(None)
            }
        })?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
