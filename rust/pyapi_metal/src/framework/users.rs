use crate::_helpers::{map_to_pydict, typed_value, with_new_pydict};
use crate::framework::PyOutcome;
use crate::{key_error, pypath, runtime_error, type_error};
use origen_metal as om;
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
use crate::frontend::{LOOKUP_HOME_DIR_FUNC_KEY};

use super::file_permissions::FilePermissions;
use std::path::PathBuf;

use crate::prelude::sessions::*;
use crate::_helpers::contextlib::wrap_instance_method;

// TODO add a users prelude?
use om::framework::users::DatasetConfig as OMDatasetConfig;
use om::framework::users::PopulateUserReturn as OmPopulateUserReturn;
use om::framework::users::PopulateUsersReturn as OmPopulateUsersReturn;
use om::framework::users::SessionConfig as OMUserSessionConfig;
use om::framework::users::User as OMUser;
use om::Result as OMResult;

const DATA_FIELDS: [&str; 5] = [
    "email",
    "first_name",
    "last_name",
    "display_name",
    "username",
];

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "users")?;
    subm.add_class::<Users>()?;
    subm.add_class::<User>()?;
    subm.add_class::<UserDataset>()?;
    subm.add_class::<UserDatasetConfig>()?;
    subm.add_class::<PopulateUsersReturn>()?;
    subm.add_class::<PopulateUserReturn>()?;
    subm.add_class::<UsersSessionConfig>()?;
    subm.add_class::<UserSessionConfig>()?;
    subm.add_wrapped(wrap_pyfunction!(users))?;
    m.add_submodule(subm)?;

    let users_class = subm.getattr("Users")?;
    users_class.setattr("current_user_as", wrap_instance_method(py, "current_user_as", Some(vec!("new_current")), None)?)?;
    Ok(())
}

#[pyfunction]
pub fn users() -> PyResult<Users> {
    Ok(Users {})
}

/// To allow `origen.users` to act as a dict-like property without falling out of sync
/// with the backend, this class should remain stateless.
#[pyclass]
pub struct Users {}

#[pymethods]
impl Users {
    #[getter]
    pub fn current_user(&self) -> PyResult<Option<User>> {
        Ok(match om::get_current_user_id()? {
            Some(id) => Some(User::new(&id)?),
            None => None,
        })
    }

    #[getter]
    pub fn current(&self) -> PyResult<Option<User>> {
        self.current_user()
    }

    #[setter(current_user)]
    pub fn current_user_setter(&self, u: &PyAny) -> PyResult<()> {
        self.set_current_user(u)?;
        Ok(())
    }

    pub fn set_current_user(&self, u: &PyAny) -> PyResult<bool> {
        Ok(if u.is_none() {
            // TODO needs test
            om::clear_current_user()?
        } else if let Ok(user) = u.extract::<PyRef<User>>() {
            om::set_current_user(&user.user_id)?
        } else if let Ok(id) = u.extract::<&str>() {
            om::set_current_user(id)?
        } else {
            return type_error!(format!(
                "Cannot resolve user from type '{}'",
                u.get_type().name()?
            ));
        })
    }

    pub fn clear_current_user(&self) -> PyResult<bool> {
        let mut users = om::users_mut();
        Ok(users.clear_current_user()?)
    }

    #[getter]
    pub fn initial_user(&self) -> PyResult<Option<User>> {
        Ok(match om::get_initial_user_id()? {
            Some(id) => {
                if om::users().users().contains_key(&id) {
                    Some(User::new(&id)?)
                } else {
                    return runtime_error!(format!(
                        "Initial user '{}' is no longer an active user!",
                        id
                    ));
                }
            }
            None => None,
        })
    }

    #[getter]
    pub fn initial(&self) -> PyResult<Option<User>> {
        self.initial_user()
    }

    pub fn add(&self, id: &str, auto_populate: Option<bool>) -> PyResult<User> {
        om::add_user(id, auto_populate)?;
        User::new(id)
    }

    fn remove(&self, id: &str) -> PyResult<bool> {
        let mut users = om::users_mut();
        Ok(users.remove(id)?)
    }

    pub fn get(&self, id: &str) -> PyResult<Option<User>> {
        let users = om::users();
        if let Ok(u) = users.user(id) {
            Ok(Some(User::new(u.id())?))
        } else {
            Ok(None)
        }
    }

    #[getter]
    fn ids(&self) -> PyResult<Vec<String>> {
        let users = om::users();
        Ok(users.users().keys().map(|id| id.to_string()).collect())
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        self.ids()
    }

    fn values(&self) -> PyResult<Vec<User>> {
        let users = om::users();
        let mut retn = vec![];
        for id in users.users().keys() {
            retn.push(User::new(id)?);
        }
        Ok(retn)
    }

    fn items(&self) -> PyResult<Vec<(String, User)>> {
        let users = om::users();
        let mut retn = vec![];
        for id in users.users().keys() {
            retn.push((id.to_string(), User::new(id)?));
        }
        Ok(retn)
    }

