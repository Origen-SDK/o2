use super::metadata::{extract_as_metadata, metadata_to_pyobj};
use crate::application;
use pyo3::class::basic::CompareOp;
use pyo3::class::mapping::PyMappingProtocol;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
use std::path::PathBuf;

#[pymodule]
fn session_store(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(app_session))?;
    m.add_wrapped(wrap_pyfunction!(app_sessions))?;
    m.add_wrapped(wrap_pyfunction!(app_root))?;
    m.add_wrapped(wrap_pyfunction!(set_app_root))?;
    m.add_wrapped(wrap_pyfunction!(user_session))?;
    m.add_wrapped(wrap_pyfunction!(user_sessions))?;
    m.add_wrapped(wrap_pyfunction!(user_root))?;
    m.add_wrapped(wrap_pyfunction!(set_user_root))?;
    m.add_wrapped(wrap_pyfunction!(clear_cache))?;
    m.add_class::<SessionStore>()?;
    Ok(())
}

#[pyfunction(target = "None")]
pub fn app_session(session: Option<&PyAny>) -> PyResult<SessionStore> {
    // Can accept:
    //  None -> Top app's session
    //  String -> Session by name
    //  App -> (origen.application.Base) -> Session for this app/plugin
    let t;
    if let Some(s) = session {
        if let Ok(name) = s.extract::<String>() {
            t = Some(name);
        } else if application::is_base_app(s)? {
            t = Some(application::get_name(s)?);
        } else {
            return crate::runtime_error!(format!(
                "Could not derive session from input {}",
                s.get_type().to_string()
            ));
        }
    } else {
        t = None;
    }
    let mut s = origen::sessions();
    let sess = s.app_session(t)?;
    Ok(SessionStore::new(sess.path.clone(), true, sess.name()?))
}

#[pyfunction(target = "None")]
pub fn user_session(session: Option<&PyAny>) -> PyResult<SessionStore> {
    // Can accept:
    //  None -> Top app's session
    //  String -> Session by name
    //  App -> (origen.application.Base) -> Session for this app/plugin
    let t;
    if let Some(s) = session {
        if let Ok(name) = s.extract::<String>() {
            t = Some(name);
        } else if application::is_base_app(s)? {
            t = Some(application::get_name(s)?);
        } else {
            return crate::runtime_error!(format!(
                "Could not derive session from input {}",
                s.get_type().to_string()
            ));
        }
    } else {
        t = None;
    }
    let mut s = origen::sessions();
    let sess = s.user_session(t)?;
    Ok(SessionStore::new(sess.path.clone(), false, sess.name()?))
}

#[pyfunction()]
pub fn app_sessions(_py: Python) -> PyResult<HashMap<String, SessionStore>> {
    let mut retn: HashMap<String, SessionStore> = HashMap::new();
    let s = origen::sessions();
    for (n, p) in s.available_app_sessions()?.iter() {
        retn.insert(
            n.to_string(),
            SessionStore::new(p.to_path_buf(), true, n.to_string()),
        );
    }
    Ok(retn)
}

#[pyfunction()]
pub fn user_sessions(_py: Python) -> PyResult<HashMap<String, SessionStore>> {
    let mut retn: HashMap<String, SessionStore> = HashMap::new();
    let s = origen::sessions();
    for (n, p) in s.available_user_sessions()?.iter() {
        retn.insert(
            n.to_string(),
            SessionStore::new(p.to_path_buf(), true, n.to_string()),
        );
    }
    Ok(retn)
}

#[pyfunction]
pub fn set_app_root(root: &PyAny) -> PyResult<()> {
    let path;
    if let Ok(p) = root.extract::<String>() {
        path = p;
    } else if root.get_type().name()?.to_string() == "Path"
        || root.get_type().name()?.to_string() == "WindowsPath"
        || root.get_type().name()?.to_string() == "PosixPath"
    {
        path = root.call_method0("__str__")?.extract::<String>()?;
    } else {
        return crate::type_error!(&format!(
            "Cannot extract input as either a str or pathlib.Path object. Received {}",
            root.get_type().name()?.to_string()
        ));
    }
    let mut s = origen::sessions();
    s.app_session_root = Some(PathBuf::from(path));
    Ok(())
}

#[pyfunction]
pub fn set_user_root(root: &PyAny) -> PyResult<()> {
    let path;
    if let Ok(p) = root.extract::<String>() {
        path = p;
    } else if root.get_type().name()?.to_string() == "Path"
        || root.get_type().name()?.to_string() == "WindowsPath"
        || root.get_type().name()?.to_string() == "PosixPath"
    {
        path = root.call_method0("__str__")?.extract::<String>()?;
    } else {
        return crate::type_error!(&format!(
            "Cannot extract input as either a str or pathlib.Path object. Received {}",
            root.get_type().name()?.to_string()
        ));
    }
    let mut s = origen::sessions();
    s.user_session_root = PathBuf::from(path);
    Ok(())
}

#[pyfunction]
pub fn app_root() -> PyResult<PyObject> {
    let s = origen::sessions();
    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(crate::pypath!(py, s.get_app_session_root_string()?))
}

#[pyfunction]
pub fn user_root() -> PyResult<PyObject> {
    let s = origen::sessions();
    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(crate::pypath!(py, s.get_user_session_root_string()?))
}

#[pyfunction]
pub fn clear_cache() -> PyResult<()> {
    let mut s = origen::sessions();
    s.clear_cache()?;
    Ok(())
}

#[pyclass(subclass)]
pub struct SessionStore {
    path: PathBuf,
    app_session: bool,
    name: String,
}

