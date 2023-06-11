static MOD_PATH: &'static str = "origen_metal._origen_metal";
static MOD: &'static str = "frontend";
static PY_FRONTEND: &'static str = "__py_frontend__";

crate::lazy_static! {
    pub static ref LOOKUP_HOME_DIR_FUNC_KEY: &'static str = "lookup_default_home_dir_function";
}

#[macro_export]
macro_rules! frontend_mod {
    ($py:expr) => {{
        $py.import(crate::frontend::MOD_PATH)?
            .getattr(crate::frontend::MOD)?
            .extract::<&pyo3::types::PyModule>()?
    }};
}

mod _frontend;
mod py_data_stores;
mod py_frontend;

use origen_metal::log_trace;
use pyo3::prelude::*;

pub use _frontend::Frontend;
pub use py_data_stores::{PyDataStoreCategory, PyDataStores};
pub use py_frontend::PyFrontend;
pub use py_frontend::{with_mut_py_frontend, with_py_frontend, with_required_rc, with_py_data_stores, with_mut_py_data_stores, with_required_mut_py_category, with_required_py_category};

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let fm = PyModule::new(py, "frontend")?;
    fm.add_class::<PyFrontend>()?;
    fm.add_class::<PyDataStores>()?;
    fm.add_class::<PyDataStoreCategory>()?;
    fm.add_function(wrap_pyfunction!(initialize, fm)?)?;
    fm.add_function(wrap_pyfunction!(frontend, fm)?)?;
    fm.add_function(wrap_pyfunction!(reset, fm)?)?;
    m.add_submodule(fm)?;
    Ok(())
}

pub(crate) fn with_frontend_mod<F, T>(mut func: F) -> PyResult<T>
where
    F: FnMut(Python, &PyModule) -> PyResult<T>,
{
    Python::with_gil(|py| {
        let fm = frontend_mod!(py);
        func(py, fm)
    })
}

#[pyfunction]
pub(crate) fn frontend(py: Python) -> PyResult<Option<PyRef<PyFrontend>>> {
    if origen_metal::frontend::frontend_set()? {
        let m = frontend_mod!(py);
        Ok(Some(
            m.getattr(PY_FRONTEND)?.extract::<PyRef<PyFrontend>>()?,
        ))
    } else {
        Ok(None)
    }
}

#[pyfunction]
pub fn initialize(_py: Python) -> PyResult<bool> {
    if origen_metal::frontend::frontend_set()? {
        log_trace!("PyAPI Metal Frontend Already Initialized");
        Ok(false)
    } else {
        log_trace!("PyAPI Metal Frontend Not Initialized... Initializing...");
        origen_metal::frontend::set_frontend(Box::new(Frontend::new()?))?;
        Ok(true)
    }
}

#[pyfunction]
pub(crate) fn reset(_py: Python) -> PyResult<()> {
    origen_metal::frontend::reset()?;
    with_frontend_mod(|py, m| m.setattr(PY_FRONTEND, py.None()))
}

use super::framework::outcomes::Outcome as PyOutcome;
#[cfg(debug_assertions)]
use pyo3::types::{PyDict, PyList};

/// The following are all test functions that are only available when debugging metal itself.
/// Many of these, especially, "getters" make redundant round-trips.
///     That is, if you need the Python 'DataStoreCategory' for example, getting it from the backend then converting
///     it back to the Python type is unnecessary outside of the testing process.
/// These should not be used as examples for production code.
#[cfg(debug_assertions)]
pub(crate) fn define_tests(_py: Python, m: &PyModule) -> PyResult<()> {
    // Global data stores functions
    m.add_function(wrap_pyfunction!(backend_contains_cat, m)?)?;
    m.add_function(wrap_pyfunction!(backend_add_cat, m)?)?;
    m.add_function(wrap_pyfunction!(backend_remove_cat, m)?)?;
    m.add_function(wrap_pyfunction!(backend_available_cats, m)?)?;

    // Single category functions
    m.add_function(wrap_pyfunction!(backend_cat_contains_store, m)?)?;
    m.add_function(wrap_pyfunction!(backend_available_stores_for_cat, m)?)?;
    m.add_function(wrap_pyfunction!(backend_add_store, m)?)?;
    m.add_function(wrap_pyfunction!(backend_remove_store, m)?)?;

    // Single data store functions
    m.add_function(wrap_pyfunction!(backend_get_name, m)?)?;
    m.add_function(wrap_pyfunction!(backend_get_category, m)?)?;
    m.add_function(wrap_pyfunction!(backend_get_class, m)?)?;
    m.add_function(wrap_pyfunction!(backend_store_contains, m)?)?;
    m.add_function(wrap_pyfunction!(backend_get_stored, m)?)?;
    m.add_function(wrap_pyfunction!(backend_store_item, m)?)?;
    m.add_function(wrap_pyfunction!(backend_remove_item, m)?)?;
    m.add_function(wrap_pyfunction!(backend_all_items_in_store, m)?)?;
    m.add_function(wrap_pyfunction!(backend_call, m)?)?;

    // Other
    m.add("backend_test_cat_name", "BE_cat")?;
    m.add("backend_test_store_name", "BE_ds")?;
    m.add("backend_test_store_with_opts_name", "BE_ds_with_opts")?;

    Ok(())
}