    #[getter]
    fn users<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let users = om::users();
        let retn = PyDict::new(py);
        for id in users.users().keys() {
            retn.set_item(id, Py::new(py, User::new(id)?)?)?;
        }
        Ok(retn)
    }

    // TODO
    // TEST_NEEDED
    // pub fn users_for(&self, roles: Option<&PyAny>, motives: Option<&PyAny>) -> PyResult<Vec<String>> {
    //     let users = om::users();
    //     py_into_vec!(roles)
    //     users.users_for(py_into_vec!(roles), py_into_vec!(roles))
    // }

    #[allow(non_snake_case)]
    pub fn __enter__current_user_as<'p>(&self, py: Python<'p>, u: &PyAny) -> PyResult<(Option<PyObject>, Option<String>)> {
        let current_id = om::get_current_user_id()?;
        self.set_current_user(u)?;
        let new_current = match om::get_current_user_id()? {
            Some(id) => Some(Py::new(py, User::new(&id)?)?),
            None => None
        };
        Ok((Some(new_current.to_object(py)), current_id))
    }

    #[allow(non_snake_case)]
    pub fn __exit__current_user_as(&self, _py: Python, _yield_retn: Option<&PyAny>, _yield_context: Option<PyRef<User>>, old_user: &PyAny) -> PyResult<()> {
        self.set_current_user(old_user)?;
        Ok(())
    }

    #[allow(non_snake_case)]
    #[getter]
    fn DATA_FIELDS(&self) -> PyResult<[&str; 5]> {
        Ok(DATA_FIELDS)
    }

    #[args(update_current = "false")]
    pub fn lookup_current_id(&self, update_current: bool) -> PyResult<String> {
        if update_current {
            let r = om::try_lookup_and_set_current_user()?;
            if let Some(user_added) = r.1 {
                if let Some(pop_retn) = user_added{
                    if let Some(error_msg) = pop_retn.log(&r.0)? {
                        return runtime_error!(error_msg);
                    }
                }
            }
            Ok(r.0)
        } else {
            Ok(om::try_lookup_current_user()?)
        }
    }

    fn unload(&self) -> PyResult<()> {
        Ok(om::unload_users(false)?)
    }

    #[getter]
    pub fn get_lookup_current_id_function(&self) -> PyResult<Option<PyObject>> {
        crate::frontend::with_py_frontend(|py, py_fe| {
            Ok(match py_fe._users_.get("lookup_current_id_function") {
                Some(f) => Some(f.to_object(py)),
                None => None,
            })
        })
    }

    #[setter]
    pub fn set_lookup_current_id_function(&self, func: Option<&PyAny>) -> PyResult<()> {
        crate::frontend::with_mut_py_frontend(|py, mut py_fe| {
            match func {
                Some(f) => py_fe
                    ._users_
                    .insert("lookup_current_id_function".to_string(), f.to_object(py)),
                None => py_fe._users_.remove("lookup_current_id_function"),
            };
            Ok(())
        })
    }

    #[getter]
    pub fn get_lookup_home_dir_function(&self) -> PyResult<Option<PyObject>> {
        crate::frontend::with_py_frontend(|py, py_fe| {
            Ok(match py_fe._users_.get(*LOOKUP_HOME_DIR_FUNC_KEY) {
                Some(f) => Some(f.to_object(py)),
                None => None,
            })
        })
    }

    #[setter]
    pub fn set_lookup_home_dir_function(&self, func: Option<&PyAny>) -> PyResult<()> {
        crate::frontend::with_mut_py_frontend(|py, mut py_fe| {
            match func {
                Some(f) => py_fe._users_.insert(LOOKUP_HOME_DIR_FUNC_KEY.to_string(), f.to_object(py)),
                None => py_fe._users_.remove(*LOOKUP_HOME_DIR_FUNC_KEY),
            };
            Ok(())
        })
    }

    #[getter]
    fn get_datakeys(&self) -> PyResult<Vec<String>> {
        let users = om::users();
        Ok(users
            .default_datakeys()
            .iter()
            .map(|dk| dk.to_string())
            .collect())
    }

    #[getter]
    fn get_datasets<'a>(&self, py: Python<'a>) -> PyResult<&'a PyDict> {
        let users = om::users();
        let retn = PyDict::new(py);
        for (n, config) in users.default_datasets().iter() {
            retn.set_item(
                n.to_string(),
                UserDatasetConfig::new_py(py, config.to_owned())?,
            )?;
        }
        Ok(retn)
    }

    fn dataset(&self, ds: &str) -> PyResult<Option<UserDatasetConfig>> {
        let users = om::users();
        if let Some(c) = users.default_datasets().get(ds) {
            Ok(Some(c.into()))
        } else {
            Ok(None)
        }
    }

    #[args(config = "None")]
    fn register_dataset(&self, name: &str, config: Option<&PyAny>) -> PyResult<()> {
        let mut users = om::users_mut();
        users.register_default_dataset(name, UserDatasetConfig::into_om(config)?)?;
        Ok(())
    }

    #[args(config = "None", as_topmost = "true")]
    pub fn add_dataset(
        &self,
        name: &str,
        config: Option<&PyAny>,
        as_topmost: bool,
    ) -> PyResult<()> {
        let mut users = om::users_mut();
        users.add_default_dataset(name, UserDatasetConfig::into_om(config)?, as_topmost)?;
        Ok(())
    }

    #[args(config = "None")]
    pub fn override_default_dataset(&self, name: &str, config: Option<&PyAny>) -> PyResult<()> {
        let mut users = om::users_mut();
        users.override_default_dataset(name, UserDatasetConfig::into_om(config)?)?;
        Ok(())
    }

    #[getter]
    fn get_data_lookup_hierarchy(&self) -> PyResult<Vec<String>> {
        let users = om::users();
        Ok(users
            .default_data_lookup_hierarchy()
            .iter()
            .map(|dk| dk.to_string())
            .collect())
    }

    #[setter]
    fn data_lookup_hierarchy(&self, new_hierarchy: Vec<String>) -> PyResult<()> {
        self.set_data_lookup_hierarchy(new_hierarchy)
    }

    pub fn set_data_lookup_hierarchy(&self, new_hierarchy: Vec<String>) -> PyResult<()> {
        let mut users = om::users_mut();
        Ok(users.set_default_data_lookup_hierarchy(new_hierarchy)?)
    }

    #[getter]
    fn motives(&self, py: Python) -> PyResult<Py<PyDict>> {
        let users = om::users_mut();
        Ok(map_to_pydict(py, &mut users.motive_mapping().iter())?)
    }

    #[args(replace_existing = "false")]
    pub fn add_motive(
        &self,
        motive: &str,
        dataset: &str,
        replace_existing: bool,
    ) -> PyResult<Option<String>> {
        let mut users = om::users_mut();
        Ok(users.add_motive(motive.to_string(), dataset.to_string(), replace_existing)?)
    }

    fn dataset_for(&self, motive: &str) -> PyResult<Option<String>> {
        let users = om::users_mut();
        Ok(users.dataset_for(motive)?.cloned())
    }

    #[args(
        repopulate = "false",
        continue_on_error = "false",
        stop_on_failure = "false"
    )]
    pub fn populate(
        &self,
        repopulate: bool,
        continue_on_error: bool,
        stop_on_failure: bool,
    ) -> PyResult<PopulateUsersReturn> {
        let users = om::users();
        Ok(PopulateUsersReturn::from_om(users.populate(
            repopulate,
            continue_on_error,
            stop_on_failure,
        )?))
    }

    #[getter]
    pub fn default_auto_populate(&self) -> PyResult<Option<bool>> {
        let users = om::users();
        Ok(*users.default_auto_populate())
    }

    #[setter]
    pub fn set_default_auto_populate(&self, set_to: Option<bool>) -> PyResult<()> {
        let mut users = om::users_mut();
        Ok(users.set_default_auto_populate(set_to))
    }

    #[getter]
    pub fn default_should_validate_passwords(&self) -> PyResult<Option<bool>> {
        let users = om::users();
        Ok(*users.default_should_validate_passwords())
    }

    #[setter]
    pub fn set_default_should_validate_passwords(&self, should_validate_passwords: Option<bool>) -> PyResult<()> {
        let mut users = om::users_mut();
        Ok(users.set_default_should_validate_passwords(should_validate_passwords))
    }

    #[getter]
    pub fn session_config(&self) -> PyResult<UsersSessionConfig> {
        Ok(UsersSessionConfig {})
    }

    #[getter]
    pub fn get_default_roles(&self) -> PyResult<Vec<String>> {
        let users = om::users();
        Ok(users.default_roles()?.to_owned())
    }

    #[setter]
    pub fn set_default_roles(&self, new: &PyAny) -> PyResult<()> {
        let mut users = om::users_mut();
        if new.is_none() {
            users.clear_default_roles()?;
        } else if let Ok(role) = new.extract::<String>() {
            users.set_default_roles(&vec!(role))?;
        } else if let Ok(roles) = new.extract::<Vec<String>>() {
            users.set_default_roles(&roles)?;
        } else {
            return type_error!("Cannot interpret roles as either 'str', 'list of strs', or 'None'.");
        }
        Ok(())
    }

    #[getter]
    pub fn roles(&self) -> PyResult<Vec<String>> {
        let users = om::users();
        Ok(users.roles()?)
    }

    #[getter]
    pub fn by_role(&self) -> PyResult<HashMap<String, Vec<User>>> {
        let users = om::users();
        let mut retn = HashMap::new();
        for (r, ids) in users.users_by_role(None)? {
            retn.insert(r.to_owned(), ids.iter().map(|id| User::new(id)).collect::<PyResult<Vec<User>>>()?);
        }
        Ok(retn)
    }

    #[args(exclusive="false", required="false")]
    pub fn for_role(&self, role: &str, exclusive: bool, required: bool) -> PyResult<Vec<User>> {
        let users = om::users();
        let r = users.users_by_role(Some( &|_u, rn| rn == role ))?;
        if let Some(ids) = r.get(role) {
            if exclusive {
                if ids.len() > 1 {
                    return runtime_error!(format!(
                        "Found multiple users matching exclusive role '{}': {}",
                        role,
                        ids.iter().map(|id| format!("'{}'", id)).collect::<Vec<String>>().join(", ")
                    ));
                }
            }
            Ok(ids.iter().map(|id| User::new(id)).collect::<PyResult<Vec<User>>>()?)
        } else {
            if required {
                runtime_error!(format!(
                    "No users with role '{}' could be found",
                    role
                ))
            } else {
                Ok(vec!())
            }
        }
    }

    #[args(required="false")]
    pub fn for_exclusive_role(&self, role: &str, required: bool) -> PyResult<Option<User>> {
        Ok(self.for_role(role, true, required)?.pop())
    }

    #[setter]
    pub fn set_default_password_cache_option(&self, password_cache_option: Option<&str>) -> PyResult<()> {
        let mut users = om::users_mut();
        Ok(users.set_default_password_cache_option(password_cache_option)?)
    }

    #[getter]
    pub fn get_default_password_cache_option(&self) -> PyResult<Option<String>> {
        let users = om::users_mut();
        Ok(users.default_password_cache_option().as_ref().map_or( None, |p| p.into()))
    }

    fn __getitem__(&self, id: &str) -> PyResult<User> {
        let users = om::users();
        match users.user(id) {
            Ok(u) => Ok(User::new(u.id())?),
            Err(e) => key_error!(e.to_string()),
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let users = om::users();
        Ok(users.users().len())
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<UsersIter> {
        Ok(UsersIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}

impl Users {
    pub fn require_user(&self, id: &str) -> PyResult<User> {
        let users = om::users();
        users.user(id)?;
        Ok(User::new(id)?)
    }
}

#[pyclass]
pub struct UsersIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pymethods]
impl UsersIter {
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

#[pyclass]
pub struct UsersSessionConfig {}

#[pymethods]
impl UsersSessionConfig {
    #[getter]
    pub fn get_root(&self, py: Python) -> PyResult<Option<PyObject>> {
        let users = om::users();
        match users.default_session_config().root.as_ref() {
            Some(r) => Ok(Some(pypath!(py, r.display()))),
            None => Ok(None),
        }
    }

    #[setter]
    pub fn set_root(&self, root: Option<PathBuf>) -> PyResult<()> {
        let mut users = om::users_mut();
        users.default_session_config_mut().root = root;
        Ok(())
    }

    #[getter]
    pub fn get_offset(&self, py: Python) -> PyResult<Option<PyObject>> {
        let users = om::users();
        match users.default_session_config().offset.as_ref() {
            Some(o) => Ok(Some(pypath!(py, o.display()))),
            None => Ok(None),
        }
    }

    #[setter]
    pub fn set_offset(&self, offset: Option<PathBuf>) -> PyResult<()> {
        let mut users = om::users_mut();
        users.default_session_config_mut().set_offset(offset)?;
        Ok(())
    }

    #[getter]
    pub fn get_file_permissions(&self) -> PyResult<FilePermissions> {
        let users = om::users();
        Ok((&users.default_session_config().file_permissions).into())
    }

    #[setter]
    pub fn set_file_permissions(&self, fp: &PyAny) -> PyResult<()> {
        let mut users = om::users_mut();
        users.default_session_config_mut().file_permissions = FilePermissions::to_metal(fp)?;
        Ok(())
    }

    #[getter]
    pub fn get_fp(&self) -> PyResult<FilePermissions> {
        self.get_file_permissions()
    }

    #[setter]
    pub fn set_fp(&self, fp: &PyAny) -> PyResult<()> {
        self.set_file_permissions(fp)
    }

    // TODO ?
    // #[getter]
    // pub fn get_path(&self) -> PyResult<PathBuf> {
    //     let users = om::users();
    //     users.default_session_config().path()
    // }

    // TODO ?
    // pub fn __group_name__(&self) -> PyResult<&str> {
    //     ?
    // }
}

#[pyclass]
pub struct UserDataset {
    user_id: String,
    dataset: String,
}

#[pymethods]
impl UserDataset {
    #[getter]
    fn dataset_name(&self) -> PyResult<String> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.dataset_name.clone()),
        )?)
    }

    #[getter]
    fn get_username(&self) -> PyResult<Option<String>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.username.clone()),
        )?)
    }

    #[setter]
    fn set_username(&self, username: Option<String>) -> PyResult<()> {
        Ok(om::with_user_dataset_mut(
            Some(&self.user_id),
            &self.dataset,
            |d| {
                d.username = username.clone();
                Ok(())
            },
        )?)
    }

    #[getter]
    fn get_email(&self) -> PyResult<Option<String>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.email.clone()),
        )?)
    }

    #[setter]
    fn set_email(&self, email: Option<String>) -> PyResult<()> {
        Ok(om::with_user_dataset_mut(
            Some(&self.user_id),
            &self.dataset,
            |d| {
                d.email = email.clone();
                Ok(())
            },
        )?)
    }

    #[getter]
    fn get_first_name(&self) -> PyResult<Option<String>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.first_name.clone()),
        )?)
    }

    #[setter]
    fn set_first_name(&self, first_name: Option<String>) -> PyResult<()> {
        Ok(om::with_user_dataset_mut(
            Some(&self.user_id),
            &self.dataset,
            |d| {
                d.first_name = first_name.clone();
                Ok(())
            },
        )?)
    }

    #[getter]
    fn get_last_name(&self) -> PyResult<Option<String>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.last_name.clone()),
        )?)
    }

    #[setter]
    fn set_last_name(&self, last_name: Option<String>) -> PyResult<()> {
        Ok(om::with_user_dataset_mut(
            Some(&self.user_id),
            &self.dataset,
            |d| {
                d.last_name = last_name.clone();
                Ok(())
            },
        )?)
    }

    #[getter]
    fn get_display_name(&self) -> PyResult<String> {
        Ok(om::with_user(&self.user_id, |u| {
            u.display_name_for(Some(&self.dataset))
        })?)
    }

    #[allow(non_snake_case)]
    #[getter]
    fn get___display_name__(&self) -> PyResult<Option<String>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.display_name.clone()),
        )?)
    }

    #[setter]
    fn set_display_name(&self, display_name: Option<String>) -> PyResult<()> {
        Ok(om::with_user_dataset_mut(
            Some(&self.user_id),
            &self.dataset,
            |d| {
                d.display_name = display_name.clone();
                Ok(())
            },
        )?)
    }

    #[getter]
    pub fn get_home_dir(&self, py: Python) -> PyResult<Option<PyObject>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| match d.home_dir.as_ref() {
                Some(d) => Ok(Some(pypath!(py, d.display()))),
                None => Ok(None),
            },
        )?)
    }

    #[getter]
    pub fn require_home_dir(&self, py: Python) -> PyResult<PyObject> {
        Ok(om::with_user(&self.user_id, |u| {
            u.with_dataset(&self.dataset, |d| {
                Ok(pypath!(py, d.require_home_dir(u)?.display()))
            })
        })?)
    }

    #[setter(home_dir)]
    pub fn home_dir_setter(&self, hd: Option<PathBuf>) -> PyResult<()> {
        Ok(om::with_user_dataset_mut(
            Some(&self.user_id),
            &self.dataset,
            |d| {
                d.home_dir = hd.clone();
                Ok(())
            },
        )?)
    }

    pub fn set_home_dir(&self, d: Option<PathBuf>) -> PyResult<()> {
        self.home_dir_setter(
            if d.is_some() {
                d
            } else {
                origen_metal::framework::users::user::try_default_home_dir(Some(&self.user_id), Some(&self.dataset))?
            }
        )
    }

    pub fn clear_home_dir(&self) -> PyResult<()> {
        self.home_dir_setter(None)
    }

    /// Gets the password for this dataset
    #[getter]
    fn password(&self) -> PyResult<String> {
        Ok(om::with_user(&self.user_id, |u| {
            u.password(Some(&self.dataset), false, None)
        })?)
    }

    #[getter]
    fn __password__(&self) -> PyResult<Option<String>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.password.as_ref().map( |s| s.to_string())),
        )?)
    }

    #[setter]
    pub fn set_password(&self, password: Option<String>) -> PyResult<()> {
        Ok(om::with_user(&self.user_id, |u| {
            u.set_password(password.clone(), Some(&self.dataset), None)
        })?)
    }

    fn clear_cached_password(&self) -> PyResult<()> {
        Ok(om::with_user(&self.user_id, |u| {
            u.clear_cached_password(Some(&self.dataset))
        })?)
    }

    #[getter]
    pub fn should_validate_password(&self) -> PyResult<bool> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.config().should_validate_password()),
        )?)
    }

    #[setter]
    fn set_should_validate_password(&self, should_validate_password: Option<bool>) -> PyResult<()> {
        Ok(om::with_user_dataset_mut(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.set_should_validate_password(should_validate_password)),
        )?)
    }

    fn validate_password(&self) -> PyResult<PyOutcome> {
        Ok(PyOutcome::from_origen(om::with_user(&self.user_id, |u| {
            u.validate_password(&u.password(Some(&self.dataset), false, None)?, Some(&self.dataset))
        })?.outcome()?.clone()))
    }

    #[getter]
    fn data_store(&self) -> PyResult<DataStore> {
        Ok(DataStore::new(&self.user_id, &self.dataset))
    }

    #[getter]
    fn data(&self) -> PyResult<DataStore> {
        self.data_store()
    }

    #[getter]
    fn other(&self) -> PyResult<DataStore> {
        self.data_store()
    }

    #[args(
        repopulate = "false",
        continue_on_error = "false",
        stop_on_failure = "false"
    )]
    pub fn populate(
        &self,
        repopulate: bool,
        continue_on_error: bool,
        stop_on_failure: bool,
    ) -> PyResult<Option<PyOutcome>> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(
                match u.populate_dataset(
                    &self.dataset,
                    repopulate,
                    continue_on_error,
                    stop_on_failure,
                )? {
                    Some(feat_rtn) => Some(PyOutcome::from_origen(feat_rtn)),
                    None => None,
                },
            )
        })?)
    }

    #[getter]
    fn populated(&self) -> PyResult<bool> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.populated),
        )?)
    }

    #[getter]
    fn populate_attempted(&self) -> PyResult<bool> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.populate_attempted),
        )?)
    }

    #[getter]
    fn populate_succeeded(&self) -> PyResult<Option<bool>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.populate_succeeded()),
        )?)
    }

    #[getter]
    fn populate_failed(&self) -> PyResult<Option<bool>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.populate_failed()),
        )?)
    }

    #[getter]
    fn config(&self) -> PyResult<UserDatasetConfig> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |ds| Ok(ds.config().into()),
        )?)
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
        let ds = match other.extract::<PyRef<Self>>() {
            Ok(d) => d,
            Err(_) => return Ok(false),
        };
        let result = self.user_id == ds.user_id && self.dataset == ds.dataset;

        match op {
            CompareOp::Eq => Ok(result),
            CompareOp::Ne => Ok(!result),
            _ => crate::not_implemented_error!(
                "UserDataset only supports equals and not-equals comparisons"
            ),
        }
    }
}

