use pyo3::prelude::*;
use std::collections::HashMap;
use pyo3::class::mapping::PyMappingProtocol;
use pyo3::wrap_pyfunction;

#[pymodule]
fn ldap(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<LDAPs>()?;
    m.add_class::<LDAP>()?;
    m.add_wrapped(wrap_pyfunction!(ldaps))?;
    Ok(())
}

#[pyfunction]
fn ldaps(_py: Python) -> PyResult<LDAPs> {
    Ok(LDAPs {})
}

/// Essentially just a dictionary to get/list available LDAPs from the frontend
#[pyclass]
pub struct LDAPs {}

#[pymethods]
impl LDAPs {
    fn keys(&self) -> PyResult<Vec<String>> {
        let ldaps = origen::ldaps();
        Ok(ldaps.ldaps().keys().cloned().collect())
    }

    // fn values(&self) -> PyResult<Vec<LDAP>> {}
    // fn items(&self) -> PyResult<Vec<(String, LDAP)>> {}
    // fn get(&self, action: &PyAny) -> PyResult<Option<LDAP>> {}
}

#[pyproto]
impl PyMappingProtocol for LDAPs {
    fn __getitem__(&self, ldap: &str) -> PyResult<LDAP> {
        let ldaps = origen::ldaps();
        if ldaps.ldaps().contains_key(ldap) {
            Ok(LDAP {
                name: ldap.to_string()
            })
        } else {
            crate::runtime_error!(format!("No LDAP available named {}", ldap))
        }
    }
}

// #[pyproto]
// impl pyo3::class::sequence::PySequenceProtocol for LDAPs {
//     fn __contains__(&self, item: &PyAny) -> PyResult<bool> {
//         match pyo3::PyMappingProtocol::__getitem__(self, &item) {
//             Ok(_) => Ok(true),
//             Err(_) => Ok(false),
//         }
//     }
// }

#[pyclass]
pub struct LDAPsIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for LDAPsIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        if slf.i >= slf.keys.len() {
            return Ok(None);
        }
        let name = slf.keys[slf.i].clone();
        slf.i += 1;
        Ok(Some(name))
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for LDAPs {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<LDAPsIter> {
        Ok(LDAPsIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}

#[pyclass(subclass)]
pub struct LDAP {
    name: String
}

#[pymethods]
impl LDAP {

    #[getter]
    fn get_server(&self) -> PyResult<String> {
        let ldaps = origen::ldaps();
        let ldap = ldaps._get(&self.name)?;
        Ok(ldap.server().to_string())
    }

    #[getter]
    fn get_base(&self) -> PyResult<String> {
        let ldaps = origen::ldaps();
        let ldap = ldaps._get(&self.name)?;
        Ok(ldap.base().to_string())
    }

    fn search(&self, filter: &str, attrs: Vec<&str>) -> PyResult<
        HashMap<String, (
            HashMap<String, Vec<String>>,
            HashMap<String, Vec<Vec<u8>>>
        )>
    > {
        let mut ldaps = origen::ldaps();
        let ldap = ldaps._get_mut(&self.name)?;
        Ok(ldap.search(filter, attrs)?)
    }

    fn single_filter_search(&self, filter: &str, attrs: Vec<&str>) -> PyResult<(
        HashMap<String, Vec<String>>,
        HashMap<String, Vec<Vec<u8>>>
    )> {
        let mut ldaps = origen::ldaps();
        let ldap = ldaps._get_mut(&self.name)?;
        Ok(ldap.single_filter_search(filter, attrs)?)
    }
}