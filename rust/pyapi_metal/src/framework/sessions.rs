use pyo3::prelude::*;
use pyo3::class::mapping::PyMappingProtocol;
use pyo3::class::basic::CompareOp;
use origen_metal::framework::sessions as om_ss;
use om_ss::SessionStore as OmSS;
use om_ss::SessionGroup as OmSG;
use om_ss::Sessions as OmS;
use origen_metal::Result as OMResult;
use crate::_helpers::pypath_as_pathbuf;
use crate::_helpers::typed_value::{extract_as_typed_value, typed_value_to_pyobj};
use std::collections::HashMap;
use origen_metal::sessions as om_sessions;
use std::sync::MutexGuard;
use super::FilePermissions;

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "sessions")?;
    subm.add_class::<Sessions>()?;
    subm.add_class::<SessionGroup>()?;
    subm.add_class::<SessionStore>()?;
    subm.add_wrapped(wrap_pyfunction!(sessions))?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyfunction]
pub fn sessions(_py: Python) -> PyResult<Sessions> {
    Ok(Sessions {})
}

#[pyclass(subclass)]
pub struct Sessions {}

#[pymethods]
impl Sessions {
    #[getter]
    pub fn get_groups(&self) -> PyResult<HashMap<String, SessionGroup>> {
        let om = om_sessions();
        let mut retn = HashMap::new();
        for (n, g) in om.groups() {
            retn.insert(n.to_string(), SessionGroup::from_metal(g)?);
        }
        Ok(retn)
    }
    
    pub fn group(&self, name: &str) -> PyResult<Option<SessionGroup>> {
        let om = om_sessions();
        Ok(match om.get_group(name) {
            Some(g) => Some(SessionGroup::from_metal(g)?),
            None => None
        })
    }
    
    pub fn add_group(&self, name: &str, root: &PyAny, file_permissions: Option<&PyAny>) -> PyResult<SessionGroup> {
        let mut om = om_sessions();
        om.add_group(name, pypath_as_pathbuf(root)?.as_path(), FilePermissions::to_metal_optional(file_permissions)?)?;
        Ok(SessionGroup::new(name))
    }

    pub fn delete_group(&self, name: &str) -> PyResult<bool> {
        let mut om = om_sessions();
        Ok(om.delete_group(name)?)
    }

    #[getter]
    pub fn get_standalones(&self) -> PyResult<HashMap<String, SessionStore>> {
        let om = om_sessions();
        let mut retn = HashMap::new();
        for (n, _) in om.standalones() {
            retn.insert(n.to_string(), SessionStore::new(n.to_string(), None));
        }
        Ok(retn)
    }
    
    pub fn standalone(&self, name: &str) -> PyResult<Option<SessionStore>> {
        let om = om_sessions();
        Ok(match om.get_standalone(name) {
            Some(_) => Some(SessionStore::new(name.to_string(), None)),
            None => None
        })
    }

    pub fn add_standalone(&self, name: &str, root: &PyAny, file_permissions: Option<&PyAny>) -> PyResult<SessionStore> {
        let mut om = om_sessions();
        om.add_standalone(name, pypath_as_pathbuf(root)?.as_path(), FilePermissions::to_metal_optional(file_permissions)?)?;
        Ok(SessionStore::new(name.to_string(), None))
    }

    pub fn delete_standalone(&self, name: &str) -> PyResult<bool> {
        let mut om = om_sessions();
        Ok(om.delete_standalone(name)?)
    }

    /// Refreshes all standalones and groups
    fn refresh(&self) -> PyResult<()> {
        let om = om_sessions();
        om.refresh()?;
        Ok(())
    }
}

#[pyclass(subclass)]
pub struct SessionGroup {
    name: String,
}

impl SessionGroup {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string()
        }
    }

    pub fn from_metal(grp: &OmSG) -> PyResult<Self> {
        Ok(Self::new(&grp.name()?.to_string()))
    }

    pub fn with_om_sg<F, T>(&self, sessions: Option<MutexGuard<OmS>>, mut f: F) -> OMResult<T>
    where
        F: FnMut(&OmSG) -> OMResult<T>,
    {
        match sessions {
            Some(s) => {
                Ok(f(s.require_group(&self.name)?)?)
            },
            None => {
                let s = om_sessions();
                Ok(f(s.require_group(&self.name)?)?)
            }
        }
    }

    pub fn with_mut_om_sg<F, T>(&self, sessions: Option<MutexGuard<OmS>>, mut f: F) -> OMResult<T>
    where
        F: FnMut(&mut OmSG) -> OMResult<T>,
    {
        match sessions {
            Some(mut s) => {
                Ok(f(s.require_mut_group(&self.name)?)?)
            },
            None => {
                let mut s = om_sessions();
                Ok(f(s.require_mut_group(&self.name)?)?)
            }
        }
    }
}