impl UserDataset {
    pub fn new(user_id: &str, dataset_name: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            dataset: dataset_name.to_string(),
        }
    }

    pub fn user_id(&self) -> &String {
        &self.user_id
    }

    pub fn dataset(&self) -> &String {
        &self.dataset
    }

    pub fn name_from_pyany(any: &PyAny) -> PyResult<String> {
        if let Ok(ds) = any.extract::<PyRef<Self>>() {
            Ok(ds.dataset.to_owned())
        } else if let Ok(n) = any.extract::<String>() {
            Ok(n)
        } else {
            type_error!(&format!(
                "Cannot extract a dataset name from type {}",
                any.get_type().name()?
            ))
        }
    }
}

#[pyclass(subclass)]
pub struct UserDatasetConfig {
    om_config: OMDatasetConfig,
}

impl UserDatasetConfig {
    pub fn into_om(config: Option<&PyAny>) -> PyResult<OMDatasetConfig> {
        Ok(if let Some(c) = config {
            if let Ok(c_dict) = c.extract::<&PyDict>() {
                OMDatasetConfig::new(
                    match c_dict.get_item("category") {
                        Some(v) => Some(v.extract::<String>()?),
                        None => None,
                    },
                    match c_dict.get_item("data_store") {
                        Some(v) => Some(v.extract::<String>()?),
                        None => None,
                    },
                    match c_dict.get_item("auto_populate") {
                        Some(v) => Some(v.extract::<bool>()?),
                        None => None,
                    },
                    match c_dict.get_item("should_validate_password") {
                        Some(v) => Some(v.extract::<bool>()?),
                        None => None,
                    },
                )?
            } else if let Ok(c_obj) = c.extract::<PyRef<Self>>() {
                c_obj.om_config.clone()
            } else {
                return type_error!(format!(
                    "'config' must be either a dict or UserDatasetConfig. Received: '{}'",
                    c.get_type().name()?
                ));
            }
        } else {
            OMDatasetConfig::default()
        })
    }

