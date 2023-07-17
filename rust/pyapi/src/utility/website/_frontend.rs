use crate::application::{get_pyapp, PyApplication};
use crate::utility::results::BuildResult;
use origen::core::frontend as ofrontend;
use pyo3::prelude::*;
use origen_metal::{Result, Outcome};

pub struct Website {}

impl ofrontend::Website for Website {
    fn build(&self) -> Result<Outcome> {
        Python::with_gil(|py| {
            let pyapp = get_pyapp(py)?;
            let web = PyApplication::_get_website(pyapp, py)?;
            let py_pbr = web.call_method0(py, "build")?;
            let pbr = py_pbr.extract::<PyRef<BuildResult>>(py)?;
            Ok(pbr.build_result()?.clone())
        })
    }
}
