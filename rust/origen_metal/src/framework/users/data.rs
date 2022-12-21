use super::password_cache_options::PASSWORD_KEY;
use super::User;
use crate::frontend::FeatureReturn;
use crate::{frontend, Outcome};
use crate::{Result, TypedValueMap};
use indexmap::IndexMap;
use std::path::PathBuf;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct DatasetConfig {
    pub category: Option<String>,
    pub data_store: Option<String>,
    pub auto_populate: Option<bool>,

    // Password config options
    pub should_validate_password: Option<bool>,
}

impl DatasetConfig {
    pub fn new(
        category: Option<String>,
        data_store: Option<String>,
        auto_populate: Option<bool>,
        should_validate_password: Option<bool>,
    ) -> Result<Self> {
        Ok(Self {
            category: category,
            data_store: data_store,
            auto_populate: auto_populate,
            should_validate_password: should_validate_password,
        })
    }

    pub fn should_auto_populate(&self) -> bool {
        self.auto_populate.unwrap_or(false)
    }

    pub fn should_validate_password(&self) -> bool {
        self.should_validate_password.unwrap_or(false)
    }

    pub fn has_empty_populate_config(&self) -> bool {
        self.category.is_none() && self.data_store.is_none()
    }
}

impl From<DatasetConfig> for TypedValueMap {
    fn from(ds: DatasetConfig) -> TypedValueMap {
        let mut tvm = TypedValueMap::new();
        tvm.insert("category", ds.category);
        tvm.insert("data_store", ds.data_store);
        tvm.insert("auto_populate", ds.auto_populate);
        tvm.insert("should_validate_password", ds.should_validate_password);
        tvm
    }
}

impl From<&DatasetConfig> for TypedValueMap {
    fn from(ds: &DatasetConfig) -> TypedValueMap {
        let mut tvm = TypedValueMap::new();
        tvm.insert("category", ds.category.as_ref());
        tvm.insert("data_store", ds.data_store.as_ref());
        tvm.insert("auto_populate", ds.auto_populate.as_ref());
        tvm.insert(
            "should_validate_password",
            ds.should_validate_password.as_ref(),
        );
        tvm
    }
}

/// Struct to hold user data for a given dataset.
/// At least in the backend, each data instance is independent
/// with an dataset-dependent features (e.g., username, password, retrieval)
/// handled by the owning User struct.
#[derive(Default)]
pub struct Data {
    pub dataset_name: String,
    pub password: Option<String>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,

    pub other: TypedValueMap,
    pub home_dir: Option<PathBuf>,
    // Will be set after trying to get a missing name, e.g. from the
    // Git config to differentiate between an name which has not been
    // looked up and name which has been looked up but which could not
    // be found.
    pub name_tried: bool,
    pub email: Option<String>,
    pub email_tried: bool,

    // Authentication
    pub authenticated: bool,
    pub populated: bool,
    pub populate_attempted: bool,

    pub roles: Vec<Roles>,

    config: DatasetConfig,
}

impl Data {
    pub fn new(dataset_name: &str, config: &DatasetConfig) -> Self {
        Self {
            dataset_name: dataset_name.to_string(),
            config: config.clone(),
            ..Default::default()
        }
    }

    pub fn get_display_name(&self) -> Option<String> {
        if let Some(n) = &self.display_name {
            Some(n.clone())
        } else if self.first_name.is_some() && self.last_name.is_some() {
            Some(format!(
                "{} {}",
                self.first_name.as_ref().unwrap().to_string(),
                self.last_name.as_ref().unwrap().to_string()
            ))
        } else if let Some(n) = &self.username {
            Some(n.clone())
        } else {
            None
        }
    }

    pub fn require_home_dir(&self, parent: &User) -> Result<PathBuf> {
        self.home_dir.as_ref().map_or_else(
            || {
                bail!(
                "Required a home directory for user '{}' and dataset '{}', but none has been set",
                parent.id(),
                self.dataset_name
            )
            },
            |d| Ok(d.to_owned()),
        )
    }

    pub fn password_key(&self) -> String {
        format!("{}{}", PASSWORD_KEY, self.dataset_name)
    }