    pub fn new_py(py: Python, om_config: OMDatasetConfig) -> PyResult<Py<Self>> {
        Py::new(
            py,
            Self {
                om_config: om_config,
            },
        )
    }
}

impl From<&OMDatasetConfig> for UserDatasetConfig {
    fn from(config: &OMDatasetConfig) -> Self {
        Self {
            om_config: config.clone(),
        }
    }
}

impl From<OMDatasetConfig> for UserDatasetConfig {
    fn from(config: OMDatasetConfig) -> Self {
        Self { om_config: config }
    }
}

// TESTS_NEEDED
#[pymethods]
impl UserDatasetConfig {
    // TODO needed?
    // /// The config points to a valid config in a user dataset.
    // /// This method does not yield a DatasetConfig, since it may not actually exists,
    // /// but instead yields a PyDict that guaranteed-to-be-valid keys
    // /// This dict can be passed to other methods or further manipulated
    // #[classmethod]
    // #[args(data_store_source="None", auto_populate="None")]
    // pub fn validated<'a>(_cls: &'a PyType, py: Python<'a>, data_store_source: Option<String>, auto_populate: Option<bool>) -> PyResult<&'a PyDict> {
    //     let dict = PyDict::new(py);
    //     dict.set_item("data_store_source", data_store_source)?;
    //     dict.set_item("auto_populate", auto_populate)?;
    //     Ok(dict)
    // }

