use super::data::DatasetConfig;
use super::user::{PopulateUserReturn, SessionConfig, User};
use crate::{Outcome, Result, USERS};
use indexmap::IndexMap;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
// TODO rename to _helpers?
use crate::_utility::validate_input_list;
use crate::prelude::session_store::*;
use crate::utils::encryption;
use std::collections::HashMap;
use std::path::PathBuf;

lazy_static! {
    pub static ref DEFAULT_DATASET_KEY: &'static str = "__origen__default__";
}

macro_rules! new_default_datasets {
    () => {{
        let mut i = IndexMap::new();
        i.insert(DEFAULT_DATASET_KEY.to_string(), DatasetConfig::default());
        i
    }};
}

macro_rules! new_default_lookup_hierarchy {
    () => {{
        vec![DEFAULT_DATASET_KEY.to_string()]
    }};
}

pub fn users<'a>() -> RwLockReadGuard<'a, Users> {
    USERS.read().unwrap()
}

pub fn users_mut<'a>() -> RwLockWriteGuard<'a, Users> {
    USERS.write().unwrap()
}

pub fn with_users<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&Users) -> Result<T>,
{
    let users = USERS.read().unwrap();
    func(&users)
}

pub fn with_users_mut<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&mut Users) -> Result<T>,
{
    let mut users = USERS.write().unwrap();
    func(&mut users)
}

pub fn with_user<T, F>(id: &str, mut func: F) -> Result<T>
where
    F: FnMut(&User) -> Result<T>,
{
    let users = USERS.read().unwrap();
    let user = users.user(id)?;
    func(user)
}

pub fn with_user_mut<T, F>(id: &str, mut func: F) -> Result<T>
where
    F: FnMut(&mut User) -> Result<T>,
{
    let mut users = USERS.write().unwrap();
    let user = users.user_mut(id)?;
    func(user)
}

pub fn with_user_or_current<T, F, S>(id: Option<S>, mut func: F) -> Result<T>
where
    F: FnMut(&User) -> Result<T>,
    S: AsRef<str>,
{
    if let Some(u_id) = id {
        let users = USERS.read().unwrap();
        let user = users.user(u_id.as_ref())?;
        func(user)
    } else {
        with_current_user(func)
    }
}

pub fn get_current_user_id() -> Result<Option<String>> {
    let u = users();
    Ok(match u.get_current_id()? {
        Some(id) => Some(id.to_string()),
        None => None,
    })
}

pub fn require_current_user_id() -> Result<String> {
    let u = users();
    Ok(u.current_id()?.to_string())
}

pub fn with_current_user<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&User) -> Result<T>,
{
    let users = USERS.read().unwrap();
    let current = users.current_user()?;
    func(current)
}

pub fn get_current_user_home_dir() -> Result<Option<PathBuf>> {
    with_current_user(|u| u.home_dir())
}

pub fn require_current_user_home_dir() -> Result<PathBuf> {
    with_current_user(|u| u.require_home_dir())
}

pub fn get_current_user_email() -> Result<Option<String>> {
    with_current_user(|u| u.get_email())
}

pub fn require_current_user_email() -> Result<String> {
    with_current_user(|u| u.require_email())
}

// TODO move tests from origen to test these
// TEST_NEEDED
pub fn with_current_user_session<T, F>(namespace: Option<String>, func: F) -> Result<T>
where
    F: FnMut(&Sessions, &SessionGroup, &SessionStore) -> Result<T>,
{
    let users = USERS.read().unwrap();
    let current = users.current_user()?;
    current.with_session(namespace, func)
}

pub fn get_initial_user_id() -> Result<Option<String>> {
    let u = users();
    Ok(match u.get_initial_id()? {
        Some(id) => Some(id.to_string()),
        None => None,
    })
}

pub fn add_user(id: &str, auto_populate: Option<bool>) -> Result<Option<PopulateUserReturn>> {
    Users::add_user(id, auto_populate)
}