    pub fn clear_cached_password(&mut self, parent: &User) -> Result<()> {
        let k = self.password_key();
        log_trace!("Clearing cached password for {}", k);
        self.password = None;
        self.authenticated = false;
        parent
            .password_cache_option()
            .clear_cached_password(parent, self)?;
        Ok(())
    }

    pub fn set_should_validate_password(&mut self, new_val: Option<bool>) -> () {
        self.config.should_validate_password = new_val;
    }

    pub fn config(&self) -> &DatasetConfig {
        &self.config
    }

    pub fn password_needs_validation(&self) -> bool {
        self.config.should_validate_password() && !self.authenticated
    }

    pub fn require_data_source_for(&self, op: &str, user_id: &str) -> Result<(&str, &str)> {
        match &self.config.data_store {
            Some(ds) => match &self.config.category {
                Some(cat) => Ok((cat, ds)),
                None => bail!(
                    "Requested operation '{}' for user id '{}' requires that dataset '{}' contains a data source, but no 'category' was provided.",
                    op,
                    user_id,
                    self.dataset_name
                )
            },
            None => bail!(
                "Requested operation '{}' for user id '{}' requires that dataset '{}' contains a data source, but no 'data source' was provided.",
                op,
                user_id,
                self.dataset_name
            )
        }
    }

    pub fn populate_succeeded(&self) -> Option<bool> {
        if self.populate_attempted {
            if self.populated {
                Some(true)
            } else {
                Some(false)
            }
        } else {
            None
        }
    }

    pub fn populate_failed(&self) -> Option<bool> {
        match self.populate_succeeded() {
            Some(res) => Some(!res),
            None => None,
        }
    }

    // TODO add a "can populate" method
    pub fn has_empty_populate_config(&self) -> bool {
        self.config.has_empty_populate_config()
    }

    pub fn should_auto_populate(&self) -> bool {
        self.config.should_auto_populate()
    }

    pub fn populate(
        user: &User,
        ds_name: &str,
        repopulate: bool,
        continue_on_error: bool,
        stop_on_failure: bool,
    ) -> Result<Option<Outcome>> {
        // Grab the DS for writing, releasing it at the end
        let lookup: (String, String);
        {
            let t = user.with_dataset_mut(ds_name, |ds| {
                if ds.populated && !repopulate {
                    return Ok(None);
                }

                ds.populate_attempted = true;

                // Look up the data source
                let t = ds.require_data_source_for("populate", user.id())?;
                Ok(Some((t.0.to_string(), t.1.to_string())))
            })?;
            match t {
                Some(l) => lookup = l,
                None => return Ok(None),
            }
        }

        // Allow the DS to be free for population
        // Let the populate callback handle the population as it wants
        let f = frontend::require()?;
        let r = f.with_data_store(&lookup.0, &lookup.1, |dstore| {
            match dstore.populate_user(user.id(), ds_name) {
                Ok(fr) => Ok(fr),
                Err(e) => {
                    if continue_on_error {
                        let mut oc = Outcome::new_err();
                        oc.set_msg(e.to_string());
                        oc.inferred = Some(true);
                        let fr = FeatureReturn::new(Ok(oc));
                        return Ok(fr);
                    } else {
                        Err(e)
                    }
                }
            }
        })?;
        let o = match r.outcome() {
            Ok(oc) => oc,
            Err(e) => {
                if continue_on_error {
                    let mut oc = Outcome::new_err();
                    oc.set_msg(e.to_string());
                    return Ok(Some(oc));
                } else {
                    return Err(e);
                }
            }
        };

        user.with_dataset_mut(ds_name, |ds| {
            if o.errored() {
                if continue_on_error {
                    Ok(Some(o.to_owned()))
                } else {
                    bail!(
                        "Errors encountered populating dataset '{}' for user '{}': {}",
                        ds_name,
                        user.id(),
                        o.msg_or_default()
                    )
                }
            } else if o.failed() {
                if stop_on_failure {
                    bail!(
                        "Failed to populate dataset '{}' for user '{}': {}",
                        ds_name,
                        user.id(),
                        o.msg_or_default()
                    )
                } else {
                    Ok(Some(o.to_owned()))
                }
            } else {
                ds.populated = true;
                Ok(Some(o.to_owned()))
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum Roles {
    User(Option<IndexMap<String, crate::TypedValue>>),
}

impl std::default::Default for Roles {
    fn default() -> Self {
        Self::User(Option::None)
    }
}