    // TODO rename to new?
    #[new]
    #[args(category = "None", data_store = "None", auto_populate = "None")]
    fn py_new(
        category: Option<String>,
        data_store: Option<String>,
        auto_populate: Option<bool>,
        should_validate_password: Option<bool>,
    ) -> PyResult<Self> {
        Ok(Self {
            om_config: OMDatasetConfig::new(
                category,
                data_store,
                auto_populate,
                should_validate_password,
            )?,
        })
    }

    #[getter]
    pub fn category(&self) -> PyResult<Option<String>> {
        Ok(self.om_config.category.clone())
    }

    #[getter]
    pub fn data_store(&self) -> PyResult<Option<String>> {
        Ok(self.om_config.data_store.clone())
    }

    #[getter]
    pub fn auto_populate(&self) -> PyResult<Option<bool>> {
        Ok(self.om_config.auto_populate.clone())
    }

    #[getter]
    pub fn should_validate_password(&self) -> PyResult<Option<bool>> {
        Ok(self.om_config.should_validate_password.clone())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
        let c = match other.extract::<PyRef<Self>>() {
            Ok(config) => config,
            Err(_) => return Ok(false),
        };
        let result = self.om_config == c.om_config;

        match op {
            CompareOp::Eq => Ok(result),
            CompareOp::Ne => Ok(!result),
            _ => crate::not_implemented_error!(
                "UserDatasetConfig only supports equals and not-equals comparisons"
            ),
        }
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<UserDatasetConfigIter> {
        Ok(UserDatasetConfigIter {
            values: om::TypedValueMap::from(&slf.om_config).into_pairs(),
            i: 0,
        })
    }
}

// TODO add this again?
// #[pyproto]
// impl PyMappingProtocol for UserDatasetConfig {
//     fn __getitem__(&self, id: &str) -> PyResult<PyObject> {
//         om_dsc = self.om_ds_config()?;
//         let users = om::users();
//         match users.user(id) {
//             Ok(u) => Ok(User::new(u.id())?),
//             Err(e) => key_error!(e.to_string())
//         }
//     }

//     fn __len__(&self) -> PyResult<usize> {
//         let users = om::users();
//         Ok(users.users().len())
//     }
// }

#[pyclass]
pub struct UserDatasetConfigIter {
    pub values: Vec<(String, om::TypedValue)>,
    pub i: usize,
}

#[pymethods]
impl UserDatasetConfigIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<(String, Option<PyObject>)>> {
        if slf.i >= slf.values.len() {
            Ok(None)
        } else {
            let retn = (
                slf.values[slf.i].0.to_string(),
                typed_value::to_pyobject(
                    Some(slf.values[slf.i].1.clone()),
                    Some(&slf.values[slf.i].0),
                )?,
            );
            slf.i += 1;
            Ok(Some(retn))
        }
    }
}

#[pyclass]
pub struct PopulateUserReturn {
    om: OmPopulateUserReturn,
}

#[pymethods]
impl PopulateUserReturn {
    #[getter]
    fn succeeded(&self) -> PyResult<bool> {
        Ok(self.om.succeeded())
    }

    #[getter]
    fn outcomes(&self, py: Python) -> PyResult<Py<PyDict>> {
        with_new_pydict(py, |d| {
            for (ds_name, outcome) in self.om.outcomes().iter() {
                match outcome {
                    Some(o) => d.set_item(ds_name, PyOutcome::to_py(py, o)?)?,
                    None => d.set_item(ds_name, py.None())?,
                }
            }
            Ok(())
        })
    }

    #[getter]
    fn failed_outcomes(&self, py: Python) -> PyResult<Py<PyDict>> {
        with_new_pydict(py, |d| {
            for (ds_name, outcome) in self.om.failed_outcomes().iter() {
                d.set_item(ds_name, PyOutcome::to_py(py, outcome)?)?;
            }
            Ok(())
        })
    }

    #[getter]
    fn failed_datasets(&self) -> PyResult<Vec<String>> {
        Ok(self.om.failed_datasets().clone())
    }

    #[getter]
    fn failed(&self) -> PyResult<bool> {
        Ok(self.om.failed())
    }

    #[getter]
    fn errored_outcomes(&self, py: Python) -> PyResult<Py<PyDict>> {
        with_new_pydict(py, |d| {
            for (ds_name, outcome) in self.om.errored_outcomes().iter() {
                d.set_item(ds_name, PyOutcome::to_py(py, outcome)?)?;
            }
            Ok(())
        })
    }

    #[getter]
    fn errored_datasets(&self) -> PyResult<Vec<String>> {
        Ok(self.om.errored_datasets().clone())
    }

    #[getter]
    fn errored(&self) -> PyResult<bool> {
        Ok(self.om.errored())
    }

    fn __bool__(&self) -> PyResult<bool> {
        Ok(self.om.succeeded())
    }
}

impl PopulateUserReturn {
    pub fn from_om(om_rtn: OmPopulateUserReturn) -> Self {
        Self { om: om_rtn }
    }

    pub fn py_from_om(py: Python, om_rtn: OmPopulateUserReturn) -> PyResult<Py<Self>> {
        Py::new(py, Self { om: om_rtn })
    }
}

#[pyclass]
pub struct PopulateUsersReturn {
    om: OmPopulateUsersReturn,
}

#[pymethods]
impl PopulateUsersReturn {
    #[getter]
    fn succeeded(&self) -> PyResult<bool> {
        Ok(self.om.succeeded())
    }

    #[getter]
    fn outcomes(&self, py: Python) -> PyResult<Py<PyDict>> {
        with_new_pydict(py, |d| {
            for (ds_name, pop_rtn) in self.om.outcomes().iter() {
                d.set_item(
                    ds_name,
                    PopulateUserReturn::py_from_om(py, pop_rtn.to_owned())?,
                )?;
            }
            Ok(())
        })
    }

    #[getter]
    fn failed_outcomes(&self, py: Python) -> PyResult<Py<PyDict>> {
        with_new_pydict(py, |d_users| {
            for (failed_uid, outcomes) in self.om.failed_outcomes().iter() {
                let py_dsets = with_new_pydict(py, |d_dsets| {
                    for (ds_name, outcome) in outcomes {
                        d_dsets.set_item(ds_name, PyOutcome::to_py(py, outcome)?)?;
                    }
                    Ok(())
                })?;
                d_users.set_item(failed_uid, py_dsets)?;
            }
            Ok(())
        })
    }