pub fn set_current_user(id: &str) -> Result<bool> {
    let mut u = users_mut();
    u.set_current_user(id)
}

pub fn clear_current_user() -> Result<bool> {
    let mut u = users_mut();
    u.clear_current_user()
}

pub fn try_lookup_current_user() -> Result<String> {
    Users::try_lookup_current_user()
}

pub fn try_lookup_and_set_current_user() -> Result<(String, Option<Option<PopulateUserReturn>>)> {
    Users::try_lookup_and_set_current_user()
}

pub fn unload(continue_on_user_unload_fail: bool) -> Result<()> {
    let mut u = users_mut();
    u.unload(continue_on_user_unload_fail)
}

#[derive(Debug, Clone, Default)]
pub struct PopulateUsersReturn {
    outcomes: IndexMap<String, PopulateUserReturn>,

    // Cache the failed and errored outcomes
    failed_datasets: IndexMap<String, Vec<String>>,
    errored_datasets: IndexMap<String, Vec<String>>,
}
type ErrorFailedOutcomeReturn<'a> = IndexMap<&'a String, IndexMap<&'a String, &'a Outcome>>;

impl PopulateUsersReturn {
    pub fn outcomes(&self) -> &IndexMap<String, PopulateUserReturn> {
        &self.outcomes
    }

    pub fn all_succeeded(&self) -> bool {
        self.failed_datasets.is_empty() && self.errored_datasets.is_empty()
    }

    pub fn succeeded(&self) -> bool {
        self.all_succeeded()
    }

    pub fn failed(&self) -> bool {
        !self.failed_datasets.is_empty()
    }

    pub fn errored(&self) -> bool {
        !self.errored_datasets.is_empty()
    }

    pub fn failed_datasets(&self) -> &IndexMap<String, Vec<String>> {
        &self.failed_datasets
    }

    pub fn failed_outcomes(&self) -> ErrorFailedOutcomeReturn {
        self.failed_datasets
            .iter()
            .map(|(uid, _)| {
                (
                    uid,
                    self.outcomes
                        .get(uid)
                        .expect(&format!(
                            "Expected PopulateUsersReturn to have outcome for {}",
                            uid
                        ))
                        .failed_outcomes(),
                )
            })
            .collect::<ErrorFailedOutcomeReturn>()
    }

    pub fn errored_datasets(&self) -> &IndexMap<String, Vec<String>> {
        &self.errored_datasets
    }

    pub fn errored_outcomes(&self) -> ErrorFailedOutcomeReturn {
        self.errored_datasets
            .iter()
            .map(|(uid, _)| {
                (
                    uid,
                    self.outcomes
                        .get(uid)
                        .expect(&format!(
                            "Expected PopulateUsersReturn to have outcome for {}",
                            uid
                        ))
                        .errored_outcomes(),
                )
            })
            .collect::<ErrorFailedOutcomeReturn>()
    }

    pub fn insert(
        &mut self,
        id: &str,
        pop_user_rtn: PopulateUserReturn,
    ) -> Option<PopulateUserReturn> {
        if pop_user_rtn.failed() {
            self.failed_datasets
                .insert(id.to_owned(), pop_user_rtn.failed_datasets().to_owned());
        }
        if pop_user_rtn.errored() {
            self.errored_datasets
                .insert(id.to_owned(), pop_user_rtn.errored_datasets().to_owned());
        }
        self.outcomes.insert(id.to_owned(), pop_user_rtn)
    }
}

#[allow(non_snake_case)]
pub struct Users {
    users: IndexMap<String, User>,
    current_id: Option<String>,
    initial_id: Option<String>,
    default_datasets: IndexMap<String, DatasetConfig>,
    default_data_lookup_hierarchy: Vec<String>,
    default_motive_mapping: IndexMap<String, String>,
    default_session_config: SessionConfig,
    default_auto_populate: Option<bool>,
    default_should_validate_passwords: Option<bool>,
    default_roles: Vec<String>,
    uid_cnt: usize,
    password_encryption_key__byte_str: String,
    password_encryption_nonce__byte_str: String,
}

