use crate::application::{get_pyapp, PyApplication};
use crate::utility::results::{BuildResult, UploadResult};
use origen::core::frontend as ofrontend;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use origen_metal::{Result, Outcome};

pub struct Publisher {}

impl ofrontend::Publisher for Publisher {
    fn build_package(&self) -> Result<Outcome> {
        Python::with_gil(|py| {
            let pyapp = get_pyapp(py)?;
            let pb = PyApplication::_get_publisher(pyapp, py)?;
            let py_pbr = pb.call_method0(py, "build_package")?;
            let pbr = py_pbr.extract::<PyRef<BuildResult>>(py)?;
            Ok(pbr.build_result()?.clone())
        })
    }

    fn upload(
        &self,
        build_result: &Outcome,
        dry_run: bool,
    ) -> Result<Outcome> {
        Python::with_gil(|py| {
            let pyapp = get_pyapp(py)?;
            let pb = PyApplication::_get_publisher(pyapp, py)?;
            let py_pbr = pb.call_method1(
                py,
                "upload",
                PyTuple::new(
                    py,
                    &[
                        BuildResult::to_py(py, build_result)?.to_object(py),
                        dry_run.to_object(py),
                    ],
                ),
            )?;
            let pur = py_pbr.extract::<PyRef<UploadResult>>(py)?;
            Ok(pur.upload_result()?.clone())
        })
    }
}