    #[getter]
    fn failed_datasets(&self, py: Python) -> PyResult<Py<PyDict>> {
        Ok(map_to_pydict(py, &mut self.om.failed_datasets().iter())?)
    }

    #[getter]
    fn failed(&self) -> PyResult<bool> {
        Ok(self.om.failed())
    }

    #[getter]
    fn errored_outcomes(&self, py: Python) -> PyResult<Py<PyDict>> {
        with_new_pydict(py, |d_users| {
            for (errored_uid, outcomes) in self.om.errored_outcomes().iter() {
                let py_dsets = with_new_pydict(py, |d_dsets| {
                    for (ds_name, outcome) in outcomes {
                        d_dsets.set_item(ds_name, PyOutcome::to_py(py, outcome)?)?;
                    }
                    Ok(())
                })?;
                d_users.set_item(errored_uid, py_dsets)?;
            }
            Ok(())
        })
    }

    #[getter]
    fn errored_datasets(&self, py: Python) -> PyResult<Py<PyDict>> {
        Ok(map_to_pydict(py, &mut self.om.errored_datasets().iter())?)
    }

    #[getter]
    fn errored(&self) -> PyResult<bool> {
        Ok(self.om.errored())
    }

    fn __bool__(&self) -> PyResult<bool> {
        Ok(self.om.succeeded())
    }
}

impl PopulateUsersReturn {
    pub fn from_om(om_rtn: OmPopulateUsersReturn) -> Self {
        Self { om: om_rtn }
    }
}

#[pyclass(subclass)]
pub struct User {
    user_id: String,
}

impl User {
    pub fn new(id: &str) -> PyResult<Self> {
        Ok(Self {
            user_id: id.to_string(),
        })
    }

    pub fn user_id(&self) -> &String {
        &self.user_id
    }
}

#[pymethods]
impl User {
    #[getter]
    pub fn is_current(&self) -> PyResult<bool> {
        Ok(om::with_user(&self.user_id, |u| u.is_current())?)
    }

    #[getter]
    pub fn is_current_user(&self) -> PyResult<bool> {
        self.is_current()
    }

    #[getter]
    pub fn get_id(&self) -> PyResult<String> {
        Ok(om::with_user(&self.user_id, |u| Ok(u.id().to_string()))?)
    }

    #[getter]
    pub fn get_username(&self) -> PyResult<String> {
        Ok(om::with_user(&self.user_id, |u| u.username())?)
    }

    #[setter]
    pub fn set_username(&self, username: Option<String>) -> PyResult<()> {
        om::with_user(&self.user_id, |u| u.set_username(username.clone()))?;
        Ok(())
    }

    #[getter]
    pub fn get_email(&self) -> PyResult<Option<String>> {
        Ok(om::with_user(&self.user_id, |u| u.get_email())?)
    }

    #[setter]
    pub fn set_email(&self, email: Option<String>) -> PyResult<()> {
        om::with_user(&self.user_id, |u| u.set_email(email.clone()))?;
        Ok(())
    }

    #[getter]
    pub fn get_first_name(&self) -> PyResult<Option<String>> {
        Ok(om::with_user(&self.user_id, |u| u.first_name())?)
    }

    #[setter]
    pub fn set_first_name(&self, first_name: Option<String>) -> PyResult<()> {
        om::with_user(&self.user_id, |u| u.set_first_name(first_name.clone()))?;
        Ok(())
    }

    #[getter]
    pub fn get_last_name(&self) -> PyResult<Option<String>> {
        Ok(om::with_user(&self.user_id, |u| u.last_name())?)
    }

    #[setter]
    pub fn set_last_name(&self, last_name: Option<String>) -> PyResult<()> {
        om::with_user(&self.user_id, |u| u.set_last_name(last_name.clone()))?;
        Ok(())
    }

    #[getter]
    pub fn get_display_name(&self) -> PyResult<String> {
        Ok(om::with_user(&self.user_id, |u| u.display_name())?)
    }

    #[setter]
    pub fn set_display_name(&self, display_name: Option<String>) -> PyResult<()> {
        om::with_user(&self.user_id, |u| u.set_display_name(display_name.clone()))?;
        Ok(())
    }

    /// Gets the password for the default dataset
    #[getter]
    pub fn password(&self) -> PyResult<String> {
        Ok(om::with_user(&self.user_id, |u| {
            u.password(None, true, None)
        })?)
    }

    #[setter]
    pub fn set_password(&self, password: Option<String>) -> PyResult<()> {
        Ok(om::with_user(&self.user_id, |u| {
            u.set_password(password.clone(), None, None)
        })?)
    }

    pub fn validate_password(&self) -> PyResult<PyOutcome> {
        Ok(PyOutcome::from_origen(om::with_user(&self.user_id, |u| {
            u.validate_password(&u.password(None, true, None)?, None)
        })?.outcome()?.clone()))
    }

