// TODO Need to clean up

use crate::framework::users::{User, UserDataset};
use crate::prelude::*;
use om::with_user;
use origen_metal::utils::ldap::LdapPopUserConfig as OmLdapPopUserConfig;
use origen_metal::utils::ldap::SupportedAuths;
use origen_metal::utils::ldap::LDAP as OmLdap;
use origen_metal::Result as OMResult;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict};
use std::collections::HashMap;

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "ldap")?;
    subm.add_class::<LDAP>()?;
    m.add_submodule(subm)?;
    Ok(())
}

enum InnerLDAP {
    // https://github.com/rust-lang/rust/issues/33685
    // Func((
    //     Box<dyn for<'a> Fn(&'a LDAP) -> OMResult<&'a OmLdap> + Send>,
    //     Box<dyn for<'a> Fn(&'a ()) -> OMResult<&'a mut OmLdap> + Send>,
    // )),
    Om(OmLdap),
}

// pub struct SimpleBindBuilder {}

/// A single LDAP instance
#[pyclass(subclass)]
pub struct LDAP {
    // name: String,
    inner: InnerLDAP,
}

impl LDAP {
    // TODO remove inner ldap stuff. not needed
    pub fn with_inner_ldap<F, T>(&self, func: F) -> OMResult<T>
    where
        F: FnOnce(&OmLdap) -> OMResult<T>,
    {
        match &self.inner {
            // InnerLDAP::Func(f) => {
            //     func(f.0(&self)?)
            // },
            InnerLDAP::Om(l) => func(&l),
        }
    }

    // pub fn with_mut_inner_ldap<F, T>(&mut self, func: F) -> OMResult<T>
    // where
    //     F: FnOnce(&mut OmLdap) -> OMResult<T>
    // {
    //     // let needs_mut = match self.inner_ldap
    //     match &mut self.inner {
    //         InnerLDAP::Func(f) => {
    //             func(f.1(&())?)
    //         },
    //         InnerLDAP::Om(ref mut l) => {
    //             func(l)
    //         }
    //     }
    // }
}

