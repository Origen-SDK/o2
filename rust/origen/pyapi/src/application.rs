use origen::core::application::target::CURRENT_TARGET;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pymodule]
/// Implements the module _origen.app in Python
pub fn app(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(current_target))?;

    Ok(())
}

#[pyfunction]
/// Returns a dict containing information about the currently selected
/// target/environment
fn current_target(py: Python) -> PyResult<PyObject> {
    let ret = pyo3::types::PyDict::new(py);
    // Don't think an error can really happen here, so not handled
    let _ = ret.set_item("target_name", &CURRENT_TARGET.target_name);
    if CURRENT_TARGET.target_file.is_some() {
        let _ = ret.set_item(
            "target_file",
            format!("{}", CURRENT_TARGET.target_file.as_ref().unwrap().display()),
        );
    } else {
        let _ = ret.set_item("target_file", None::<String>);
    }
    let _ = ret.set_item("env_name", &CURRENT_TARGET.env_name);
    if CURRENT_TARGET.env_file.is_some() {
        let _ = ret.set_item(
            "env_file",
            format!("{}", CURRENT_TARGET.env_file.as_ref().unwrap().display()),
        );
    } else {
        let _ = ret.set_item("env_file", None::<String>);
    }
    let _ = ret.set_item("is_loaded", &CURRENT_TARGET.is_loaded);
    Ok(ret.into())
}