    // Note: with regards to kwargs['default']:
    // We can't get a optional None value (as least as far as I know...)
    // Passing in None on the Python side makes this look like the argument given, so can't
    // get a nested None.
    #[args(kwargs = "**")]
    fn password_for(&self, motive: &str, kwargs: Option<&PyDict>) -> PyResult<String> {
        let default: Option<Option<String>>;
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
                    default = Some(Some(UserDataset::name_from_pyany(d)?));
                }
            } else {
                default = None;
            }
        } else {
            default = None;
        }
        Ok(om::with_user(&self.user_id, |u| {
            u.password(Some(motive), true, default.as_ref().map( |d| d.as_ref().map( |d2| d2.as_str())))
        })?)
    }

    #[getter]
    fn motives(&self, py: Python) -> PyResult<Py<PyDict>> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(map_to_pydict(py, &mut u.motive_mapping().iter())?)
        })?)
    }

    #[args(replace_existing = "false")]
    fn add_motive(
        &self,
        motive: &str,
        dataset: &PyAny,
        replace_existing: bool,
    ) -> PyResult<Option<String>> {
        Ok(om::with_user_mut(&self.user_id, |u| {
            Ok(u.add_motive(motive.to_string(), UserDataset::name_from_pyany(dataset)?, replace_existing)?)
        })?)
    }

    fn dataset_for(&self, motive: &str) -> PyResult<Option<UserDataset>> {
        Ok(om::with_user(&self.user_id, |u| {
            if let Some(d) = u.dataset_for(motive)? {
                Ok(Some(UserDataset::new(&self.user_id, d)))
            } else {
                Ok(None)
            }
        })?)
    }

    /// Clears all cached passwords for all datasets
    fn clear_cached_passwords(&self) -> PyResult<()> {
        Ok(om::with_user(&self.user_id, |u| {
            u.clear_cached_passwords()
        })?)
    }

    /// Clears the cached password only for the default dataset
    fn clear_cached_password(&self) -> PyResult<()> {
        Ok(om::with_user(&self.user_id, |u| {
            u.clear_cached_password(None)
        })?)
    }

    #[getter]
    pub fn should_validate_passwords(&self) -> PyResult<bool> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(u.should_validate_passwords())
        })?)
    }

    #[setter]
    pub fn set_should_validate_passwords(&self, should_validate_passwords: Option<bool>) -> PyResult<()> {
        Ok(om::with_user_mut(&self.user_id, |u| {
            Ok(u.set_should_validate_passwords(should_validate_passwords))
        })?)
    }

    #[getter]
    pub fn __should_validate_passwords__(&self) -> PyResult<Option<bool>> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(*u.should_validate_passwords_value())
        })?)
    }

    #[getter]
    pub fn prompt_for_passwords(&self) -> PyResult<bool> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(u.prompt_for_passwords())
        })?)
    }

    #[setter]
    pub fn set_prompt_for_passwords(&self, prompt_for_passwords: Option<bool>) -> PyResult<()> {
        Ok(om::with_user_mut(&self.user_id, |u| {
            Ok(u.set_prompt_for_passwords(prompt_for_passwords))
        })?)
    }

    #[getter]
    pub fn __prompt_for_passwords__(&self) -> PyResult<Option<bool>> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(*u.prompt_for_passwords_value())
        })?)
    }

    // TODO?
    //     #[getter]
    //     fn authenticated(&self) -> PyResult<bool> {
    //         Ok(origen::with_user(&self.user_id, |u| Ok(u.authenticated()))?)
    //     }

    // TODO?
    //     // fn switch_to() -> PyResult<()> {
    //     //     // ...
    //     // }

    #[getter]
    fn data_lookup_hierarchy(&self) -> PyResult<Vec<String>> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(u.data_lookup_hierarchy().clone())
        })?)
    }

    #[setter]
    fn set_data_lookup_hierarchy(&self, hierarchy: Vec<String>) -> PyResult<()> {
        Ok(om::with_user_mut(&self.user_id, |u| {
            u.set_data_lookup_hierarchy(hierarchy.clone())
        })?)
    }

    #[getter]
    fn top_datakey(&self) -> PyResult<String> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(u.top_datakey()?.to_string())
        })?)
    }

    #[args(config = "None", replace_existing = "None", as_topmost = "true")]
    pub fn add_dataset(
        &self,
        name: &str,
        config: Option<&PyAny>,
        replace_existing: Option<bool>,
        as_topmost: bool,
    ) -> PyResult<UserDataset> {
        om::add_dataset_to_user(
            &self.user_id,
            name,
            UserDatasetConfig::into_om(config)?,
            replace_existing.unwrap_or(false),
            as_topmost,
        )?;
        Ok(UserDataset::new(&self.user_id, name))
    }

    #[args(config = "None", replace_existing = "None")]
    pub fn register_dataset(
        &self,
        name: &str,
        config: Option<&PyAny>,
        replace_existing: Option<bool>,
    ) -> PyResult<UserDataset> {
        om::register_dataset_with_user(
            &self.user_id,
            name,
            UserDatasetConfig::into_om(config)?,
            replace_existing.unwrap_or(false),
        )?;
        Ok(UserDataset::new(&self.user_id, name))
    }

    #[getter]
    fn get_datasets<'a>(&self, py: Python<'a>) -> PyResult<&'a PyDict> {
        let retn = PyDict::new(py);
        om::with_user(&self.user_id, |u| {
            for n in u.datasets().keys() {
                retn.set_item(
                    n.to_string(),
                    Py::new(py, UserDataset::new(&self.user_id, n))?,
                )?;
            }
            Ok(())
        })?;
        Ok(retn)
    }

    #[getter]
    fn get_datakeys<'a>(&self) -> PyResult<Vec<String>> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(u.datakeys()
                .iter()
                .map(|k| k.to_string())
                .collect::<Vec<String>>())
        })?)
    }

    #[getter]
    fn data_store(&self) -> PyResult<DataStore> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(DataStore::new(&self.user_id, &u.top_datakey()?))
        })?)
    }

    #[getter]
    fn data(&self) -> PyResult<DataStore> {
        self.data_store()
    }

    #[getter]
    fn other(&self) -> PyResult<DataStore> {
        self.data_store()
    }

    #[args(
        repopulate = "false",
        continue_on_error = "false",
        stop_on_failure = "false"
    )]
    pub fn populate(
        &self,
        repopulate: bool,
        continue_on_error: bool,
        stop_on_failure: bool,
    ) -> PyResult<PopulateUserReturn> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(PopulateUserReturn::from_om(u.populate(
                repopulate,
                continue_on_error,
                stop_on_failure,
            )?))
        })?)
    }

    // TODO add?
    // populated
    // populate_attempted

    #[getter]
    pub fn auto_populate(&self) -> PyResult<bool> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(u.should_auto_populate())
        })?)
    }

    #[setter]
    pub fn set_auto_populate(&self, set_to: Option<bool>) -> PyResult<()> {
        Ok(om::with_user_mut(&self.user_id, |u| {
            Ok(u.set_auto_populate(set_to))
        })?)
    }

    #[getter]
    pub fn __auto_populate__(&self) -> PyResult<Option<bool>> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(*u.auto_populate_value())
        })?)
    }

    #[getter]
    pub fn get_home_dir(&self, py: Python) -> PyResult<Option<PyObject>> {
        Ok(om::with_user(&self.user_id, |u| {
            match u.home_dir()?.as_ref() {
                Some(d) => Ok(Some(pypath!(py, d.display()))),
                None => Ok(None),
            }
        })?)
    }

    #[getter]
    pub fn require_home_dir(&self, py: Python) -> PyResult<PyObject> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(pypath!(py, u.require_home_dir()?.display()))
        })?)
    }

    #[setter(home_dir)]
    pub fn home_dir_setter(&self, d: Option<PathBuf>) -> PyResult<()> {
        Ok(om::with_user(&self.user_id, |u| {
            u.set_home_dir(d.clone())?;
            Ok(())
        })?)
    }

    pub fn set_home_dir(&self, d: Option<PathBuf>) -> PyResult<()> {
        self.home_dir_setter(
            if d.is_some() {
                d
            } else {
                origen_metal::framework::users::user::try_default_home_dir(Some(&self.user_id), None)?
            }
        )
    }

    pub fn clear_home_dir(&self) -> PyResult<()> {
        self.home_dir_setter(None)
    }

    #[allow(non_snake_case)]
    #[getter]
    pub fn __dot_origen_dir__(&self, py: Python) -> PyResult<PyObject> {
        Ok(pypath!(py, om::with_user(&self.user_id, |u| {
            u.dot_origen_dir()
        })?.display()))
    }

    #[getter]
    pub fn session_config(&self) -> PyResult<UserSessionConfig> {
        Ok(UserSessionConfig::new(&self.user_id))
    }

    #[getter]
    pub fn sessions(&self) -> PyResult<PySessionGroup> {
        Ok(om::with_user(&self.user_id, |u| {
            let mut sessions = om::sessions();
            Ok(PySessionGroup::new(
                &u.ensure_session(&mut sessions, None)?.2,
            ))
        })?)
    }

    #[getter]
    pub fn session(&self) -> PyResult<PySessionStore> {
        Ok(om::with_user(&self.user_id, |u| {
            let mut sessions = om::sessions();
            let ss = u.ensure_session(&mut sessions, None)?;
            Ok(PySessionStore::new(ss.3, Some(ss.2)))
        })?)
    }

    #[getter]
    pub fn get_roles(&self) -> PyResult<Vec<String>> {
        Ok(om::with_user(&self.user_id, |u| {
            Ok(u.roles()?.iter().map( |r| r.to_owned()).collect::<Vec<String>>())
        })?)
    }

    #[args(roles="*")]
    pub fn add_roles(&self, roles: &PyAny) -> PyResult<Vec<bool>> {
        let v: Vec<String>;
        if let Ok(r) = roles.extract::<String>() {
            v = vec!(r);
        } else if let Ok(r) = roles.extract::<Vec<String>>() {
            v = r;
        } else {
            return type_error!("Cannot interpret roles as either a 'str' or a 'list of strs'");
        }

        Ok(om::with_user(&self.user_id, |u| {
            u.add_roles(&v)
        })?)
    }

    #[args(roles="*")]
    pub fn remove_roles(&self, roles: &PyAny) -> PyResult<Vec<bool>> {
        let v: Vec<String>;
        if let Ok(r) = roles.extract::<String>() {
            v = vec!(r);
        } else if let Ok(r) = roles.extract::<Vec<String>>() {
            v = r;
        } else {
            return type_error!("Cannot interpret roles as either a 'str' or a 'list of strs'");
        }

        Ok(om::with_user(&self.user_id, |u| {
            u.remove_roles(&v)
        })?)
    }

    // The Python user only stores an ID - can just compare the IDs directly.
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
        let o = other.extract::<PyRef<Self>>()?;
        Ok(match op {
            CompareOp::Eq => {
                self.user_id == o.user_id
            }
            CompareOp::Ne => {
                self.user_id != o.user_id
            }
            _ => return Err(pyo3::exceptions::PyNotImplementedError::new_err(format!("Comparison operator '{:?}' is not applicable", op))),
        })
    }
}

