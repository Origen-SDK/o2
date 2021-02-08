use pyo3::{wrap_pyfunction};
use pyo3::prelude::*;
use pyo3::types::{PyDict, IntoPyDict};
use super::utility::session_store::{SessionStore, user_session};
use std::collections::HashMap;
use pyo3::class::mapping::PyMappingProtocol;
use super::utility::metadata::{metadata_to_pyobj, extract_as_metadata};

const DATA_FIELDS: [&str; 5] = ["email", "first_name", "last_name", "display_name", "username"];

#[pymodule]
fn users(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(current_user))?;
    m.add("users", wrap_pyfunction!(users_cls)(py))?;
    m.add_class::<Users>()?;
    m.add_class::<User>()?;
    Ok(())
}

#[pyfunction]
/// Retrieves the current user
fn current_user() -> PyResult<User> {
    User::new(&origen::core::user::get_current_id()?)
}

// #[pyfunction]
// fn switch_user() -> PyResult<()> {
//     // ...
// }

#[pyfunction]
fn users_cls() -> PyResult<Users> {
    Ok(Users {})
}

/// To allow `origen.users` to act as a dict-like property without falling out of sync
/// with the backend, this class should remain stateless.
#[pyclass]
pub struct Users {
}

#[pymethods]
impl Users {
    fn current_user(&self) -> PyResult<User> {
        User::new(&origen::core::user::get_current_id()?)
    }

    fn add(&self, id: &str) -> PyResult<User> {
        let mut users = origen::users_mut();
        users.add(id)?;
        User::new(id)
    }

    fn get(&self, id: &str) -> PyResult<Option<User>> {
        let users = origen::users();
        if let Ok(u) = users.user(id) {
            Ok(Some(User::new(u.id())?))
        } else {
            Ok(None)
        }
    }

    fn ids(&self) -> PyResult<Vec<String>> {
        let users = origen::users();
        Ok(users.users().keys().map( |id| id.to_string()).collect())
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        self.ids()
    }

    fn values(&self) -> PyResult<Vec<User>> {
        let users = origen::users();
        let mut retn = vec![];
        for id in users.users().keys() {
            retn.push(User::new(id)?);
        }
        Ok(retn)
    }

    fn items(&self) -> PyResult<Vec<(String, User)>> {
        let users = origen::users();
        let mut retn = vec![];
        for id in users.users().keys() {
            retn.push((id.to_string(), User::new(id)?));
        }
        Ok(retn)
    }

    #[allow(non_snake_case)]
    #[getter]
    fn DATA_FIELDS(&self) -> PyResult<[&str; 5]> {
        Ok(DATA_FIELDS)
    }
}

#[pyproto]
impl PyMappingProtocol for Users {
    fn __getitem__(&self, id: &str) -> PyResult<User> {
        let users = origen::users();
        if let Ok(u) = users.user(id) {
            Ok(User::new(u.id())?)
        } else {
            Err(pyo3::exceptions::KeyError::py_err(format!(
                "Could not find user '{}'. Add a new user with 'origen.users.add(\"{}\")'",
                id,
                id
            )))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let users = origen::users();
        Ok(users.users().len())
    }
}

#[pyclass]
pub struct UsersIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for UsersIter {
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
impl pyo3::class::iter::PyIterProtocol for Users {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<UsersIter> {
        Ok(UsersIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}

#[pyclass]
pub struct UserDataset {
    user_id: String,
    dataset: String
}

#[pymethods]
impl UserDataset {
    #[getter]
    fn get_username(&self) -> PyResult<Option<String>> {
        Ok(origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| Ok(d.username.clone()))?)
    }

    #[setter]
    fn set_username(&self, username: Option<String>) -> PyResult<()> {
        Ok(origen::user::with_user_dataset_mut(Some(&self.user_id), &self.dataset, |d| {d.username = username.clone(); Ok(()) })?)
    }