#[pymethods]
impl SessionGroup {
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        // Attempt to grab the OmSG to ensure it exists (ensure not stale)
        self.with_om_sg(None, |_| { Ok(()) })?;
        Ok(self.name.clone())
    }

    #[getter]
    fn get_path(&self) -> PyResult<PyObject> {
        Ok(Python::with_gil( |py| {
            self.with_om_sg(None, |g| {
                Ok(crate::pypath!(py, format!("{}", g.path().display())))
            })
        })?)
    }

    #[getter]
    fn get_sessions(&self) -> PyResult<HashMap<String, SessionStore>> {
        Ok(self.items()?.into_iter().collect())
    }

    fn get(&self, session_name: &str) -> PyResult<Option<SessionStore>> {
        Ok(self.with_om_sg(None, |sg| {
            Ok(match sg.get(session_name) {
                Some(_) => Some(SessionStore::new(session_name.to_string(), Some(self.name.to_string()))),
                None => None
            })
        })?)
    }

    fn add_session(&self, name: &str) -> PyResult<SessionStore> {
        self.with_mut_om_sg(None, |sg| {
            sg.add_session(name)?;
            Ok(())
        })?;
        Ok(SessionStore::new(name.to_string(), Some(self.name.to_string())))
    }

    fn delete_session(&self, name: &str) -> PyResult<bool> {
        Ok(self.with_mut_om_sg(None, |sg| {
            sg.delete_session(name)
        })?)
    }

    fn refresh(&self) -> PyResult<()> {
        Ok(self.with_mut_om_sg(None, |sg| {
            sg.refresh()?;
            Ok(())
        })?)
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self.with_om_sg(None, |g| {
            Ok(g.sessions().keys().map(|k| k.to_string()).collect())
        })?)
    }

    fn values(&self) -> PyResult<Vec<SessionStore>> {
        let mut retn: Vec<SessionStore> = vec![];
        self.with_om_sg(None, |g| {
            for (n, _) in g.sessions().iter() {
                retn.push(SessionStore::new(n.to_string(), Some(self.name.to_string())));
            }
            Ok(())
        })?;
        Ok(retn)
    }

    fn items(&self) -> PyResult<Vec<(String, SessionStore)>> {
        let mut retn: Vec<(String, SessionStore)> = vec![];
        self.with_om_sg(None, |sg| {
            for (n, _) in sg.sessions() {
                retn.push((
                    n.to_string(),
                    SessionStore::new(n.to_string(), Some(self.name.to_string()))
                ));
            }
            Ok(())
        })?;
        Ok(retn)
    }

    #[getter]
    fn permissions(&self) -> PyResult<FilePermissions> {
        Ok(self.with_om_sg(None, |sg| {
            Ok(FilePermissions::from_metal(sg.file_permissions()))
        })?)
    }
}

#[pyproto]
impl pyo3::class::basic::PyObjectProtocol for SessionGroup {
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        let other_grp = match other.extract::<PyRef<Self>>() {
            Ok(grp) => grp,
            Err(_) => return Python::with_gil( |py| Ok(false.to_object(py)))
        };

        let s = om_sessions();
        let om_grp = s.require_group(&self.name)?;
        let om_other = s.require_group(&other_grp.name)?;
        let result = om_grp == om_other;

        Python::with_gil( |py| {
            match op {
                CompareOp::Eq => Ok(result.to_object(py)),
                CompareOp::Ne => Ok((!result).to_object(py)),
                _ => Ok(py.NotImplemented()),
            }
        })
    }
}

#[pyproto]
impl PyMappingProtocol for SessionGroup {
    fn __getitem__(&self, key: &str) -> PyResult<SessionStore> {
        if let Some(s) = self.get(key)? {
            Ok(s)
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err({
                self.with_om_sg(None, |sg| {
                    Ok(format!("Session '{}' has not been added to session group '{}' (path: '{}')", key, self.name, sg.path().display()))
                })?
            }))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.with_om_sg(None, |g| {
            Ok(g.sessions().len())
        })?)
    }
}

#[pyclass]
pub struct SessionGroupIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for SessionGroupIter {
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
impl pyo3::class::iter::PyIterProtocol for SessionGroup {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<SessionGroupIter> {
        Ok(SessionGroupIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}

#[pyclass(subclass)]
pub struct SessionStore {
    name: String,
    group: Option<String>,
}

#[pymethods]
impl SessionStore {
    fn refresh(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        slf.with_om_ss(|ss| {
            ss.refresh()
        })?;
        Ok(slf.into())
    }

    #[getter]
    fn path(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let p = self.with_om_ss(|ss| {
                Ok(format!("{}", ss.path().display()))
            })?;
            Ok(crate::pypath!(py, p))
        })
    }

    #[getter]
    fn name(&self) -> PyResult<String> {
        Ok(self.with_om_ss(|ss| {
            Ok(ss.name()?)
        })?)
    }

    // TODO
    // fn permissions(&self) -> Result<String> {
    //     // ...
    // }