impl Users {
    pub fn get_current_id(&self) -> Result<Option<&String>> {
        Ok(self.current_id.as_ref())
    }

    pub fn get_current_user(&self) -> Result<Option<&User>> {
        Ok(match self.current_id.as_ref() {
            Some(id) => Some(
                // Unwrapping first here to ensure the user is available. If not, something has gone wrong.
                self.users.get(id).unwrap(),
            ),
            None => None,
        })
    }

    pub fn get_current_user_mut(&mut self) -> Result<Option<&mut User>> {
        Ok(match self.current_id.as_ref() {
            Some(id) => Some(self.users.get_mut(id).unwrap()),
            None => None,
        })
    }

    pub fn get_user(&self, id: &str) -> Result<Option<&User>> {
        Ok(self.users.get(id))
    }

    pub fn get_user_mut(&mut self, id: &str) -> Result<Option<&mut User>> {
        Ok(self.users.get_mut(id))
    }

    pub fn get_initial_id(&self) -> Result<Option<&String>> {
        Ok(self.initial_id.as_ref())
    }

    pub fn get_initial_user(&self) -> Result<Option<&User>> {
        match self.initial_id.as_ref() {
            Some(id) => {
                if let Some(u) = self.users.get(id) {
                    Ok(Some(u))
                } else {
                    bail!("Initial user '{}' not found in existing users")
                }
            }
            None => Ok(None),
        }
    }

    // Sister functions to the above that will return a standard error message
    // if the requested is not found, or if the current user is not set

    pub fn current_id(&self) -> Result<&str> {
        match self.current_id.as_ref() {
            Some(id) => Ok(id),
            None => bail!("No current user has been set!"),
        }
    }

    pub fn current_user(&self) -> Result<&User> {
        Ok(self.users.get(self.current_id()?).unwrap())
    }

    pub fn current_user_mut(&mut self) -> Result<&mut User> {
        let id = self.current_id()?.to_string();
        Ok(self.users.get_mut(&id).unwrap())
    }

    pub fn user(&self, u: &str) -> Result<&User> {
        if let Some(user) = self.users.get(u).as_ref() {
            Ok(&user)
        } else {
            bail!("No user '{}' has been added", u)
        }
    }

    pub fn user_mut(&mut self, u: &str) -> Result<&mut User> {
        if let Some(user) = self.users.get_mut(u) {
            Ok(user)
        } else {
            bail!("No user '{}' has been added", u)
        }
    }

    // Other Functions

    pub fn users(&self) -> &IndexMap<String, User> {
        &self.users
    }

    pub fn set_current_user(&mut self, id: &str) -> Result<bool> {
        if !self.users.contains_key(id) {
            bail!(
                "Cannot set current user with id '{}'. User has not been added yet!",
                id
            );
        }

        let rtn = match self.current_id.as_ref() {
            Some(cid) => id != cid,
            None => true,
        };
        self.current_id = Some(id.to_string());
        if self.initial_id.is_none() {
            self.initial_id = Some(id.to_string());
        }
        Ok(rtn)
    }

    pub fn clear_current_user(&mut self) -> Result<bool> {
        let rtn = self.current_id.is_some();
        self.current_id = None;
        Ok(rtn)
    }

    pub fn try_lookup_current_user() -> Result<String> {
        // TODO see about wrapping function calls like this (optional frontend functions)
        let fe_res = crate::with_optional_frontend(|f| {
            if let Some(fe) = f {
                if let Some(result) = fe.lookup_current_user() {
                    if let Some(u_id) = result? {
                        return Ok(Some(u_id));
                    }
                }
            }
            Ok(None)
        })?;
        if let Some(r) = fe_res {
            Ok(r.to_string())
        } else {
            super::whoami()
        }
    }

