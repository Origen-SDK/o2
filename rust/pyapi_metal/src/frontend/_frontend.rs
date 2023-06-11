use super::py_data_stores::PyDataStoreCategory;
use super::{with_py_frontend, PyFrontend};
use crate::framework::outcomes::pyobj_into_om_outcome;
use crate::framework::outcomes::Outcome as PyOutcome;
use origen_metal::prelude::frontend::*;
use origen_metal::{Outcome, TypedValue, TypedValueMap, TypedValueVec};

use crate::runtime_error;
use indexmap::IndexMap;
use origen_metal::log_trace;
use origen_metal::Result as OMResult;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::framework::users::{User, UserDataset};

use crate::_helpers::typed_value;

pub struct Frontend {
    rc: crate::utils::revision_control::_frontend::RevisionControlFrontend,
    // TODO
    // mailers: HashMap<String, crate::utils::mailer::_frontend::MailerFrontend>,
    // mailer: Option<String>,
    // maillists:
}

impl Frontend {
    pub fn new() -> PyResult<Self> {
        log_trace!("PyAPI Metal: Creating new frontend");
        PyFrontend::initialize()?;
        Ok(Self {
            rc: crate::utils::revision_control::_frontend::RevisionControlFrontend {},
        })
    }
}

impl origen_metal::frontend::FrontendAPI for Frontend {
    fn revision_control(&self) -> OMResult<Option<&dyn RevisionControlFrontendAPI>> {
        Ok(with_py_frontend(|_py, py_frontend| {
            if py_frontend.rc.is_some() {
                Ok(Some(&self.rc as &dyn RevisionControlFrontendAPI))
            } else {
                Ok(None)
            }
        })?)
    }

    fn data_store_categories(
        &self,
    ) -> OMResult<IndexMap<String, Box<dyn DataStoreCategoryFrontendAPI>>> {
        let mut retn: IndexMap<String, Box<dyn DataStoreCategoryFrontendAPI>> = IndexMap::new();
        with_py_frontend(|py, py_frontend| {
            for (n, cat) in py_frontend.data_stores.borrow(py).rusty_categories() {
                retn.insert(n.to_string(), Box::new(cat.borrow(py).into_frontend()?));
            }
            Ok(())
        })?;
        Ok(retn)
    }

    fn get_data_store_category(
        &self,
        category: &str,
    ) -> OMResult<Option<Box<dyn DataStoreCategoryFrontendAPI>>> {
        let om_cat = with_py_frontend(|py, py_frontend| {
            if let Some(cat) = py_frontend.data_stores.borrow(py).get(py, category)? {
                let c = &*cat.borrow(py);
                Ok(Some(c.into_frontend()?))
            } else {
                Ok(None)
            }
        })?;
        match om_cat {
            Some(c) => Ok(Some(Box::new(c))),
            None => Ok(None),
        }
    }

    fn add_data_store_category(
        &self,
        cat: &str,
        load_function: Option<TypedValue>,
        autoload: Option<bool>,
    ) -> OMResult<Box<dyn DataStoreCategoryFrontendAPI>> {
        with_py_frontend(|py, py_frontend| {
            py_frontend
                .data_stores
                .borrow_mut(py)
                .add_category(
                    py,
                    cat,
                    match &load_function {
                        Some(f) => Some(pyo3::types::PyString::new(py, &f.as_string()?).to_object(py)),
                        None => None
                    },
                    autoload,
                )?;
            Ok(())
        })?;
        Ok(Box::new(DataStoreCategoryFrontend::new(cat)))
    }

    fn remove_data_store_category(&self, cat: &str) -> OMResult<()> {
        with_py_frontend(|py, py_frontend| {
            py_frontend
                .data_stores
                .borrow_mut(py)
                .remove_category(py, cat)
        })?;
        Ok(())
    }