    #[getter]
    fn get_email(&self) -> PyResult<Option<String>> {
        Ok(origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| Ok(d.email.clone()))?)
    }

    #[setter]
    fn set_email(&self, email: Option<String>) -> PyResult<()> {
        Ok(origen::user::with_user_dataset_mut(Some(&self.user_id), &self.dataset, |d| {d.email = email.clone(); Ok(()) })?)
    }

    #[getter]
    fn get_first_name(&self) -> PyResult<Option<String>> {
        Ok(origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| Ok(d.first_name.clone()))?)
    }

    #[setter]
    fn set_first_name(&self, first_name: Option<String>) -> PyResult<()> {
        Ok(origen::user::with_user_dataset_mut(Some(&self.user_id), &self.dataset, |d| {d.first_name = first_name.clone(); Ok(()) })?)
    }

    #[getter]
    fn get_last_name(&self) -> PyResult<Option<String>> {
        Ok(origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| Ok(d.last_name.clone()))?)
    }

    #[setter]
    fn set_last_name(&self, last_name: Option<String>) -> PyResult<()> {
        Ok(origen::user::with_user_dataset_mut(Some(&self.user_id), &self.dataset, |d| {d.last_name = last_name.clone(); Ok(()) })?)
    }

    #[getter]
    fn get_display_name(&self) -> PyResult<String> {
        Ok(origen::with_user(&self.user_id, |u| u.display_name_for(Some(&self.dataset)))?)
    }

    #[allow(non_snake_case)]
    #[getter]
    fn get___display_name__(&self) -> PyResult<Option<String>> {
        Ok(origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| Ok(d.display_name.clone()))?)
    }

    #[setter]
    fn set_display_name(&self, display_name: Option<String>) -> PyResult<()> {
        Ok(origen::user::with_user_dataset_mut(Some(&self.user_id), &self.dataset, |d| {d.display_name = display_name.clone(); Ok(()) })?)
    }

    /// Gets the password for this dataset
    #[getter]
    fn get_password(&self) -> PyResult<String> {
        Ok(origen::with_user(&self.user_id, |u| u.password(Some(&self.dataset), false, None))?)
    }

    #[setter]
    fn set_password(&self, password: Option<String>) -> PyResult<()> {
        Ok(origen::with_user(&self.user_id, |u| { u.set_password(password.clone(), Some(&self.dataset), None) })?)
    }

    fn clear_cached_password(&self) -> PyResult<()> {
        Ok(origen::with_user(&self.user_id, |u| u.clear_cached_password(Some(&self.dataset)))?)
    }

    #[getter]
    fn data_store(&self) -> PyResult<DataStore> {
        Ok(DataStore::new(&self.user_id, &self.dataset))
    }

    fn populate(&self) -> PyResult<()> {
        origen::with_user(&self.user_id, |u| u.populate(
            &self.dataset,
            origen::user::lookup_dataset_config(&self.user_id)?,
            false
        ))?;
        Ok(())
    }
}

impl UserDataset {
    pub fn new(user_id: &str, dataset_name: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            dataset: dataset_name.to_string()
        }
    }
}

#[pyclass(subclass)]
pub struct User {
    user_id: String
}

impl User {
    fn new(id: &str) -> PyResult<Self> {
        Ok(Self {
            user_id: id.to_string()
        })
    }
}

#[pymethods]
impl User {

    #[getter]
    fn get_id(&self) -> PyResult<String> {
        Ok(origen::with_user(&self.user_id, |u| Ok(u.id().to_string()))?)
    }

    #[getter]
    fn get_username(&self) -> PyResult<String> {
        Ok(origen::with_user(&self.user_id, |u| u.username())?)
    }

    #[setter]
    fn set_username(&self, username: Option<String>) -> PyResult<()> {
        origen::with_user(&self.user_id, |u| u.set_username(username.clone()))?;
        Ok(())
    }