    pub fn try_lookup_and_set_current_user() -> Result<(String, Option<Option<PopulateUserReturn>>)>
    {
        let id = Self::try_lookup_current_user()?;
        let need_to_add = with_users(|users| Ok(!users.users.contains_key(&id)))?;

        let pop_return: Option<Option<PopulateUserReturn>>;
        if need_to_add {
            pop_return = Some(Self::add_user_option_less(&id)?);
        } else {
            pop_return = None;
        }

        with_users_mut(|users| users.set_current_user(&id))?;
        Ok((id, pop_return))
    }

    fn add(&mut self, id: &str, auto_populate: Option<bool>) -> Result<()> {
        if self.users.contains_key(id) {
            bail!("User '{}' has already been added", id)
        } else {
            self.users.insert(
                id.to_string(),
                User::new(
                    id,
                    &self,
                    // TODO support password cache option
                    None,
                    self.uid_cnt,
                    auto_populate
                        .map_or_else(|| self.default_auto_populate.to_owned(), |ap| Some(ap)),
                )?,
            );
            self.uid_cnt += 1;
            Ok(())
        }
    }

    pub fn add_user(id: &str, auto_populate: Option<bool>) -> Result<Option<PopulateUserReturn>> {
        log_trace!("Adding user '{}'", id);

        let mut roles: Vec<String> = vec![];
        with_users_mut(|users| {
            roles = users.default_roles.to_owned();
            users.add(id, auto_populate)
        })?;

        with_user(id, |u| {
            if !roles.is_empty() {
                u.add_roles(&roles)?;
            }
            u.autopopulate()
        })
    }

    // Adds an user, inheriting all options from the global `users`
    pub fn add_user_option_less(id: &str) -> Result<Option<PopulateUserReturn>> {
        Self::add_user(id, None)
    }

    pub fn remove(&mut self, id: &str) -> Result<bool> {
        // TODO need call unload on user
        match self.users.remove(id) {
            Some(user) => {
                let mut retn = false;
                if let Some(cid) = self.current_id.as_ref() {
                    if cid == id {
                        self.current_id = None;
                        retn = true;
                    }
                }
                user.unload()?;
                Ok(retn)
            }
            None => bail!("Cannot remove nonexistent user '{}'", id),
        }
    }

    pub fn initial_id(&self) -> Result<&str> {
        match self.initial_id.as_ref() {
            Some(id) => Ok(id),
            None => bail!("The current user has yet to be set. An initial user is only available after the current user has been set at least once.")
        }
    }

    pub fn initial_user(&self) -> Result<&User> {
        match self.get_initial_user()? {
            Some(u) => Ok(u),
            None => bail!("The current user has yet to be set. An initial user is only available after the current user has been set at least once.")
        }
    }

    pub fn unload(&mut self, _continue_on_user_unload_fail: bool) -> Result<()> {
        // Note: Purposefully leave the uid_cnt unchanged

        // TODO use continue_on_user_unload_fail and add tests
        for (_id, u) in self.users().iter() {
            u.unload()?;
        }

        self.users = IndexMap::new();
        self.current_id = None;
        self.initial_id = None;
        self.default_datasets = new_default_datasets!();
        self.default_data_lookup_hierarchy = new_default_lookup_hierarchy!();
        self.default_motive_mapping = IndexMap::new();
        self.default_session_config = SessionConfig::new();
        self.default_auto_populate = None;
        self.default_should_validate_passwords = None;
        self.default_roles = vec![];
        self.password_encryption_key__byte_str =
            encryption::default_encryption_key__byte_str().to_string();
        self.password_encryption_nonce__byte_str =
            encryption::default_encryption_nonce__byte_str().to_string();
        Ok(())
    }

    pub fn default_datasets(&self) -> &IndexMap<String, DatasetConfig> {
        &self.default_datasets
    }

    pub fn require_default_dataset(&self, dataset: &str) -> Result<&DatasetConfig> {
        match &self.default_datasets.get(dataset) {
            Some(ds) => Ok(ds),
            None => bail!("Users has not had a default dataset '{dataset}' added yet!"),
        }
    }