#[pymethods]
impl SessionStore {
    fn refresh(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let mut s = origen::sessions();
        s.get_mut_session(slf.path.clone(), slf.app_session)?
            .refresh()?;
        Ok(slf.into())
    }

    #[getter]
    fn path(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(crate::pypath!(py, format!("{}", self.path.display())))
    }

    #[getter]
    fn is_app_session(&self) -> PyResult<bool> {
        Ok(self.app_session)
    }

    #[getter]
    fn is_user_session(&self) -> PyResult<bool> {
        Ok(!self.app_session)
    }

    #[getter]
    fn name(&self) -> PyResult<String> {
        Ok(self.name.to_string())
    }

    // fn permissions(&self) -> Result<String> {
    //     // ...
    // }

    // fn set_permissions(&self) -> Result<String> {
    //     // ...
    // }

    fn get(&self, key: &str) -> PyResult<Option<PyObject>> {
        let mut s = origen::sessions();
        let data = s
            .get_mut_session(self.path.clone(), self.app_session)?
            .retrieve(key)?;
        metadata_to_pyobj(data, Some(key))
    }

    fn delete(&self, key: &str) -> PyResult<Option<PyObject>> {
        let mut s = origen::sessions();
        let session = s.get_mut_session(self.path.clone(), self.app_session)?;
        metadata_to_pyobj(session.delete(key)?, Some(key))
    }

    fn store(slf: PyRef<Self>, key: &str, value: &PyAny) -> PyResult<Py<Self>> {
        slf._store(key, value)?;
        Ok(slf.into())
    }

    fn store_serialized(slf: PyRef<Self>, key: &str, value: &[u8]) -> PyResult<Py<Self>> {
        let mut s = origen::sessions();
        let session = s.get_mut_session(slf.path.clone(), slf.app_session)?;
        session.store_serialized(
            key.to_string(),
            value,
            Some("Python-Frontend".to_string()),
            None,
        )?;
        Ok(slf.into())
    }

    fn remove_file(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let mut s = origen::sessions();
        s.get_mut_session(slf.path.clone(), slf.app_session)?
            .remove_file()?;
        Ok(slf.into())
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        let mut s = origen::sessions();
        let session = s.get_mut_session(self.path.clone(), self.app_session)?;
        Ok(session.keys().iter().map(|k| k.to_string()).collect())
    }

    fn values(&self) -> PyResult<Vec<Option<PyObject>>> {
        let mut s = origen::sessions();
        let session = s.get_mut_session(self.path.clone(), self.app_session)?;
        let mut retn: Vec<Option<PyObject>> = vec![];
        for (k, v) in session.data()?.iter() {
            retn.push(metadata_to_pyobj(Some(v.clone()), Some(k))?);
        }
        Ok(retn)
    }

    fn items(&self) -> PyResult<Vec<(String, Option<PyObject>)>> {
        let mut s = origen::sessions();
        let session = s.get_mut_session(self.path.clone(), self.app_session)?;
        let mut retn: Vec<(String, Option<PyObject>)> = vec![];
        for (k, v) in session.data()?.iter() {
            retn.push((k.to_string(), metadata_to_pyobj(Some(v.clone()), Some(k))?));
        }
        Ok(retn)
    }
}

#[pyproto]
impl pyo3::class::basic::PyObjectProtocol for SessionStore {
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        let other_s = other.extract::<PyRef<Self>>()?;
        let s = origen::sessions();
        let origen_self = s.get_session(self.path.clone(), self.app_session)?;
        let origen_other = s.get_session(other_s.path.clone(), other_s.app_session)?;
        let result = origen_self == origen_other;

        let gil = Python::acquire_gil();
        let py = gil.python();
        match op {
            CompareOp::Eq => Ok(result.to_object(py)),
            CompareOp::Ne => Ok((!result).to_object(py)),
            _ => Ok(py.NotImplemented()),
        }
    }
}

#[pyproto]
impl PyMappingProtocol for SessionStore {
    fn __getitem__(&self, key: &str) -> PyResult<PyObject> {
        if let Some(l) = self.get(key)? {
            Ok(l)
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err({
                if self.app_session {
                    format!("Key {} not in app session {}", key, self.name)
                } else {
                    format!("Key {} not in user session {}", key, self.name)
                }
            }))
        }
    }

    fn __setitem__(&mut self, key: &str, value: &PyAny) -> PyResult<()> {
        self._store(key, value)
    }

    fn __len__(&self) -> PyResult<usize> {
        let mut s = origen::sessions();
        let session = s.get_mut_session(self.path.clone(), self.app_session)?;
        Ok(session.len())
    }
}

#[pyclass]
pub struct SessionStoreIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for SessionStoreIter {
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
impl pyo3::class::iter::PyIterProtocol for SessionStore {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<SessionStoreIter> {
        Ok(SessionStoreIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}

impl SessionStore {
    fn new(path: PathBuf, is_app_session: bool, name: String) -> Self {
        Self {
            path: path,
            app_session: is_app_session,
            name: name,
        }
    }

    fn _store(&self, key: &str, value: &PyAny) -> PyResult<()> {
        let mut s = origen::sessions();
        let session = s.get_mut_session(self.path.clone(), self.app_session)?;

        if value.is_none() {
            session.delete(key)?;
        } else {
            let data = extract_as_metadata(value)?;
            session.store(key.to_string(), data)?;
        }
        Ok(())
    }

    pub fn with_origen_session<F>(&self, mut f: F) -> origen::Result<()>
    where
        F: FnMut(&mut origen::utility::session_store::SessionStore) -> origen::Result<()>,
    {
        let mut s = origen::sessions();
        f(s.get_mut_session(self.path.clone(), self.app_session)?)?;
        Ok(())
    }
}