    #[getter]
    fn get_email(&self) -> PyResult<Option<String>> {
        Ok(origen::with_user(&self.user_id, |u| Ok(u.email()))?)
    }

    #[setter]
    fn set_email(&self, email: Option<String>) -> PyResult<()> {
        origen::with_user(&self.user_id, |u| u.set_email(email.clone()))?;
        Ok(())
    }

    #[getter]
    fn get_first_name(&self) -> PyResult<Option<String>> {
        Ok(origen::with_user(&self.user_id, |u| u.first_name())?)
    }

    #[setter]
    fn set_first_name(&self, first_name: Option<String>) -> PyResult<()> {
        origen::with_user(&self.user_id, |u| u.set_first_name(first_name.clone()))?;
        Ok(())
    }

    #[getter]
    fn get_last_name(&self) -> PyResult<Option<String>> {
        Ok(origen::with_user(&self.user_id, |u| u.last_name())?)
    }

    #[setter]
    fn last_name(&self, last_name: Option<String>) -> PyResult<()> {
        origen::with_user(&self.user_id, |u| u.set_last_name(last_name.clone()))?;
        Ok(())
    }

    #[getter]
    fn get_display_name(&self) -> PyResult<String> {
        Ok(origen::with_user(&self.user_id, |u| u.display_name())?)
    }

    #[setter]
    fn display_name(&self, display_name: Option<String>) -> PyResult<()> {
        origen::with_user(&self.user_id, |u| u.set_display_name(display_name.clone()))?;
        Ok(())
    }

    /// Gets the password for the default dataset
    #[getter]
    fn password(&self) -> PyResult<String> {
        Ok(origen::with_user(&self.user_id, |u| u.password(None, true, None))?)
    }

    #[setter]
    fn set_password(&self, password: Option<String>) -> PyResult<()> {
        Ok(origen::with_user(&self.user_id, |u| { u.set_password(password.clone(), None, None) })?)
    }

    // Note that we can't get a optional None value (as least as far as I know...)
    // Passing in None on the Python side makes this look like the argument given, so can't
    // get a nested None.
    #[args(kwargs="**")]
    fn password_for(&self, reason: &str, kwargs: Option<&PyDict>) -> PyResult<String> {
        let default: Option<Option<&str>>;
        if let Some(opts) = kwargs {
            // Important, supporting None to mean default data key here
            // So, need to check if the key exists, instead of a "if let Some(...)" as
            // we can't distinguish between a key that wasn't given, e.g. Option::None, and a key that
            // was given but was set to None, e.g. Option::Some(None)
            if opts.contains("default")? {
                let d = opts.get_item("default").unwrap();
                if d.is_none() {
                    default = Some(None);
                } else {
                    default = Some(Some(d.extract::<&str>()?));
                }
            } else {
                default = None;
            }
        } else {
            default = None;
        }
        Ok(origen::with_user(&self.user_id, |u| { u.password(Some(reason), true, default) })?)
    }

    fn dataset_for(&self, reason: &str) -> PyResult<Option<String>> {
        Ok(origen::with_user(&self.user_id, |u| { 
            if let Some(d) = u.dataset_for(reason) {
                Ok(Some(d.to_string()))
            } else {
                Ok(None)
            }
        })?)
    }

    /// Clears all cached passwords for all datasets
    fn clear_cached_passwords(&self) -> PyResult<()> {
        Ok(origen::with_user(&self.user_id, |u| u.clear_cached_passwords())?)
    }

    /// Clears the cached password only for the default dataset
    fn clear_cache_password(&self) -> PyResult<()> {
        Ok(origen::with_user(&self.user_id, |u| u.clear_cached_password(None))?)
    }

    #[getter]
    fn authenticated(&self) -> PyResult<bool> {
        Ok(origen::with_user(&self.user_id, |u| Ok(u.authenticated()))?)
    }