    pub fn default_datakeys(&self) -> Vec<&String> {
        self.default_datasets
            .keys()
            .into_iter()
            .collect::<Vec<&String>>()
    }

    /// Overrides the default datakey/dataset, removing the origen default and replacing it with this one.
    /// This can only be done prior to adding any users or auxillary datasets, otherwise an error is thrown
    pub fn override_default_dataset(
        &mut self,
        new_ds: &str,
        new_ds_config: DatasetConfig,
    ) -> Result<()> {
        if self.users.len() > 0 {
            bail!(
                "The default dataset can only be overridden prior to adding any users. Found users: {}",
                self.users.keys().map(|k| format!("'{}'", k)).collect::<Vec<String>>().join(", ")
            );
        } else if self.default_datasets.len() != 1 {
            let def_ds = self
                .default_datasets
                .first()
                .expect("Something has gone wrong and Users.default_datasets is empty")
                .0;
            bail!(
                "The default dataset can only be overridden prior to adding any additional datasets. Found additional datasets: {}",
                // TODO need to remove default from existing
                self.default_datasets.keys().filter(|k| *k != def_ds).map(|k| format!("'{}'", k)).collect::<Vec<String>>().join(", ")
            );
        }

        self.default_datasets = IndexMap::new();
        self.default_datasets
            .insert(new_ds.to_string(), new_ds_config);
        self.default_data_lookup_hierarchy = vec![new_ds.to_string()];
        Ok(())
    }

    pub fn default_data_lookup_hierarchy(&self) -> &Vec<String> {
        &self.default_data_lookup_hierarchy
    }

    pub fn set_default_data_lookup_hierarchy(&mut self, hierarchy: Vec<String>) -> Result<()> {
        // Check that each item in hierarchy is valid and that there are no duplicates ::<String, String, Vec<String>, Vec<String>>
        validate_input_list(
            hierarchy.iter().collect::<Vec<&String>>(),
            Some(self.default_datakeys()),
            false,
            Some(&super::duplicate_dataset_hierarchy_closure),
            Some(&super::invalid_dataset_hierarchy_closure),
        )?;
        self.default_data_lookup_hierarchy = hierarchy;
        Ok(())
    }

    /// Registers a new default dataset but does not add it to the hierarchy
    pub fn register_default_dataset(&mut self, name: &str, config: DatasetConfig) -> Result<()> {
        if self.default_datasets.contains_key(name) {
            bail!("A dataset '{}' is already present", name);
        }

        self.default_datasets.insert(name.to_string(), config);
        Ok(())
    }

    /// Registers a new default dataset and adds it to the hierarchy.
    /// 'as_topmost' = true -> inserts at the beginning (highest) hierarchy
    /// 'as_topmost' = false -> inserts at the lowest point in the hierarchy
    pub fn add_default_dataset(
        &mut self,
        name: &str,
        config: DatasetConfig,
        as_topmost: bool,
    ) -> Result<()> {
        self.register_default_dataset(name, config)?;
        if as_topmost {
            self.default_data_lookup_hierarchy
                .insert(0, name.to_string());
        } else {
            self.default_data_lookup_hierarchy.push(name.to_string());
        }
        Ok(())
    }

    pub fn motive_mapping(&self) -> &IndexMap<String, String> {
        &self.default_motive_mapping
    }

    pub fn add_motive(
        &mut self,
        motive: String,
        dataset: String,
        replace_existing: bool,
    ) -> Result<Option<String>> {
        if !self.default_datasets.contains_key(&dataset) {
            bail!(
                "Cannot add motive corresponding to nonexistent dataset '{}'",
                &dataset
            );
        }
        if !replace_existing {
            if let Some(ds) = self.default_motive_mapping.get(&motive) {
                bail!(
                    "Motive '{}' already corresponds to dataset '{}'. Use the 'replace_existing' option to update the motive",
                    motive,
                    ds
                );
            }
        }
        Ok(self.default_motive_mapping.insert(motive, dataset))
    }