#[pymethods]
impl LDAP {
    #[new]
    #[args(
        username = "None",
        password = "None",
        populate_user_config = "None",
        timeout = "None"
    )]
    fn new(
        name: &str,
        server: &str,
        base: &str,
        auth: Option<&PyDict>,
        continuous_bind: Option<bool>,
        populate_user_config: Option<&PyDict>,
        timeout: Option<&PyAny>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: {
                InnerLDAP::Om({
                    OmLdap::new(
                        name,
                        server,
                        base,
                        continuous_bind.unwrap_or(false),
                        {
                            if let Some(a) = auth {
                                let scheme;
                                if let Some(s) = a.get_item("scheme") {
                                    scheme = s.extract::<String>()?;
                                } else {
                                    scheme = "simple_bind".to_string();
                                }

                                match SupportedAuths::from_str(scheme.as_str())? {
                                    SupportedAuths::SimpleBind(mut sb) => {
                                        if let Some(username) = a.get_item("username") {
                                            sb.username = Some(username.extract::<String>()?);
                                        }
                                        if let Some(password) = a.get_item("password") {
                                            sb.password = Some(password.extract::<String>()?);
                                        }
                                        if let Some(priority_motives) =
                                            a.get_item("priority_motives")
                                        {
                                            sb.priority_motives =
                                                priority_motives.extract::<Vec<String>>()?;
                                        }
                                        if let Some(backup_motives) = a.get_item("backup_motives") {
                                            sb.backup_motives =
                                                backup_motives.extract::<Vec<String>>()?;
                                        }
                                        if let Some(allow_default_password) =
                                            a.get_item("allow_default_password")
                                        {
                                            sb.allow_default_password =
                                                allow_default_password.extract::<bool>()?;
                                        }
                                        if let Some(use_default_motives) =
                                            a.get_item("use_default_motives")
                                        {
                                            sb.use_default_motives =
                                                use_default_motives.extract::<bool>()?;
                                        }
                                        SupportedAuths::SimpleBind(sb)
                                    }
                                }
                            } else {
                                SupportedAuths::from_str("simple_bind")?
                            }
                        },
                        {
                            if let Some(py_t) = timeout {
                                if let Ok(b) = py_t.downcast::<PyBool>() {
                                    if b.is_true() {
                                        None
                                    } else {
                                        Some(None)
                                    }
                                } else {
                                    Some(Some(py_t.extract::<u64>()?))
                                }
                            } else {
                                None
                            }
                        },
                        {
                            if let Some(pop_config) = populate_user_config {
                                let mut config = OmLdapPopUserConfig::default();
                                if let Some(data_id) = pop_config.get_item("data_id") {
                                    config.data_id = data_id.extract::<String>()?;
                                }
                                if let Some(mapping) = pop_config.get_item("mapping") {
                                    config.mapping =
                                        mapping.extract::<HashMap<String, String>>()?;
                                }
                                if let Some(required) = pop_config.get_item("required") {
                                    config.required = required.extract::<Vec<String>>()?;
                                }
                                if let Some(include_all) = pop_config.get_item("include_all") {
                                    config.include_all = include_all.extract::<bool>()?;
                                }
                                if let Some(attrs) = pop_config.get_item("attributes") {
                                    config.attributes = Some(attrs.extract::<Vec<String>>()?);
                                }
                                Some(config)
                            } else {
                                None
                            }
                        },
                    )?
                })
            },
        })
    }

    /// Retrieves the server this LDAP was instantiated with
    #[getter]
    fn get_server(&self) -> PyResult<String> {
        // let ldaps = origen::ldaps();
        // let ldap = ldaps._get(&self.name)?;
        // Ok(ldap.server().to_string())
        // self.with_inner_ldap( |om_ldap| {
        //     Ok(om_ldap.server().to_string())
        // })?;
        Ok(self.with_inner_ldap(|om_ldap| Ok(om_ldap.server().to_string()))?)
    }

    /// Retrieves the base DNs
    #[getter]
    fn get_base(&self) -> PyResult<String> {
        // let ldaps = origen::ldaps();
        // let ldap = ldaps._get(&self.name)?;
        // Ok(ldap.base().to_string())
        Ok(self.with_inner_ldap(|om_ldap| Ok(om_ldap.base().to_string()))?)
    }

    /// Retrieves this LDAP's name. Does not actually influence anything in the connection itself
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // Ok(ldap.name().to_string())
        Ok(self.with_inner_ldap(|om_ldap| Ok(om_ldap.name().to_string()))?)
    }

    /// Retrieves this LDAP's authentication configuration
    #[getter]
    fn get_auth_config<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // Ok(ldap.auth().to_hashmap())
        Ok(self.with_inner_ldap(|om_ldap| {
            Ok(crate::_helpers::typed_value::into_pydict(
                py,
                om_ldap.auth().config_into_map(),
            )?)
        })?)
    }

    /// Retrieves this LDAP's authentication configuration after resolution
    #[getter]
    fn get_auth<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // Ok(ldap.auth().to_hashmap())
        Ok(self.with_inner_ldap(|om_ldap| {
            Ok(crate::_helpers::typed_value::into_pydict(
                py,
                om_ldap.auth().resolve_and_into_map(om_ldap)?,
            )?)
        })?)
    }

    /// Indicates whether this LDAP is currently bound. Returns true only if previously bound
    /// and the bind attempt was successful
    #[getter]
    fn get_bound(&self) -> PyResult<bool> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // Ok(ldap.bound())
        Ok(self.with_inner_ldap(|om_ldap| Ok(om_ldap.bound()))?)
    }

    #[getter]
    fn get_timeout(&self) -> PyResult<Option<u64>> {
        Ok(self.with_inner_ldap(|om_ldap| Ok(om_ldap.timeout()))?)
    }

    #[getter]
    fn get_continuous_bind(&self) -> PyResult<bool> {
        Ok(self.with_inner_ldap(|om_ldap| Ok(om_ldap.continuous_bind()))?)
    }

    /// search(filter: str, attrs: list) -> dict
    ///
    /// Runs a search with the given |ldap:filter| and ``attribute list``. Returns a nested
    /// |dict| where the first level are the returned ``DNs``. Each ``DN`` is a tuple with
    /// exactly two items in this order: ``(returned data, binary returned data)``. Whether
    /// or not your query returns data, binary data, or both is dependent on the server
    /// configuration and query itself.
    ///
    /// Args:
    ///     filter (str): Search criteria formatted as an |ldap:filter|
    ///     attrs (list): List of attributes to retrieve from matching search criteria.
    ///                   An empty list returns all available attributes, equivalent to ["*"]
    ///
    /// Returns:
    ///     dict: Nested |dict| where the first level is the returned ``DNs``. Each ``DN``
    ///     is a tuple with  exactly two items in this order: ``(returned data, binary returned data)``.
    ///     Whether or not your query returns data, binary data, or both is dependent on the server configuration and query itself.
    ///
    /// See Also
    /// --------
    /// * For examples, see the specs tests written against a free LDAP server
    /// * {{ link_to('origen_utilities:ldap', 'LDAP in the guides') }}
    fn search(
        &self,
        filter: &str,
        attrs: Vec<&str>,
    ) -> PyResult<HashMap<String, (HashMap<String, Vec<String>>, HashMap<String, Vec<Vec<u8>>>)>>
    {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // Ok(ldap.search(filter, attrs)?)
        Ok(self.with_inner_ldap(|om_ldap| om_ldap.search(filter, attrs))?)
    }

    /// single_filter_search(filter: str, attrs: list) -> tuple
    ///
    /// Similar to :meth:search except that this removes the first ``dict`` layer when a single ``DN`` is expected.
    ///
    /// Args:
    ///     filter (str): Search criteria formatted as an |ldap:filter|, expecting to yield at most one ``DN``.
    ///     attrs (list): List of attributes to retrieve from matching search criteria.
    ///                   An empty list returns all available attributes, equivalent to ["*"]
    ///
    /// Returns:
    ///     tuple: Two-item tuple, each item being a dict corresponding to ``(returned data, binary returned data)``
    ///     respectively.
    fn single_filter_search(
        &self,
        filter: &str,
        attrs: Vec<&str>,
    ) -> PyResult<(HashMap<String, Vec<String>>, HashMap<String, Vec<Vec<u8>>>)> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // Ok(ldap.single_filter_search(filter, attrs)?)
        Ok(self.with_inner_ldap(|om_ldap| om_ldap.single_filter_search(filter, attrs))?)
    }

    /// bind(self) -> bool
    ///
    /// Attempts to bind using its current Auth settings.
    ///
    /// Returns:
    ///     bool: ``True`` if the bind was successful. Raises an error otherwise. Note this method
    ///     will never return ``False``.
    fn bind(&self) -> PyResult<bool> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // ldap.bind()?;
        // Ok(true)
        self.with_inner_ldap(|om_ldap| om_ldap.bind())?;
        Ok(true)
    }

    fn unbind(&self) -> PyResult<bool> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // ldap.unbind()?;
        // Ok(true)
        Ok(self.with_inner_ldap(|om_ldap| om_ldap.unbind())?)
    }

    fn validate_credentials(&self, username: &str, password: &str) -> PyResult<bool> {
        Ok(self.with_inner_ldap(|om_ldap| om_ldap.try_password(username, password))?)
    }

    // TEST_NEEDED
    // TODO
    #[getter]
    fn get_populate_user_config<'py>(&self, _py: Python<'py>) -> PyResult<Option<&'py PyDict>> {
        todo!();
        // Ok(self.with_inner_ldap( |om_ldap| {
        //     Ok(crate::_helpers::typed_value::into_optional_pydict(py, om_ldap.populate_user_config().config_into_map())?)
        // })?)
    }

    fn populate_user(&self, user: PyRef<User>, dataset: PyRef<UserDataset>) -> PyResult<PyOutcome> {
        Ok(self
            .with_inner_ldap(|om_ldap| {
                with_user(&user.user_id(), |u| {
                    u.with_dataset_mut(dataset.dataset(), |d| om_ldap.populate_user(u, d))
                })
            })?
            .into())
    }
}