    // fn switch_to() -> PyResult<()> {
    //     // ...
    // }

    #[getter]
    fn dataset(&self) -> PyResult<String> {
        Ok(origen::with_user(&self.user_id, |u| Ok(u.dataset().to_string()))?)
    }

    #[getter]
    fn datasets(&self) -> PyResult<HashMap<String, UserDataset>> {
        let mut retn = HashMap::new();
        origen::with_user(&self.user_id, |u| {
            for n in u.datasets().keys() {
                retn.insert(n.to_string(), UserDataset::new(&self.user_id, n));
            }
            Ok(())
        })?;
        Ok(retn)
    }

    #[getter]
    fn data_store(&self) -> PyResult<DataStore> {
        Ok(origen::with_user(&self.user_id, |u| Ok(DataStore::new(&self.user_id, &u.dataset())))?)
    }

    #[getter]
    fn home_dir(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(crate::pypath!(py, origen::with_user(&self.user_id, |u| u.home_dir_string())?))
    }

    #[getter]
    fn session(&self) -> PyResult<SessionStore> {
        user_session(None)
    }
}

#[pyclass]
struct DataStore {
    user_id: String,
    dataset: String
}

impl DataStore {
    pub fn new(user_id: &str, dataset: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            dataset: dataset.to_string()
        }
    }
}

#[pymethods]
impl DataStore {
    fn get(&self, key: &str) -> PyResult<Option<PyObject>> {
        Ok(origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| {
            if let Some(o) = d.other.get(key) {
                // Ok(Some(o.clone()))
                Ok(metadata_to_pyobj(Some(o.clone()), Some(key))?)
            } else {
                Ok(None)
            }
        })?)
    }

    pub fn keys(&self) -> PyResult<Vec<String>> {
        Ok(origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| {
            Ok(d.other.keys().map (|k| k.to_string()).collect::<Vec<String>>())
        })?)
    }

    fn values(&self) -> PyResult<Vec<Option<PyObject>>> {
        Ok(origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| {
            let mut retn: Vec<Option<PyObject>> = vec![];
            for (key, obj) in d.other.iter() {
                retn.push(metadata_to_pyobj(Some(obj.clone()), Some(key))?);
            }
            Ok(retn)
        })?)
    }

    fn items(&self) -> PyResult<Vec<(String, Option<PyObject>)>> {
        Ok(origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| {
            let mut retn: Vec<(String, Option<PyObject>)> = vec![];
            for (key, obj) in d.other.iter() {
                retn.push((key.to_string(), metadata_to_pyobj(Some(obj.clone()), Some(key))?));
            }
            Ok(retn)
        })?)
    }
}

#[pyproto]
impl PyMappingProtocol for DataStore {
    fn __getitem__(&self, key: &str) -> PyResult<Option<PyObject>> {
        let obj = origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| {
            if let Some(o) = d.other.get(key) {
                Ok(Some(o.clone()))
            } else {
                Ok(None)
            }
        })?;
        if let Some(o) = obj {
            metadata_to_pyobj(Some(o), Some(key))
        } else {
            Err(pyo3::exceptions::KeyError::py_err(format!(
                "No data added with key '{}' in dataset '{}' for user '{}'",
                key,
                self.dataset,
                self.user_id
            )))
        }
    }

    fn __setitem__(&mut self, key: &str, value: &PyAny) -> PyResult<()> {
        origen::user::with_user_dataset_mut(Some(&self.user_id), &self.dataset, |d| {
            Ok(d.other.insert(key.to_string(), extract_as_metadata(value)?))
        })?;
        Ok(())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(origen::user::with_user_dataset(Some(&self.user_id), &self.dataset, |d| Ok(d.other.len()))?)
    }
}

#[pyclass]
pub struct DataStoreIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for DataStoreIter {
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
impl pyo3::class::iter::PyIterProtocol for DataStore {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<UsersIter> {
        Ok(UsersIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}