    fn get(&self, key: &str) -> PyResult<Option<PyObject>> {
        Ok(self.with_om_ss(|ss| {
            Ok(typed_value_to_pyobj(ss.retrieve(key)?, Some(key))?)
        })?)
    }

    fn delete(&self, key: &str) -> PyResult<Option<PyObject>> {
        Ok(self.with_om_ss(|ss| {
            Ok(typed_value_to_pyobj(ss.delete(key)?, Some(key))?)
        })?)
    }

    fn store(slf: PyRef<Self>, key: &str, value: &PyAny) -> PyResult<Py<Self>> {
        slf._store(key, value)?;
        Ok(slf.into())
    }

    fn store_serialized(slf: PyRef<Self>, key: &str, value: &[u8]) -> PyResult<Py<Self>> {
        slf.with_om_ss(|ss| {
            ss.store_serialized(
                key.to_string(),
                value,
                Some("Python-Frontend".to_string()),
                None,
            )
        })?;
        Ok(slf.into())
    }

    fn remove_file(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        slf.with_om_ss(|ss| {
            ss.remove_file()
        })?;
        Ok(slf.into())
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self.with_om_ss(|ss| {
            Ok(ss.keys()?.iter().map(|k| k.to_string()).collect())
        })?)
    }

    fn values(&self) -> PyResult<Vec<Option<PyObject>>> {
        let mut retn: Vec<Option<PyObject>> = vec![];
        self.with_om_ss(|ss| {
            for (k, v) in ss.data()?.iter() {
                retn.push(typed_value_to_pyobj(Some(v.clone()), Some(k))?);
            }
            Ok(())
        })?;
        Ok(retn)
    }

    fn items(&self) -> PyResult<Vec<(String, Option<PyObject>)>> {
        let mut retn: Vec<(String, Option<PyObject>)> = vec![];
        self.with_om_ss(|ss| {
            for (k, v) in ss.data()?.iter() {
                retn.push((k.to_string(), typed_value_to_pyobj(Some(v.clone()), Some(k))?));
            }
            Ok(())
        })?;
        Ok(retn)
    }
}


#[pyproto]
impl pyo3::class::basic::PyObjectProtocol for SessionStore {
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        let other_s = match other.extract::<PyRef<Self>>() {
            Ok(ss) => ss,
            Err(_) => return Python::with_gil( |py| Ok(false.to_object(py)))
        };

        let s = om_sessions();
        let mut result: bool = false;
        self.with_om(&s, |ss| {
            other_s.with_om(&s, |other_ss| {
                result = ss == other_ss;
                Ok(())
            })
        })?;

        Python::with_gil( |py| {
            match op {
                CompareOp::Eq => Ok(result.to_object(py)),
                CompareOp::Ne => Ok((!result).to_object(py)),
                _ => Ok(py.NotImplemented()),
            }
        })
    }
}

#[pyproto]
impl PyMappingProtocol for SessionStore {
    fn __getitem__(&self, key: &str) -> PyResult<PyObject> {
        if let Some(l) = self.get(key)? {
            Ok(l)
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err({
                self.with_om_ss( |ss| {
                    Ok(format!("Key {} not in session {}", key, ss.path().display()))
                })?
            }))
        }
    }

    fn __setitem__(&mut self, key: &str, value: &PyAny) -> PyResult<()> {
        self._store(key, value)
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.with_om_ss(|ss| {
            Ok(ss.len()?)
        })?)
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
    pub fn new(name: String, group: Option<String>) -> Self {
        Self {
            name: name,
            group: group
        }
    }

    pub fn from_metal(ss: &OmSS) -> PyResult<Self> {
        Ok(Self {
            name: ss.name()?,
            group: ss.group().clone()
        })
    }

    fn _store(&self, key: &str, value: &PyAny) -> PyResult<()> {
        self.with_om_ss(|ss| {
            if value.is_none() {
                ss.delete(key)?;
            } else {
                let data = extract_as_typed_value(value)?;
                ss.store(key.to_string(), data)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn with_om<F, T>(&self, s: &MutexGuard<OmS>, mut f: F) -> OMResult<T>
    where
        F: FnMut(& OmSS) -> OMResult<T>,
    {
        if let Some(grp) = self.group.as_ref() {
            let g = s.require_group(grp)?;
            Ok(f(g.require(&self.name)?)?)
        } else {
            Ok(f(s.require_standalone(&self.name)?)?)
        }
    }

    pub fn with_om_ss<F, T>(&self, mut f: F) -> OMResult<T>
    where
        F: FnMut(&mut OmSS) -> OMResult<T>,
    {
        let mut s = om_sessions();
        if let Some(grp) = self.group.as_ref() {
            match s.require_mut_group(grp) {
                Ok(g) => Ok(f(g.require_mut(&self.name)?)?),
                Err(_) => origen_metal::bail!(&format!(
                    "Error encountered retrieving session '{}': Error retrieving group: '{}'",
                    self.name,
                    grp
                ))
            }
        } else {
            Ok(f(s.require_mut_standalone(&self.name)?)?)
        }
    }
}
