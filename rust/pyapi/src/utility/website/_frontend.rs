use crate::application::{get_pyapp, PyApplication};
use crate::utility::results::BuildResult;
use ofrontend::BuildResult as OrigenBuildResult;
use origen::core::frontend as ofrontend;
use origen::Result as OResult;
use pyo3::prelude::*;

pub struct Website {}

impl ofrontend::Website for Website {
    fn build(&self) -> OResult<OrigenBuildResult> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let web = PyApplication::_get_website(pyapp, py)?;
        let py_pbr = web.call_method0(py, "build")?;
        let pbr = py_pbr.extract::<PyRef<BuildResult>>(py)?;
        Ok(pbr.build_result()?.clone())
    }
}