#[pyclass]
pub struct UserSessionConfig {
    user_id: String,
}

#[pymethods]
impl UserSessionConfig {
    #[getter]
    pub fn get_root(&self, py: Python) -> PyResult<Option<PyObject>> {
        self.with_om_sc(|_u, sc| match sc.root.as_ref() {
            Some(r) => Ok(Some(pypath!(py, r.display()))),
            None => Ok(None),
        })
    }

    #[setter]
    pub fn set_root(&self, root: Option<PathBuf>) -> PyResult<()> {
        self.with_om_sc_mut(|_u, sc| {
            sc.root = root;
            Ok(())
        })
    }

    #[getter]
    pub fn get_offset(&self, py: Python) -> PyResult<Option<PyObject>> {
        self.with_om_sc(|_u, sc| match sc.offset.as_ref() {
            Some(o) => Ok(Some(pypath!(py, o.display()))),
            None => Ok(None),
        })
    }

    #[setter]
    pub fn set_offset(&self, offset: Option<PathBuf>) -> PyResult<()> {
        self.with_om_sc_mut(|_u, sc| {
            sc.set_offset(offset)?;
            Ok(())
        })
    }

    #[getter]
    pub fn get_file_permissions(&self) -> PyResult<FilePermissions> {
        self.with_om_sc(|_u, sc| Ok((&sc.file_permissions).into()))
    }

    #[setter]
    pub fn set_file_permissions(&self, fp: &PyAny) -> PyResult<()> {
        self.with_om_sc_mut(|_u, sc| {
            sc.file_permissions = FilePermissions::to_metal(fp)?;
            Ok(())
        })
    }

    #[getter]
    pub fn get_fp(&self) -> PyResult<FilePermissions> {
        self.get_file_permissions()
    }

    #[setter]
    pub fn set_fp(&self, fp: &PyAny) -> PyResult<()> {
        self.set_file_permissions(fp)
    }
}

impl UserSessionConfig {
    pub fn new(user_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
        }
    }

    pub fn with_om_sc<T, F>(&self, mut f: F) -> PyResult<T>
    where
        F: FnMut(&OMUser, &OMUserSessionConfig) -> OMResult<T>,
    {
        Ok(om::with_user(&self.user_id, |u| {
            let sc = u.session_config();
            f(u, &sc)
        })?)
    }

    pub fn with_om_sc_mut<T, F>(&self, f: F) -> PyResult<T>
    where
        F: FnOnce(&OMUser, &mut OMUserSessionConfig) -> OMResult<T>,
    {
        let users = om::users();
        let u = users.user(&self.user_id)?;
        let mut sc = u.session_config_mut()?;
        Ok(f(u, &mut sc)?)
    }
}

#[pyclass]
struct DataStore {
    user_id: String,
    dataset: String,
}

impl DataStore {
    pub fn new(user_id: &str, dataset: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            dataset: dataset.to_string(),
        }
    }
}

#[pymethods]
impl DataStore {
    fn get(&self, key: &str) -> PyResult<Option<PyObject>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| {
                if let Some(o) = d.other.get(key) {
                    Ok(typed_value::to_pyobject(Some(o.clone()), Some(key))?)
                } else {
                    Ok(None)
                }
            },
        )?)
    }

    pub fn keys(&self) -> PyResult<Vec<String>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| {
                Ok(d.other
                    .keys()
                    .map(|k| k.to_string())
                    .collect::<Vec<String>>())
            },
        )?)
    }

    fn values(&self) -> PyResult<Vec<Option<PyObject>>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| {
                let mut retn: Vec<Option<PyObject>> = vec![];
                for (key, obj) in d.other.iter() {
                    retn.push(typed_value::to_pyobject(Some(obj.clone()), Some(&key))?);
                }
                Ok(retn)
            },
        )?)
    }

    fn items(&self) -> PyResult<Vec<(String, Option<PyObject>)>> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| {
                let mut retn: Vec<(String, Option<PyObject>)> = vec![];
                for (key, obj) in d.other.iter() {
                    retn.push((
                        key.to_string(),
                        typed_value::to_pyobject(Some(obj.clone()), Some(&key))?,
                    ));
                }
                Ok(retn)
            },
        )?)
    }

    fn __getitem__(&self, key: &str) -> PyResult<Option<PyObject>> {
        let obj = om::with_user_dataset(Some(&self.user_id), &self.dataset, |d| {
            if let Some(o) = d.other.get(key) {
                Ok(Some(o.clone()))
            } else {
                Ok(None)
            }
        })?;
        if let Some(o) = obj {
            typed_value::to_pyobject(Some(o), Some(key))
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err(format!(
                "No data added with key '{}' in dataset '{}' for user '{}'",
                key, self.dataset, self.user_id
            )))
        }
    }

    fn __setitem__(&mut self, key: &str, value: &PyAny) -> PyResult<()> {
        om::with_user_dataset_mut(Some(&self.user_id), &self.dataset, |d| {
            Ok(d.other
                .insert(&key.to_string(), typed_value::from_pyany(value)?))
        })?;
        Ok(())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(om::with_user_dataset(
            Some(&self.user_id),
            &self.dataset,
            |d| Ok(d.other.len()),
        )?)
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<UsersIter> {
        Ok(UsersIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}

#[pyclass]
pub struct DataStoreIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pymethods]
impl DataStoreIter {
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
