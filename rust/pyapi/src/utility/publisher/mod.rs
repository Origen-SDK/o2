pub mod _frontend;

use super::app_utility;
use crate::runtime_error;
use origen::STATUS;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "publisher")?;
    subm.add_wrapped(wrap_pyfunction!(app_publisher))?;
    m.add_submodule(subm)?;
    Ok(())
}

/// Creates a publisher from the application's ``config.toml``
#[pyfunction]
fn app_publisher() -> PyResult<Option<PyObject>> {
    match &STATUS.app {
        Some(a) => {
            let c = a.config();
            app_utility(
                "publisher",
                c.publisher.as_ref(),
                Some("origen.utility.publishers.poetry.Poetry"),
                true,
            )
        }
        None => {
            return runtime_error!(
                "Cannot retrieve the application's publisher config: no application found!"
            )
        }
    }
}