// Global data stores functions //

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_contains_cat(_py: Python, category: &str) -> PyResult<bool> {
    Ok(origen_metal::frontend::with_frontend(|f| {
        Ok(f.contains_data_store_category(category)?)
    })?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_add_cat(_py: Python, category: &str) -> PyResult<()> {
    origen_metal::frontend::with_frontend(|f| f.add_data_store_category(category, None, None))?;
    Ok(())
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_remove_cat(_py: Python, category: &str) -> PyResult<()> {
    origen_metal::frontend::with_frontend(|f| f.remove_data_store_category(category))?;
    Ok(())
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_available_cats(_py: Python) -> PyResult<Vec<String>> {
    Ok(origen_metal::frontend::with_frontend(|f| {
        Ok(f.available_data_store_categories()?)
    })?)
}

// Single category functions //

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_cat_contains_store(
    _py: Python,
    category: &str,
    store: &str,
) -> PyResult<bool> {
    let f = origen_metal::frontend::require()?;
    Ok(f.with_data_store_category(category, |cat| cat.contains_data_store(store))?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_available_stores_for_cat(
    _py: Python,
    category: &str,
) -> PyResult<Vec<String>> {
    let f = origen_metal::frontend::require()?;
    Ok(f.with_data_store_category(category, |cat| cat.available_data_stores())?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_add_store(
    _py: Python,
    category: &str,
    name: &str,
    parameters: &pyo3::types::PyDict,
    backend: Option<&pyo3::types::PyDict>,
) -> PyResult<()> {
    let f = origen_metal::frontend::require()?;
    f.with_data_store_category(category, |cat| {
        cat.add_data_store(
            name,
            crate::_helpers::typed_value::from_pydict(parameters)?,
            crate::_helpers::typed_value::from_optional_pydict(backend)?,
        )
    })?;
    Ok(())
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_remove_store(_py: Python, category: &str, name: &str) -> PyResult<()> {
    let f = origen_metal::frontend::require()?;
    f.with_data_store_category(category, |cat| cat.remove_data_store(name))?;
    Ok(())
}

// Single data store functions

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_get_name(_py: Python, category: &str, data_store: &str) -> PyResult<String> {
    let f = origen_metal::frontend::require()?;
    Ok(f.with_data_store(category, data_store, |ds| Ok(ds.name()?.to_string()))?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_get_category(
    _py: Python,
    category: &str,
    data_store: &str,
) -> PyResult<String> {
    let f = origen_metal::frontend::require()?;
    Ok(f.with_data_store(category, data_store, |ds| {
        Ok(ds.category()?.name().to_string())
    })?)
}

#[cfg(debug_assertions)]
#[pyfunction(opts = "**")]
pub(crate) fn backend_get_class(
    _py: Python,
    category: &str,
    data_store: &str,
    opts: Option<&pyo3::types::PyDict>,
) -> PyResult<String> {
    let f = origen_metal::frontend::require()?;
    Ok(f.with_data_store(category, data_store, |ds| {
        Ok(ds
            .class(crate::_helpers::typed_value::from_optional_pydict(opts)?)?
            .to_string())
    })?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_all_items_in_store(
    _py: Python,
    category: &str,
    data_store: &str,
) -> PyResult<Vec<String>> {
    let f = origen_metal::frontend::require()?;
    Ok(f.with_data_store(category, data_store, |ds| ds.keys())?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_store_contains(
    _py: Python,
    category: &str,
    data_store: &str,
    key: &str,
) -> PyResult<bool> {
    let f = origen_metal::frontend::require()?;
    Ok(f.with_data_store(category, data_store, |ds| ds.contains(key))?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_get_stored(
    _py: Python,
    category: &str,
    data_store: &str,
    key: &str,
) -> PyResult<Option<PyObject>> {
    let f = origen_metal::frontend::require()?;
    Ok(f.with_data_store(category, data_store, |ds| {
        let tv = ds.get(key)?;
        Ok(crate::_helpers::typed_value::to_pyobject(tv, None)?)
    })?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_store_item(
    _py: Python,
    category: &str,
    data_store: &str,
    key: &str,
    val: &PyAny,
) -> PyResult<bool> {
    let f = origen_metal::frontend::require()?;
    Ok(f.with_data_store(category, data_store, |ds| {
        ds.store(key, crate::_helpers::typed_value::from_pyany(val)?)
    })?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_remove_item(
    _py: Python,
    category: &str,
    data_store: &str,
    key: &str,
) -> PyResult<(bool, Option<PyObject>)> {
    let f = origen_metal::frontend::require()?;
    Ok(f.with_data_store(category, data_store, |ds| {
        let item = crate::_helpers::typed_value::to_pyobject(ds.remove(key)?, None)?;
        Ok((item.is_some(), item))
    })?)
}

#[cfg(debug_assertions)]
#[pyfunction]
pub(crate) fn backend_call(
    py: Python,
    category: &str,
    data_store: &str,
    func: &str,
    args: Option<&PyList>,
    kwargs: Option<&PyDict>,
) -> PyResult<Py<crate::framework::outcomes::Outcome>> {
    let f = origen_metal::frontend::require()?;
    PyOutcome::to_py(
        py,
        &f.with_data_store(category, data_store, |ds| {
            ds.call(
                func,
                crate::_helpers::typed_value::from_optional_pylist(args)?,
                crate::_helpers::typed_value::from_optional_pydict(kwargs)?,
                None,
            )
        })?,
    )
}