    fn lookup_current_user(&self) -> Option<OMResult<Option<String>>> {
        let fe_result = with_py_frontend(|py, fe| {
            // TODO have a lazy static for keys like this?
            Ok(match fe._users_.get("lookup_current_id_function") {
                Some(f) => {
                    let result = f.call0(py)?;
                    if result.is_none(py) {
                        Some(None)
                    } else {
                        Some(Some(result.extract::<String>(py)?))
                    }
                }
                None => None,
            })
        });
        match fe_result {
            Ok(r) => match r {
                Some(res) => return Some(Ok(res)),
                None => None,
            },
            Err(e) => Some(Err(e.into())),
        }
    }

    fn lookup_home_dir(&self, user_id: &str, dataset: Option<&str>, is_current: bool) -> Option<OMResult<Option<PathBuf>>> {
        let fe_result = with_py_frontend(|py, fe| {
            Ok(match fe._users_.get(*super::LOOKUP_HOME_DIR_FUNC_KEY) {
                Some(f) => {
                    let args = (
                        User::new(user_id)?,
                        if let Some(d) = dataset {
                            Py::new(py, UserDataset::new(user_id, d))?.to_object(py)
                        } else {
                            py.None()
                        },
                        is_current,
                    );

                    let result = f.call1(py, args)?;
                    if result.is_none(py) {
                        Some(None)
                    } else if let Ok(b) = result.extract::<bool>(py) {
                        if b {
                            return runtime_error!("'True' is not a valid return value when looking up a user's home directory")
                        } else {
                            None
                        }
                    } else {
                        Some(Some(result.extract::<PathBuf>(py)?))
                    }
                }
                None => None,
            })
        });
        match fe_result {
            Ok(r) => match r {
                Some(res) => return Some(Ok(res)),
                None => None,
            },
            Err(e) => Some(Err(e.into())),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct DataStoreCategoryFrontend {
    name: String,
}

impl DataStoreCategoryFrontend {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn as_py<F, T>(&self, mut func: F) -> PyResult<T>
    where
        F: FnMut(Python, &Py<PyDataStoreCategory>) -> PyResult<T>,
    {
        with_py_frontend(|py, py_frontend| {
            match py_frontend.data_stores.borrow(py).get(py, &self.name)? {
                Some(cat) => func(py, cat),
                None => crate::runtime_error!(format!(
                    "Stale data store category '{}' encountered",
                    self.name
                )),
            }
        })
    }
}

impl DataStoreCategoryFrontendAPI for DataStoreCategoryFrontend {
    fn name(&self) -> &str {
        &self.name
    }

    fn add_data_store(
        &self,
        name: &str,
        parameters: TypedValueMap,
        _backend: Option<TypedValueMap>,
    ) -> OMResult<Box<dyn DataStoreFrontendAPI>> {
        self.as_py( |py, py_cat| {
            let params = typed_value::into_pydict(py, parameters.typed_values())?;
            let cls = match params.get_item("class") {
                Some(c) => c,
                None => return runtime_error!(format!(
                    "Missing parameter 'class' when adding data store '{}' to category '{}'. A 'class' must be provided!",
                    name,
                    self.name
                ))
            };
            params.del_item("class")?;

            let list_args = match params.get_item("list_args") {
                Some(args) => {
                    params.del_item("list_args")?;
                    Some(args.extract::<&PyList>()?)
                },
                None => None
            };

            let cat = &mut *py_cat.borrow_mut(py);
            cat.add(py, name, cls, list_args, Some(params), None)?;
            Ok(())
        })?;
        Ok(Box::new(DataStoreFrontend::new(&self.name, name)))
    }

    fn remove_data_store(&self, name: &str) -> OMResult<()> {
        Ok(self.as_py(|py, py_cat| {
            let cat = &mut *py_cat.borrow_mut(py);
            cat.remove(py, name)
        })?)
    }

    fn data_stores(&self) -> OMResult<HashMap<String, Box<dyn DataStoreFrontendAPI>>> {
        let objs = self.as_py(|py, py_cat| {
            let cat = &*py_cat.borrow(py);
            Ok(cat
                .objects()?
                .keys()
                .map(|k| k.to_string())
                .collect::<Vec<String>>())
        })?;
        let mut retn: HashMap<String, Box<dyn DataStoreFrontendAPI>> = HashMap::new();
        for o in objs {
            let ds = DataStoreFrontend::new(&self.name, &o);
            retn.insert(o.to_string(), Box::new(ds));
        }
        Ok(retn)
    }

    fn get_data_store(&self, key: &str) -> OMResult<Option<Box<dyn DataStoreFrontendAPI>>> {
        let ds = self.as_py(|py, py_cat| {
            let cat = &*py_cat.borrow(py);
            if cat.objects()?.contains_key(key) {
                Ok(Some(DataStoreFrontend::new(&self.name, key)))
            } else {
                Ok(None)
            }
        })?;
        match ds {
            Some(om_ds) => Ok(Some(Box::new(om_ds))),
            None => Ok(None),
        }
    }
}

pub struct DataStoreFrontend {
    name: String,
    category: String,
}

impl DataStoreFrontend {
    pub fn new(category: &str, name: &str) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
        }
    }

    pub fn as_py<F, T>(&self, mut func: F) -> PyResult<T>
    where
        F: FnMut(Python, &PyObject) -> PyResult<T>,
    {
        with_py_frontend(|py, py_frontend| {
            match py_frontend.data_stores.borrow(py).get(py, &self.category)? {
                Some(cat) => {
                    let c = &*cat.borrow(py);
                    match c.objects()?.get(&self.name) {
                        Some(ds) => func(py, ds),
                        None => {
                            crate::runtime_error!(format!(
                                "Stale data store '{}' encountered",
                                self.name,
                            ))
                        }
                    }
                }
                None => crate::runtime_error!(format!(
                    "Stale data store category '{}' encountered. Cannot retrieve data store '{}'",
                    self.category, self.name
                )),
            }
        })
    }

    pub fn as_py_with_cat<F, T>(&self, mut func: F) -> PyResult<T>
    where
        F: FnMut(Python, &Py<PyDataStoreCategory>, &PyObject) -> PyResult<T>,
    {
        with_py_frontend(|py, py_frontend| {
            match py_frontend.data_stores.borrow(py).get(py, &self.category)? {
                Some(cat) => {
                    let c = &*cat.borrow(py);
                    match c.objects()?.get(&self.name) {
                        Some(ds) => func(py, cat, ds),
                        None => {
                            crate::runtime_error!(format!(
                                "Stale data store '{}' encountered",
                                self.name,
                            ))
                        }
                    }
                }
                None => crate::runtime_error!(format!(
                    "Stale data store category '{}' encountered. Cannot retrieve data store '{}'",
                    self.category, self.name
                )),
            }
        })
    }
}

impl DataStoreFrontendAPI for DataStoreFrontend {
    fn name(&self) -> OMResult<&str> {
        self.as_py(|_py, _py_self| Ok(()))?;
        Ok(&self.name)
    }

