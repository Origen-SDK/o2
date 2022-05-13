use pyo3::prelude::*;
use pyapi_metal::prelude::frontend::*;
use super::config_value_map_into_pydict;
use crate::optional_config_value_map_into_pydict;

#[pyfunction]
pub fn ldaps() -> PyResult<Py<PyDataStoreCategory>> {
    let needs_pop = crate::om::frontend::with_frontend(|f| {
        Ok(f.ensure_data_store_category(origen::FE_CAT_NAME__LDAPS)?.0)
    })?;

    with_py_frontend( |py, f| {
        // TODO wrap data_store access somewhere
        let py_cat = f.data_stores.extract::<PyRef<pyapi_metal::frontend::PyDataStores>>(py)?.ensured_cat(origen::FE_CAT_NAME__LDAPS)?;
        let mut cat = py_cat.extract::<PyRefMut<pyapi_metal::frontend::PyDataStoreCategory>>(py)?;

        if needs_pop {
            // TODO make this a constant
            let ldap_mod = pyo3::types::PyModule::import(py, "origen_metal.utils.ldap")?;
            let ldap_cls = ldap_mod.getattr("LDAP")?;
            for (name, config) in &crate::ORIGEN_CONFIG.ldaps {
                let pylist = pyo3::types::PyList::new(py, [name, &config.server, &config.base]);
                if let Some(a) = config.auth.as_ref() {
                    // TODO use optional config value macro
                    pylist.append(config_value_map_into_pydict(py, &mut a.iter())?)?;
                } else {
                    pylist.append(py.None())?;
                }
                pylist.append(config.continuous_bind)?;
                pylist.append(optional_config_value_map_into_pydict!(py, config.populate_user_config.as_ref()))?;
                pylist.append(config.timeout)?;
                cat.add(py, name, ldap_cls, Some(pylist), None, None)?;
            }
        }
        drop(cat);
        Ok(py_cat)
    })
}