    pub fn dataset_for(&self, motive: &str) -> Result<Option<&String>> {
        Ok(self.default_motive_mapping.get(motive))
    }

    pub fn populate(
        &self,
        repopulate: bool,
        continue_on_error: bool,
        stop_on_failure: bool,
    ) -> Result<PopulateUsersReturn> {
        let mut rtn = PopulateUsersReturn::default();
        for (id, u) in self.users.iter() {
            rtn.insert(
                &id,
                u.populate(repopulate, continue_on_error, stop_on_failure)?,
            );
        }
        Ok(rtn)
    }

    pub fn default_auto_populate(&self) -> &Option<bool> {
        &self.default_auto_populate
    }

    pub fn set_default_auto_populate(&mut self, set_to: Option<bool>) -> () {
        self.default_auto_populate = set_to;
    }

    pub fn default_session_config(&self) -> &SessionConfig {
        &self.default_session_config
    }

    pub fn default_session_config_mut(&mut self) -> &mut SessionConfig {
        &mut self.default_session_config
    }

    pub fn password_encryption_key(&self) -> &str {
        &self.password_encryption_key__byte_str
    }

    pub fn password_encryption_nonce(&self) -> &str {
        &self.password_encryption_nonce__byte_str
    }

    pub fn default_should_validate_passwords(&self) -> &Option<bool> {
        &self.default_should_validate_passwords
    }

    pub fn set_default_should_validate_passwords(&mut self, set_to: Option<bool>) -> () {
        self.default_should_validate_passwords = set_to;
    }

    pub fn default_roles(&self) -> Result<&Vec<String>> {
        Ok(&self.default_roles)
    }

    pub fn set_default_roles<S: AsRef<str>>(&mut self, roles: &Vec<S>) -> Result<()> {
        let rls = roles
            .iter()
            .map(|r| r.as_ref().to_string())
            .collect::<Vec<String>>();
        validate_input_list(&rls, None::<&Vec<String>>, false, None, None)?;
        self.default_roles = rls;
        Ok(())
    }

    pub fn clear_default_roles(&mut self) -> Result<()> {
        self.default_roles = vec![];
        Ok(())
    }

    /// Return all roles
    pub fn roles(&self) -> Result<Vec<String>> {
        Ok(self
            .users_by_role(None)?
            .keys()
            .map(|r| r.to_owned())
            .collect::<Vec<String>>())
    }

    pub fn users_by_role(
        &self,
        filter: Option<&dyn Fn(&User, &String) -> bool>,
    ) -> Result<HashMap<String, Vec<String>>> {
        let mut roles: HashMap<String, Vec<String>> = HashMap::new();
        for (n, u) in &self.users {
            for user_role in u.roles()?.iter() {
                if filter.map_or(true, |f| f(&u, user_role)) {
                    if let Some(r) = roles.get_mut(user_role) {
                        r.push(n.to_owned());
                    } else {
                        roles.insert(user_role.to_owned(), vec![n.to_owned()]);
                    }
                }
            }
        }
        Ok(roles)
    }
}

impl Default for Users {
    fn default() -> Self {
        Self {
            users: IndexMap::new(),
            current_id: None,
            initial_id: None,
            default_datasets: new_default_datasets!(),
            default_data_lookup_hierarchy: new_default_lookup_hierarchy!(),
            default_motive_mapping: IndexMap::new(),
            default_session_config: SessionConfig::new(),
            default_auto_populate: None,
            default_should_validate_passwords: None,
            default_roles: vec![],
            uid_cnt: 0,
            password_encryption_key__byte_str: encryption::default_encryption_key__byte_str()
                .to_string(),
            password_encryption_nonce__byte_str: encryption::default_encryption_nonce__byte_str()
                .to_string(),
        }
    }
}