    fn category(&self) -> OMResult<Box<dyn DataStoreCategoryFrontendAPI>> {
        let cat = self.as_py_with_cat(|py, cat, _py_self| Ok(cat.borrow(py).into_frontend()?))?;
        Ok(Box::new(cat))
    }

    fn class(&self, backend: Option<TypedValueMap>) -> OMResult<String> {
        Ok(self.as_py(|py, py_self| {
            py_self
                .call_method(
                    py,
                    "get_data_store_class",
                    PyTuple::empty(py),
                    typed_value::into_optional_pydict(py, backend.as_ref())?,
                )?
                .extract::<String>(py)
        })?)
    }

    //  fn features(&self) -> &std::vec::Vec<DsFeature> { todo!() }
    //  fn implementor(&self) -> &str { todo!() }
    //  fn init(&self, _: origen_metal::TypedValueMap) -> OMResult<Option<origen_metal::Outcome>> { todo!() }
    fn get(&self, key: &str) -> OMResult<Option<TypedValue>> {
        Ok(self.as_py(|py, py_self| {
            let result = py_self.call_method1(py, "get", (key,))?;
            if result.is_none(py) {
                Ok(None)
            } else {
                Ok(Some(typed_value::from_pyany(result.extract(py)?)?))
            }
        })?)
    }

