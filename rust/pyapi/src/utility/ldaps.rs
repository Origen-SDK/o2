use pyapi_metal::prelude::frontend::*;
use pyapi_metal::prelude::config::*;
use pyapi_metal::utils::ldap::import_frontend_ldap;
use pyo3::prelude::*;

#[pyfunction]
pub fn ldaps() -> PyResult<Py<PyDataStoreCategory>> {
    with_required_py_category(origen::FE_CAT_NAME__LDAPS, |_py, cat| {
        Ok(cat.into())
    })
}

#[pyfunction]
pub fn boot_ldaps(py: Python, mut cat: PyRefMut<PyDataStoreCategory>) -> PyResult<()> {
    log_trace!("Booting LDAPs from Origen config...");
    for (name, config) in &crate::ORIGEN_CONFIG.ldaps {
        log_trace!("Loading LDAP '{}'", name);
        let pylist = pyo3::types::PyList::new(py, [name, &config.server, &config.base]);
        if let Some(a) = config.auth.as_ref() {
            pylist.append(config_value_map_into_pydict(py, &mut a.iter())?)?;
        } else {
            pylist.append(py.None())?;
        }
        pylist.append(config.continuous_bind)?;
        pylist.append(optional_config_value_map_into_pydict!(
            py,
            config.populate_user_config.as_ref()
        ))?;
        pylist.append(config.timeout)?;
        cat.add(py, name, import_frontend_ldap(py)?.1, Some(pylist), None, None)?;
    }
    Ok(())
}
