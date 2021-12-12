// TODO Need to clean up

// use pyo3::class::mapping::PyMappingProtocol;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
// use origen_metal::utils::ldap::LDAPs as OmLdaps;
use origen_metal::utils::ldap::LDAP as OmLdap;
use origen_metal::utils::ldap::SupportedAuths;
use origen_metal::Result as OMResult;

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "ldap")?;
    subm.add_class::<LDAPs>()?;
    subm.add_class::<LDAP>()?;
    subm.add_wrapped(wrap_pyfunction!(ldaps))?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyfunction]
fn ldaps(_py: Python) -> PyResult<LDAPs> {
    Ok(LDAPs {})
}

/// Dict-like container to retrieve defined LDAP instances.
#[pyclass]
pub struct LDAPs {}
/*
#[pymethods]
impl LDAPs {
    fn keys(&self) -> PyResult<Vec<String>> {
        let ldaps = origen::ldaps();
        Ok(ldaps.ldaps().keys().cloned().collect())
    }

    fn values(&self) -> PyResult<Vec<LDAP>> {
        let ldaps = origen::ldaps();
        let mut retn = vec![];
        for n in ldaps.ldaps().keys() {
            retn.push(LDAP {
                name: n.to_string(),
            });
        }
        Ok(retn)
    }

    fn items(&self) -> PyResult<Vec<(String, LDAP)>> {
        let ldaps = origen::ldaps();
        let mut retn = vec![];
        for n in ldaps.ldaps().keys() {
            retn.push((
                n.to_string(),
                LDAP {
                    name: n.to_string(),
                },
            ));
        }
        Ok(retn)
    }

    fn get(&self, ldap: &str) -> PyResult<Option<LDAP>> {
        let ldaps = origen::ldaps();
        if ldaps.ldaps().contains_key(ldap) {
            Ok(Some(LDAP {
                name: ldap.to_string(),
            }))
        } else {
            Ok(None)
        }
    }
}

#[pyproto]
impl PyMappingProtocol for LDAPs {
    fn __getitem__(&self, ldap: &str) -> PyResult<LDAP> {
        if let Some(l) = self.get(ldap)? {
            Ok(l)
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err(format!(
                "No LDAP named '{}' available",
                ldap
            )))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let ldaps = origen::ldaps();
        Ok(ldaps.ldaps().len())
    }
}

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
*/

enum InnerLDAP {
    // https://github.com/rust-lang/rust/issues/33685
    // Func((
    //     Box<dyn for<'a> Fn(&'a LDAP) -> OMResult<&'a OmLdap> + Send>,
    //     Box<dyn for<'a> Fn(&'a ()) -> OMResult<&'a mut OmLdap> + Send>,
    // )),
    Om(OmLdap),
}

/// A single LDAP instance
#[pyclass(subclass)]
pub struct LDAP {
    // name: String,
    inner: InnerLDAP,
}

impl LDAP {
    pub fn with_inner_ldap<F, T>(&self, func: F) -> OMResult<T>
    where
        F: FnOnce(&OmLdap) -> OMResult<T>
    {
        match &self.inner {
            // InnerLDAP::Func(f) => {
            //     func(f.0(&self)?)
            // },
            InnerLDAP::Om(l) => {
                func(&l)
            }
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
    #[args(auth="\"simple_bind\"", username="None", password="None")]
    fn new(name: &str, server: &str, base: &str, auth: &str, username: Option<String>, password: Option<String>) -> PyResult<Self> {
        Ok(Self {
            inner: {
                InnerLDAP::Om({
                    OmLdap::new(name, server, base, SupportedAuths::from_str(auth, username.as_ref(), password.as_ref())?)?
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
        Ok(self.with_inner_ldap( |om_ldap| {
            Ok(om_ldap.server().to_string())
        })?)
    }

    /// Retrieves the base DNs
    #[getter]
    fn get_base(&self) -> PyResult<String> {
        // let ldaps = origen::ldaps();
        // let ldap = ldaps._get(&self.name)?;
        // Ok(ldap.base().to_string())
        Ok(self.with_inner_ldap( |om_ldap| {
            Ok(om_ldap.base().to_string())
        })?)
    }

    /// Retrieves this LDAP's name. Does not actually influence anything in the connection itself
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // Ok(ldap.name().to_string())
        Ok(self.with_inner_ldap( |om_ldap| {
            Ok(om_ldap.name().to_string())
        })?)
    }

    /// Retrieves this LDAP's authentication configuration
    #[getter]
    fn get_auth(&self) -> PyResult<HashMap<String, String>> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // Ok(ldap.auth().to_hashmap())
        Ok(self.with_inner_ldap( |om_ldap| {
            Ok(om_ldap.auth().to_hashmap())
        })?)
    }

    /// Indicates whether this LDAP is currently bound. Returns true only if previously bound
    /// and the bind attempt was successful
    #[getter]
    fn get_bound(&self) -> PyResult<bool> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // Ok(ldap.bound())
        Ok(self.with_inner_ldap( |om_ldap| {
            Ok(om_ldap.bound())
        })?)
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
        Ok(self.with_inner_ldap( |om_ldap| {
            om_ldap.search(filter, attrs)
        })?)
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
        Ok(self.with_inner_ldap( |om_ldap| {
            om_ldap.single_filter_search(filter, attrs)
        })?)
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
        self.with_inner_ldap( |om_ldap| {
            om_ldap.bind()
        })?;
        Ok(true)
    }

    fn unbind(&self) -> PyResult<bool> {
        // let mut ldaps = origen::ldaps();
        // let ldap = ldaps._get_mut(&self.name)?;
        // ldap.unbind()?;
        // Ok(true)
        self.with_inner_ldap( |om_ldap| {
            om_ldap.unbind()
        })?;
        Ok(true)
    }

    // fn bind_as(&mut self, username: &str, password: &str) -> PyResult<()> {
    //     // let mut ldaps = origen::ldaps();
    //     // let ldap = ldaps._get_mut(&self.name)?;
    //     // ldap.bind_as(username, password)?;
    //     // Ok(true)
    //     Ok(self.with_mut_inner_ldap( |om_ldap| {
    //         // let mut new_ldap = om_ldap.standalone()?;
    //         // new_ldap.bind_as(username, password)?;
    //         om_ldap.bind_as(username, password)
    //     })?)
    // }

    fn validate_credentials(&self, username: &str, password: &str) -> PyResult<bool> {
        // Ok(origen::utility::ldap::LDAPs::try_password(
        //     &self.name, username, password,
        // )?)
        Ok(self.with_inner_ldap( |om_ldap| {
            om_ldap.try_password(username, password)
        })?)
    }
}
