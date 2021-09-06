mod framework;
mod frontend;
mod utils;

use pyo3::prelude::*;
use pyo3::py_run;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[pymodule]
fn _origen_metal(py: Python, m: &PyModule) -> PyResult<()> {
    framework::define(py, m)?;
    utils::define(py, m)?;
    frontend::define(py, m)?;
    m.setattr("__version__", built_info::PKG_VERSION)?;
    m.setattr(
        "__origen_metal_backend_version__",
        origen_metal::VERSION.to_string(),
    )?;

    #[cfg(debug_assertions)]
    {
        // For debug builds, include the __test__ module in _origen_metal
        let test_sm = PyModule::new(py, "__test__")?;
        utils::revision_control::define_tests(py, test_sm)?;
        m.add_submodule(test_sm)?;
    }

    Ok(())
}

fn py_submodule<F>(py: Python, parent: &PyModule, path: &str, func: F) -> PyResult<()>
where
    F: FnOnce(&PyModule) -> PyResult<()>,
{
    let m = PyModule::new(py, path)?;
    func(m)?;
    // py_run! is quick-and-dirty; should be replaced by PyO3 API calls in actual code
    py_run!(py, m, &format!("import sys; sys.modules['{}'] = m", path));
    parent.add_submodule(m)?;
    Ok(())
}

#[macro_export]
macro_rules! pypath {
    ($py:expr, $path:expr) => {{
        use pyo3::types::IntoPyDict;
        let locals = [("pathlib", $py.import("pathlib")?)].into_py_dict($py);
        let obj = $py.eval(
            &format!("pathlib.Path(r\"{}\").resolve()", $path),
            None,
            Some(&locals),
        )?;
        obj.to_object($py)
    }};
}

#[macro_export]
macro_rules! bail_with_runtime_error {
    ($message:expr) => {{
        Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>($message))
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
