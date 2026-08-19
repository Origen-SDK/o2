#[macro_use]
pub mod _helpers;
pub mod framework;
pub mod frontend;
pub mod prelude;
pub mod utils;
pub mod prog_gen;

#[macro_use]
pub extern crate origen_metal_backend as origen_metal;

use origen_metal::lazy_static::lazy_static;
use origen_metal::cfg_if::cfg_if;

use pyo3::prelude::*;
use pyo3::py_run;

pub use crate::framework::Outcome as PyOutcome;
pub(crate) use origen_metal::Result as OMResult;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "_origen_metal")?;
    _define(py, subm)?;
    m.add_submodule(subm)?;
    Ok(())
}

pub fn _define(py: Python, m: &PyModule) -> PyResult<()> {
    framework::define(py, m)?;
    utils::define(py, m)?;
    frontend::define(py, m)?;
    prog_gen::interface::define(py, m)?;
    prog_gen::define(py, m)?;
    prog_gen::tester_apis::define(py, m)?;
    m.setattr("__version__", built_info::PKG_VERSION)?;
    m.setattr(
        "__origen_metal_backend_version__",
        origen_metal::VERSION.to_string(),
    )?;
    m.setattr("running_on_windows", origen_metal::running_on_windows())?;
    m.setattr("running_on_linux", origen_metal::running_on_linux())?;

    #[cfg(debug_assertions)]
    {
        // For debug builds, include the __test__ module in _origen_metal
        let test_sm = PyModule::new(py, "__test__")?;
        utils::revision_control::define_tests(py, test_sm)?;
        frontend::define_tests(py, test_sm)?;
        _helpers::define_tests(py, test_sm)?;
        m.add_submodule(test_sm)?;
    }
    Ok(())
}

#[pymodule]
pub fn _origen_metal(py: Python, m: &PyModule) -> PyResult<()> {
    _define(py, m)
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
            &format!("pathlib.Path(r\"{}\")", $path),
            None,
            Some(&locals),
        )?;
        obj.to_object($py)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use origen_metal::prog_gen::ParamValue;
    use origen_metal::TypedValue;

    #[test]
    fn initializes_module_and_converts_python_values() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let module = PyModule::new(py, "_origen_metal")?;
            _origen_metal(py, module)?;
            assert!(module.hasattr("framework")?);

            let pathlib = PyModule::import(py, "pathlib")?;
            let path = pathlib.getattr("Path")?.call1(("some/path",))?;
            assert_eq!(
                _helpers::pypath_as_pathbuf(path)?,
                std::path::PathBuf::from("some/path")
            );

            let value = true.to_object(py);
            assert!(matches!(
                _helpers::typed_value::extract_as_typed_value(value.as_ref(py))?,
                TypedValue::Bool(true)
            ));
            Ok(())
        })
    }

    #[test]
    fn preserves_python_list_formatting_for_limits() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let list = pyo3::types::PyList::new(py, [1e-6, 2e-6, 3e-6]);
            assert_eq!(
                crate::prog_gen::to_limit_param_value(list)?,
                Some(ParamValue::Any("[1e-06, 2e-06, 3e-06]".to_string()))
            );

            let tuple = pyo3::types::PyTuple::new(py, [4e-6, 5e-6]);
            assert_eq!(
                crate::prog_gen::to_limit_param_value(tuple)?,
                Some(ParamValue::Any("(4e-06, 5e-06)".to_string()))
            );
            Ok(())
        })
    }
}