    fn remove(&self, key: &str) -> OMResult<Option<TypedValue>> {
        Ok(self.as_py(|py, py_self| match self.contains(key)? {
            true => {
                let result = py_self.call_method1(py, "remove", (key,))?;
                Ok(Some(typed_value::from_pyany(result.extract(py)?)?))
            }
            false => Ok(None),
        })?)
    }

    fn contains(&self, query: &str) -> OMResult<bool> {
        Ok(self.as_py(|py, py_self| {
            let result = py_self.call_method1(py, "__contains__", (query,))?;
            result.extract::<bool>(py)
        })?)
    }

    fn store(&self, key: &str, obj: TypedValue) -> OMResult<bool> {
        Ok(self.as_py(|py, py_self| {
            let result = py_self.call_method1(
                py,
                "store",
                (key, typed_value::to_pyobject(Some(obj.clone()), Some(key))?),
            )?;
            result.extract::<bool>(py)
        })?)
    }

    fn items(&self) -> OMResult<TypedValueMap> {
        Ok(self.as_py(|py, py_self| {
            let result = py_self.call_method0(py, "items")?;
            Ok(typed_value::from_pydict(result.extract::<&PyDict>(py)?)?)
        })?)
    }

    fn call(
        &self,
        func: &str,
        pos_args: Option<TypedValueVec>,
        kw_args: Option<TypedValueMap>,
        _backend: Option<TypedValueMap>,
    ) -> OMResult<Outcome> {
        Ok(self.as_py(|py, py_self| {
            let result = py_self.call_method(
                py,
                func,
                typed_value::into_pytuple(
                    py,
                    &mut pos_args
                        .as_ref()
                        .unwrap_or(&TypedValueVec::new())
                        .typed_values()
                        .iter(),
                )?,
                typed_value::into_optional_pydict(py, kw_args.as_ref())?,
            )?;

            let rtn: Outcome;
            if let Ok(r) = result.extract::<PyRef<PyOutcome>>(py) {
                rtn = r.into_origen()?;
            } else {
                rtn = PyOutcome::new_om_inferred(Some(result.as_ref(py)))?;
            }
            Ok(rtn)
        })?)
    }

    fn populate_user(&self, user_id: &str, ds: &str) -> OMResult<FeatureReturn> {
        Ok(self.as_py(|py, py_self| {
            let func = py_self.call_method1(py, "__lookup_origen_feature__", ("populate_user",))?;
            if func.is_none(py) {
                // Not implemented
                Ok(self.unimplemented("populate_user")?)
            } else {
                let py_u = crate::framework::users::User::new(user_id)?;
                let py_ds = crate::framework::users::UserDataset::new(user_id, ds);
                let result = func.call(py, (py_self, py_u, py_ds), None)?;
                Ok(FeatureReturn::new(Ok(pyobj_into_om_outcome(py, result)?)))
            }
        })?)
    }

    fn validate_password(&self, username: &str, password: &str, user_id: &str, ds_name: &str) -> OMResult<FeatureReturn> {
        Ok(self.as_py(|py, py_self| {
            let func = py_self.call_method1(py, "__lookup_origen_feature__", ("validate_password",))?;
            if func.is_none(py) {
                // Not implemented
                Ok(self.unimplemented("validate_password")?)
            } else {
                let py_u = Py::new(py, crate::framework::users::User::new(user_id)?)?;
                let py_ds = Py::new(py, crate::framework::users::UserDataset::new(user_id, ds_name))?;
                let kwargs = PyDict::new(py);
                kwargs.set_item("user", py_u)?;
                kwargs.set_item("dataset", py_ds)?;
                let result = func.call(py, (py_self, username, password), Some(kwargs))?;
                Ok(FeatureReturn::new(Ok(pyobj_into_om_outcome(py, result)?)))
            }
        })?)
    }
}
