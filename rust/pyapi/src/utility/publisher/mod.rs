pub mod _frontend;

use super::app_utility;
use crate::runtime_error;
use origen::STATUS;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;

#[pymodule]
pub fn publisher(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(app_publisher))?;
    Ok(())
}

/// Creates a publisher from the application's ``config.toml``
#[pyfunction]
fn app_publisher() -> PyResult<Option<PyObject>> {
    let mut default;
    match &STATUS.app {
        Some(a) => {
            let c = a.config();
            app_utility(
                "publisher",
                match &c.release_scribe {
                    Some(config) => Some(config),
                    None => {
                        default = HashMap::new();
                        default.insert("system".to_string(), "origen.utility.publishers.poetry.Poetry".to_string());
                        Some(&default)
                    }
                },
                None::<fn(Option<&HashMap<String, String>>) -> PyResult<Option<PyObject>>>,
            )
        }
        None => {
            return runtime_error!(
                "Cannot retrieve the application's publisher config: no application found!"
            )
        }
    }
}
