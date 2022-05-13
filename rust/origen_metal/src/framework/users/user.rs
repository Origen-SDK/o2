use crate::_utility::{validate_input_list, unsorted_dedup};
use crate::{Result, Outcome, OutcomeState};
use std::collections::HashMap;
use std::path::{PathBuf};
use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use super::data::{Data, DatasetConfig};
use super::password_cache_options::PasswordCacheOptions;
use indexmap::IndexMap;
use crate::utils::file::FilePermissions;
use crate::prelude::session_store::*;

pub const DEFAULT_DATASET_KEY: &str = "__origen__default__";
pub const DEFAULT_USER_SESSION_PATH_OFFSET: &str = "./.o2/.session";
pub const DEFAULT_USER_SESSION_GROUP_NAME: &str = "__user__";
pub const DEFAULT_USER_SESSION_STORE_NAME: &str = "__user__";

lazy_static! {
    pub static ref DEFAULT_DATASET_CONFIG: HashMap<String, HashMap<String, String>> = {
        let mut h: HashMap<String, HashMap<String, String>> = HashMap::new();
        h.insert(DEFAULT_DATASET_KEY.to_string(), {
            let mut c: HashMap<String, String> = HashMap::new();
            c.insert("data_source".to_string(), "git".to_string());
            c.insert("auto_populate".to_string(), "false".to_string());
            c
        });
        h
    };

    static ref SORRY_PW: &'static str = "Sorry, that password is not correct";
}

pub fn with_user_dataset<T, F>(user: Option<&str>, dataset: &str, func: F) -> Result<T>
where
    F: FnMut(&Data) -> Result<T>,
{
    let urs = crate::users();
    let u;
    if let Some(uname) = user {
        u = urs.user(uname)?;
    } else {
        u = urs.current_user()?;
    }
    u.with_dataset(dataset, func)
}

pub fn with_user_dataset_mut<T, F>(user: Option<&str>, dataset: &str, func: F) -> Result<T>
where
    F: FnMut(&mut Data) -> Result<T>,
{
    let urs = crate::users();
    let u;
    if let Some(uname) = user {
        u = urs.user(uname)?;
    } else {
        u = urs.current_user()?;
    }
    u.with_dataset_mut(dataset, func)
}

pub fn add_dataset_to_user(id: &str, dataset: &str, config: DatasetConfig, replace_existing: bool, as_topmost: bool) -> Result<Option<Outcome>> {
    User::add_dataset(id, dataset, config, replace_existing, as_topmost)
}

pub fn register_dataset_with_user(id: &str, dataset: &str, config: DatasetConfig, replace_existing: bool) -> Result<Option<Outcome>> {
    User::register_dataset(id, dataset, config, replace_existing)
}

/// Temporarily run some function with 'new_top' being the highest priority datasets
pub fn with_user_hierarchy<T, F>(user: Option<&str>, new_top: &Vec<String>, func: F) -> Result<T>
where
    F: Fn(&User) -> Result<T>,
{
    let mut urs = crate::users_mut();
    let u;
    if let Some(uname) = user {
        u = urs.user_mut(uname)?;
    } else {
        u = urs.current_user_mut()?;
    }

    let old_hierarchy = u.data_lookup_hierarchy.clone();
    let mut new_hierarchy = new_top.to_vec();
    new_hierarchy.extend(old_hierarchy.clone());
    unsorted_dedup(&mut new_hierarchy);
    u.set_data_lookup_hierarchy(new_hierarchy)?;
    let res = func(u);
    u.data_lookup_hierarchy = old_hierarchy;
    res
}

#[derive(Debug, Clone, Default)]
pub struct PopulateUserReturn {
    outcomes: IndexMap<String, Option<Outcome>>,

    // Cache the failed and errored dataset names
    failed_datasets: Vec<String>,
    errored_datasets: Vec<String>
}

impl PopulateUserReturn {
    pub fn outcomes(&self) -> &IndexMap<String, Option<Outcome>> {
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

    pub fn failed_datasets(&self) -> &Vec<String> {
        &self.failed_datasets
    }

    pub fn failed_outcomes(&self) -> IndexMap<&String, &Outcome> {
        self.failed_datasets.iter().map( |d| (d, self.outcomes.get(d).expect(&format!("Expected PopulateUserReturn to have outcome for {}", d)).as_ref().expect(&format!("Expected outcome for dataset {} to not be None", d)))).collect::<IndexMap<&String, &Outcome>>()
    }

    pub fn errored_datasets(&self) -> &Vec<String> {
        &self.errored_datasets
    }

    pub fn errored_outcomes(&self) -> IndexMap<&String, &Outcome> {
        self.errored_datasets.iter().map( |d| (d, self.outcomes.get(d).expect(&format!("Expected PopulateUserReturn to have outcome for {}", d)).as_ref().expect(&format!("Expected outcome for dataset {} to not be None", d)))).collect::<IndexMap<&String, &Outcome>>()
    }

    pub fn insert(&mut self, dataset: &str, outcome: Option<Outcome>) -> Option<Option<Outcome>> {
        match outcome {
            Some(oc) => {
                match oc.state {
                    OutcomeState::Success(_, _) => {},
                    OutcomeState::Fail(_, _) => self.failed_datasets.push(dataset.to_owned()),
                    OutcomeState::Error(_, _) => self.errored_datasets.push(dataset.to_owned())
                }
                self.outcomes.insert(dataset.to_owned(), Some(oc.to_owned()))
            },
            None => self.outcomes.insert(dataset.to_owned(), None)
        }
    }
}

#[derive(Debug, Default)]
struct PopulateStatus {
    populating: RwLock<bool>,
    lock: Mutex<bool>,
}

impl PopulateStatus {
    pub fn while_populating<T, F>(&self, mut func: F) -> T
    where
        F: FnMut() -> T,
    {
        let _lock = self.lock.lock().unwrap();
        {
            let mut popping = self.populating.write().unwrap();
            *popping = true;
        }
        let result = func();

        {
            let mut popping = self.populating.write().unwrap();
            *popping = false;
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub root: Option<PathBuf>,
    pub offset: Option<PathBuf>,
    pub file_permissions: FilePermissions,
}

impl SessionConfig {
    pub fn new() -> Self {
        Self {
            root: None,
            offset: Some(PathBuf::from(DEFAULT_USER_SESSION_PATH_OFFSET)),
            file_permissions: FilePermissions::Private,
        }
    }

    pub fn to_sg_name(id: &str) -> String {
        format!("__user__{}__", id)
    }

    pub fn set_offset(&mut self, new_offset: Option<PathBuf>) -> Result<()> {
        match new_offset {
            Some(o) => {
                if o.is_absolute() {
                    bail!("Absolute offsets are not allowed in a user's session config (given: {})", o.display());
                }
                self.offset = Some(o)
            },
            None => self.offset = None
        }
        Ok(())
    }

    pub fn resolved_root(&self, user: &User) -> Result<PathBuf> {
        let mut rr: PathBuf = PathBuf::new();
        match self.root.as_ref() {
            Some(r) => rr.push(r),
            None => if let Some(d) = user.home_dir()? {
                rr.push(d);
            }
        }
        if let Some(o) = self.offset.as_ref() {
            rr.push(o);
        }
        Ok(rr)
    }

    pub fn resolved_path(&self, user: &User) -> Result<PathBuf> {
        let mut rr = self.resolved_root(user)?;
        rr.push(format!("{}{}__", DEFAULT_USER_SESSION_GROUP_NAME, user.id()));
        Ok(rr)
    }
}

pub struct User {
    // All user data is stored behind a RW lock so that it can be lazily loaded
    // from the environment and cached behind the scenes
    data: IndexMap<String, RwLock<Data>>,
    data_lookup_hierarchy: Vec<String>,
    password_semaphore: Mutex<u8>,
    id: String,
    password_cache_option: PasswordCacheOptions,
    motive_mapping: IndexMap<String, String>,
    populate_status: PopulateStatus,
    session_config: RwLock<SessionConfig>,
    uid_num: usize,
}

impl User {
    pub fn top_datakey(&self) -> Result<&str> {
        if let Some(key) = self.data_lookup_hierarchy.first() {
            Ok(key)
        } else {
            bail!("Data lookup hierarchy for user '{}' is empty", self.id)
        }
    }

    pub fn data_lookup_hierarchy(&self) -> &Vec<String> {
        &self.data_lookup_hierarchy
    }

    /// Returns the data lookup hierarchy or an error, if the hierarchy is empty
    pub fn data_lookup_hierarchy_or_err(&self) -> Result<&Vec<String>> {
        if self.data_lookup_hierarchy.is_empty() {
            bail!("Dataset hierarchy is empty! Data lookups must explicitly name the dataset to query")
        } else {
            Ok(&self.data_lookup_hierarchy)
        }
    }

    pub fn set_data_lookup_hierarchy(&mut self, hierarchy: Vec<String>) -> Result<()> {
        // Check that each item in hierarchy is valid and that there are no duplicates
        validate_input_list(
            hierarchy.iter().collect::<Vec<&String>>(),
            self.data.keys(),
            false,
            Some(&super::duplicate_dataset_hierarchy_closure),
            Some(&super::invalid_dataset_hierarchy_closure)
        )?;
        self.data_lookup_hierarchy = hierarchy;
        Ok(())
    }

    pub fn write_data(&self, key: Option<&str>) -> Result<RwLockWriteGuard<Data>> {
        let k;
        if let Some(tmp) = key {
            k = tmp;
        } else {
            k = self.top_datakey()?;
        }
        if let Some(d) = self.data.get(k) {
            Ok(d.write().unwrap())
        } else {
            bail!("Could not find user dataset {}", k)
        }
    }

    fn read_data(&self, key: Option<&str>) -> Result<RwLockReadGuard<Data>> {
        let k;
        if let Some(tmp) = key {
            k = tmp;
        } else {
            k = self.top_datakey()?;
        }
        if let Some(d) = self.data.get(k) {
            Ok(d.read().unwrap())
        } else {
            bail!("Could not find user dataset {}", k)
        }
    }

    pub fn datasets(&self) -> &IndexMap<String, RwLock<Data>> {
        &self.data
    }

    pub fn datakeys(&self) -> Vec<&String> {
        self.data.keys().into_iter().collect::<Vec<&String>>()
    }

    fn add_dataset_placeholder(&mut self, dataset: &str, config: &DatasetConfig) -> Result<()> {
        self.data
            .insert(dataset.to_string(), RwLock::new(Data::new(dataset, config)));
        Ok(())
    }

    pub fn add_dataset(id: &str, dataset: &str, config: DatasetConfig, replace_existing: bool, as_topmost: bool) -> Result<Option<Outcome>> {
        let rtn = Self::register_dataset(id, dataset, config, replace_existing)?;

        // Mutably borrow the user again to update the hierarchy
        super::users::with_user_mut(id, |u| {
            if let Some(i) = u.data_lookup_hierarchy.iter().position( |i| i == dataset) {
                u.data_lookup_hierarchy.remove(i);
            }

            if as_topmost {
                u.data_lookup_hierarchy.insert(0, dataset.to_owned());
            } else {
                u.data_lookup_hierarchy.push(dataset.to_owned())
            }
            Ok(())
        })?;
        Ok(rtn)
    }

    pub fn register_dataset(id: &str, dataset: &str, config: DatasetConfig, replace_existing: bool) -> Result<Option<Outcome>> {
        // Grab a mutable reference and update the users
        super::users::with_user_mut(id, |u| {
            if u.data.contains_key(dataset) {
                if replace_existing {
                    u.data.shift_remove(dataset);
                } else {
                    bail!("User '{}' already has dataset '{}'", &u.id, dataset);
                }
            }
            u.data.insert(dataset.to_string(), RwLock::new(Data::new(dataset, &config)));
            Ok(())
        })?;

        // Free up the mutable borrow, and, if needed, populate the config
        if config.should_auto_populate() {
            super::users::with_user(id, |u| {
                match Data::populate(u, dataset, false, false, true)? {
                    Some(res) => Ok(Some(res)),
                    // TODO backend bail
                    None => bail!("Something has gone wrong and a newly added dataset is already marked as populated")
                }
            })
        } else {
            Ok(None)
        }
    }

    pub fn new (
        id: &str,
        users: &super::users::Users,
        password_cache_option: Option<PasswordCacheOptions>,
        uid_cnt: usize,
    ) -> Result<Self> {
        let mut u = Self {
            id: id.to_string(),
            data: IndexMap::new(),

            password_semaphore: Mutex::new(0),
            data_lookup_hierarchy: users.default_data_lookup_hierarchy().clone(),
            password_cache_option: match password_cache_option {
                Some(pco) => pco,
                None => PasswordCacheOptions::None
            },
            motive_mapping: users.motive_mapping().to_owned(),
            populate_status: PopulateStatus::default(),
            session_config: RwLock::new(users.default_session_config().clone()),
            uid_num: uid_cnt,
        };

        for (ds, config) in users.default_datasets().iter() {
            u.add_dataset_placeholder(ds, config).unwrap();
        }
        Ok(u)
    }

    pub fn unload(&self) -> Result<()> {
        // TODO call some frontend method? Mark as stale somewhere? Idk. Placeholder for now
        Ok(())
    }

    pub fn is_current(&self) -> Result<bool> {
        Ok(match super::users::get_current_user_id()? {
            Some(id) => id == self.id,
            None => false
        })
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn uid_num(&self) -> usize {
        self.uid_num
    }

    pub fn password_cache_option(&self) -> &PasswordCacheOptions {
        &self.password_cache_option
    }

    pub fn add_motive(&mut self, motive: String, dataset: String, replace_existing: bool) -> Result<Option<String>> {
        if !self.datasets().contains_key(&dataset) {
            bail!(
                "Cannot add motive for user '{}' corresponding to nonexistent dataset '{}'",
                &self.id,
                &dataset
            );
        }
        if !replace_existing {
            if let Some(ds) = self.motive_mapping.get(&motive) {
                bail!(
                    "Motive '{}' for user '{}' already corresponds to dataset '{}'. Use the 'replace_existing' option to update the motive",
                    motive,
                    self.id,
                    ds
                );
            }
        }
        Ok(self.motive_mapping.insert(motive, dataset))
    }

    pub fn motive_mapping(&self) -> &IndexMap<String, String> {
        &self.motive_mapping
    }

    pub fn username(&self) -> Result<String> {
        let uname;
        {
            let data = self.read_data(None).unwrap();
            uname = data.username.clone();
        }
        if let Some(u) = uname {
            Ok(u)
        } else {
            Ok(self.id.to_string())
        }
    }

    pub fn set_username(&self, username: Option<String>) -> Result<()> {
        self.with_dataset_mut(&self.top_datakey()?, |d| {
            d.username = username.clone();
            Ok(())
        })
    }

    pub fn get_email(&self) -> Result<Option<String>> {
        for dn in self.data_lookup_hierarchy_or_err()?.iter() {
            if let Some(e) = self.with_dataset(dn, |d| Ok(d.email.clone()))? {
                return Ok(Some(e));
            }
        }
        Ok(None)
    }

    pub fn require_email(&self) -> Result<String> {
        if let Some(e) = self.get_email()? {
            Ok(e)
        } else {
            bail!(
                "Tried to retrieve email for user {} but no email has been set!",
                self.id
            )
        }
    }

    pub fn set_email(&self, email: Option<String>) -> Result<()> {
        self.with_dataset_mut(&self.top_datakey()?, |d| {
            d.email = email.clone();
            Ok(())
        })
    }

    pub fn first_name(&self) -> Result<Option<String>> {
        for dn in self.data_lookup_hierarchy_or_err()?.iter() {
            if let Some(n) = self.with_dataset(&dn, |d| Ok(d.first_name.clone()))? {
                return Ok(Some(n));
            }
        }
        Ok(None)
    }

    pub fn set_first_name(&self, first_name: Option<String>) -> Result<()> {
        self.with_dataset_mut(&self.top_datakey()?, |d| {
            d.first_name = first_name.clone();
            Ok(())
        })
    }

    pub fn last_name(&self) -> Result<Option<String>> {
        for dn in self.data_lookup_hierarchy_or_err()?.iter() {
            if let Some(n) = self.with_dataset(&dn, |d| Ok(d.last_name.clone()))? {
                return Ok(Some(n));
            }
        }
        Ok(None)
    }

    pub fn set_last_name(&self, last_name: Option<String>) -> Result<()> {
        self.with_dataset_mut(&self.top_datakey()?, |d| {
            d.last_name = last_name.clone();
            Ok(())
        })
    }

    pub fn display_name(&self) -> Result<String> {
        self.display_name_for(None)
    }

    pub fn display_name_for(&self, dataset: Option<&str>) -> Result<String> {
        let key = dataset.unwrap_or(&self.top_datakey()?);
        self.with_dataset(key, |d| {
            if let Some(n) = d.get_display_name().clone() {
                Ok(n.to_string())
            } else {
                Ok(self.id.to_string())
            }
        })
    }

    pub fn set_display_name(&self, display_name: Option<String>) -> Result<()> {
        self.with_dataset_mut(&self.top_datakey()?, |d| {
            d.display_name = display_name.clone();
            Ok(())
        })
    }

    pub fn home_dir(&self) -> Result<Option<PathBuf>> {
        for dn in self.data_lookup_hierarchy_or_err()?.iter() {
            if let Some(n) = self.with_dataset(&dn, |d| Ok(d.home_dir.clone()))? {
                return Ok(Some(n));
            }
        }
        Ok(None)
    }

    pub fn require_home_dir(&self) -> Result<PathBuf> {
        if let Some(hd) = self.home_dir()? {
            Ok(hd.to_owned())
        } else {
            bail!("Required a home directory for user '{}' but none has been set", &self.id)
        }
    }

    pub fn set_home_dir(&self, new_dir: Option<PathBuf>) -> Result<()> {
        let mut data = self.write_data(None).unwrap();
        data.home_dir = new_dir;
        Ok(())
    }


    pub fn _cache_password(&self, password: &str, dataset: &str) -> Result<bool> {
        self.password_cache_option
            .cache_password(self, password, dataset)
    }

    pub fn _password_dialog(&self, dataset: &str, reason: Option<&str>) -> Result<String> {
        // TODO add attempts back in
        // for _attempt in 0..ORIGEN_CONFIG.user__password_auth_attempts {
            let msg;
            if dataset == "" {
                msg = match reason {
                    Some(x) => format!("\nPlease enter your password {}: ", x),
                    None => "\nPlease enter your password: ".to_string(),
                };
            } else {
                msg = match reason {
                    Some(x) => format!("\nPlease enter your password ({}) {}: ", dataset, x),
                    None => format!("\nPlease enter your password ({}): ", dataset),
                };
            }
            let pass = rpassword::read_password_from_tty(Some(&msg)).unwrap();
            let attempt = self._try_password(&pass, Some(dataset))?;
            if attempt.0 {
                self._cache_password(&pass, dataset)?;
                let mut data = self.write_data(Some(dataset)).unwrap();
                data.password = Some(pass.clone());
                return Ok(pass);
            } else {
                display_redln!("Sorry, that password is incorrect");
            }
        // }
        // display_redln!(
        //     "Maximum number of authentication attempts reached ({}), exiting...",
        //     ORIGEN_CONFIG.user__password_auth_attempts
        // );
        bail!(
            // TODO
            "Maximum number of authentication attempts reached ({})",
            1 // ORIGEN_CONFIG.user__password_auth_attempts
        )
    }

    // Utility function to try a password, returning a tuple of:
    // (bool, bool, Option<Outcome>)
    // where:
    //  return.0 -> If the password validation was successful or the password was not to be validated
    //  return.1 -> If the password was actually validated
    //  return.2 -> The outcome returned from the frontend, in the event of return.1 == true
    fn _try_password(&self, password: &str, dataset_name: Option<&str>) -> Result<(bool, bool, Option<Outcome>)> {
        // TODO support this?
        // Check if we are even supposed to try the password or if its already been tried
        // let dn_opt = Some(dataset_name.unwrap_or(&self.top_datakey()?));
        // if self.should_validate_password {
            // if let Some(dn) = dn_opt {
                let ds = self.read_data(dataset_name)?;
                if ds.password_needs_validation() {
                    let lookup = ds.require_data_source_for("password validation", &self.id)?;
                    let f = crate::frontend::require()?;
                    return f.with_data_store(lookup.0, lookup.1, |dstore| {
                        let r = dstore.validate_password(&self.id, &ds, password)?;
                        let o = r.outcome()?;
                        if o.errored() {
                            bail!("Errors encountered validating password: {}", o.msg_or_default())
                        } else if o.failed() {
                            Ok((false, true, Some(o.to_owned())))
                        } else {
                            Ok((true, true, Some(o.to_owned())))
                        }
                    });
                }
            // }
        // }
        Ok((true, false, None))
    }

    pub fn dataset_for(&self, motive: &str) -> Result<Option<&String>> {
        Ok(self.motive_mapping().get(motive))
    }

    pub fn set_password(
        &self,
        password: Option<String>,
        dataset: Option<&str>,
        validate: Option<bool>,
    ) -> Result<()> {
        let _lock = self.password_semaphore.lock().unwrap();

        if let Some(p) = password.as_ref() {
            let dn = dataset.unwrap_or(&self.top_datakey()?);
            self.with_dataset_mut(dn, |d| {
                d.authenticated = false;
                Ok(())
            })?;

            let res;
            if let Some(v) = validate {
                if v {
                    res = self._try_password(p, dataset)?;
                } else {
                    res = (true, false, None);
                }
            } else {
                // Consult the config whether or not to validate the password
                res = self._try_password(p, dataset)?;
            }
            if !res.0 {
                if let Some(o) = res.2 {
                    if let Some(m) = o.msg() {
                        bail!(m)
                    } else {
                        bail!(&SORRY_PW)
                    }
                } else {
                    bail!(&SORRY_PW)
                }
            }

            // either we aren't to validate, or the validation was successful
            self.with_dataset_mut(dn, |d| {
                d.password = Some(p.to_string());
                Ok(())
            })?;
            self._cache_password(p, dn)?;
            Ok(())
        } else {
            self.clear_cached_password(dataset)
        }
    }


    pub fn password(
        &self,
        reason_or_dataset: Option<&str>,
        reason_not_dataset: bool,
        default: Option<Option<&str>>,
    ) -> Result<String> {
        // REQUIRED change name from reason to 
        // In a multi-threaded scenario, this prevents concurrent threads from prompting the user for
        // the password at the same time.
        // Instead the first thread to arrive will do it, then by the time the lock is released awaiting
        // threads will be able to used the cached value instead of prompting the user.
        let _lock = self.password_semaphore.lock().unwrap();
        let dataset: &str;

        if let Some(rod) = reason_or_dataset.as_ref() {
            if reason_not_dataset {
                if let Some(mapped_dataset) = self.dataset_for(rod)? {
                    // Use this dataset
                    dataset = mapped_dataset
                } else {
                    // Reason wasn't mapped to a dataset. See if a default should be used
                    if let Some(d1) = default {
                        // Default was given
                        if let Some(d2) = d1 {
                            // A explicit dataset was given. Check that it exists, then use that
                            if !self.data.contains_key(d2) {
                                bail!(
                                    "A default dataset '{}' was provided, but this dataset does not exists for user '{}'",
                                    d2,
                                    self.id
                                )
                            }
                            dataset = d2;
                        } else {
                            // A default of None was given, meaning use the current dataset
                            dataset = &self.top_datakey()?;
                        }
                    } else {
                        // Raise an error
                        bail!("No password available for reason: '{}'", rod,);
                    }
                }
            } else {
                dataset = rod;
            }
        } else {
            dataset = &self.top_datakey()?;
        }

        let reason: Option<&str>;
        if reason_or_dataset.is_some() && reason_not_dataset {
            reason = reason_or_dataset;
        } else {
            reason = None;
        }

        // If the password has already been set, can return it.
        // Non-current users which have had their password set can still be retrieved as these
        // are likely service/function users
        // Check if the password is already set
        // Important, this is in a block to release the read lock
        {
            let data = self.read_data(Some(dataset)).unwrap();
            if let Some(p) = &data.password {
                if self._try_password(p, Some(dataset))?.0 {
                    return Ok(p.to_string());
                }
            }
        }

        // Need to lookup the password, but will only do this for the current user
        if self.is_current()? {
            if let Some(pw) = self.password_cache_option.get_password(self, dataset)? {
                // Password was cached and retrieved - test password
                if self._try_password(&pw, Some(dataset))?.0 {
                    let mut data = self.write_data(Some(dataset)).unwrap();
                    data.password = Some(pw.clone());
                    return Ok(pw);
                } else {
                    // Note: the session will be updated if the correct password is
                    // provided from the dialog
                    display_redln!("Cached password is not valid!");
                }
            }
            return self._password_dialog(dataset, reason);
        } else {
            bail!("Can't get the password for a user which is not the current user")
        }
    }

/*
    // TODO support this again?
    pub fn authenticated(&self) -> bool {
        self.read_data(None).unwrap().authenticated
    }
*/
    /// Clear the cached password for all datasets
    pub fn clear_cached_passwords(&self) -> Result<()> {
        // TODO is this needed?
        // Important: need to ensure sessions was instantiated prior to grabbing a write-lock to avoid deadlock
        {
            if self.password_cache_option.is_session_store() {
                let _ = crate::sessions();
            }
        }
        self.for_all_datasets_mut(|d| d.clear_cached_password(self))
    }

    /// Clear the cached password for the current/default dataset
    pub fn clear_cached_password(&self, dataset: Option<&str>) -> Result<()> {
        // Important: need to ensure sessions was instantiated prior to grabbing a write-lock to avoid deadlock
        {
            if self.password_cache_option.is_session_store() {
                // REQUIRED still needed?
                let _ = crate::sessions();
            }
        }
        self.write_data(dataset)?.clear_cached_password(self)
    }

    // REQUIRED see if this is still needed
    /// Gets the user's password encryption key.
    /// Encryption here is more to just avoid storing the password as plaintext rather
    /// than for actual security, but can be made more secure allowing the config to
    /// encryption the key differently.
    /// A 'password_encryption_key' can be given in a config to change the key
    /// from Origen's default. Furthermore, users can set the ENV variable to
    /// not even store the key in text.
    /// If no particular password encryption key is given, the standard
    /// encryption key will be used.
    // pub fn get_password_encryption_key(&self) -> Result<GenericArray<u8, U32>> {
    //     if let Some(k) = &crate::ORIGEN_CONFIG.password_encryption_key {
    //         Ok(*GenericArray::from_slice(&bytes_from_str_of_bytes(&k)?))
    //     } else {
    //         Ok(*GenericArray::from_slice(&bytes_from_str_of_bytes(
    //             &crate::ORIGEN_CONFIG.default_encryption_key,
    //         )?))
    //     }
    // }
    // pub fn get_password_encryption_key(&self) -> Result<&GenericArray<u8, U32>> {
    //     &self.password_encryption_key
    // }

    /// Similar to get_password_encryption_key, but for nonce instead.
    // pub fn get_password_encryption_nonce(&self) -> Result<GenericArray<u8, U12>> {
    //     if let Some(k) = &crate::ORIGEN_CONFIG.password_encryption_nonce {
    //         Ok(*GenericArray::from_slice(&bytes_from_str_of_bytes(&k)?))
    //     } else {
    //         Ok(*GenericArray::from_slice(&bytes_from_str_of_bytes(
    //             &crate::ORIGEN_CONFIG.default_encryption_nonce,
    //         )?))
    //     }
    // }
    // pub fn get_password_encryption_nonce(&self) -> Result<&GenericArray<u8, U32>> {
    //     &self.password_encryption_nonce
    // }

    // fn error_or_failure(msg: &str, allow_failures: bool, popped: &mut bool) -> Result<()> {
    //     *popped = false;
    //     if allow_failures {
    //         display_redln!("{}", msg);
    //         Ok(())
    //     } else {
    //         bail!("{}", msg)
    //     }
    // }

    pub fn autopopulate(&self) -> Result<PopulateUserReturn> {
        self.populate_status.while_populating(|| {
            let mut rtn = PopulateUserReturn::default();
            for (n, d) in self.data.iter() {
                if d.read().unwrap().should_auto_populate() {
                    match self.populate_dataset(n, false, false, true)? {
                        Some(r) => rtn.insert(n, Some(r)),
                        None => bail!("Something has gone wrong and a newly added dataset is already marked as populated")
                    };
                }
            }
            Ok(rtn)
        })
    }

    pub fn populate(&self, repopulate: bool, continue_on_error: bool, stop_on_failure: bool) -> Result<PopulateUserReturn> {
        // This also functions as means to lock out multiple, simultaneous populate attempts
        self.populate_status.while_populating(|| {
            let mut rtn = PopulateUserReturn::default();
            for (n, d) in self.data.iter() {
                if !d.read().unwrap().has_empty_populate_config() {
                    rtn.insert(n, self.populate_dataset(n, repopulate, continue_on_error, stop_on_failure)?);
                }
            }
            Ok(rtn)
        })
    }

    /// Populate any data fields
    pub fn populate_dataset(
        &self,
        name: &str,
        repopulate: bool,
        continue_on_error: bool,
        stop_on_failure: bool,
    ) -> Result<Option<Outcome>> {
        log_trace!("Populating user dataset {}", name);
        Data::populate(&self, name, repopulate, continue_on_error, stop_on_failure)
    }

    pub fn session_config(&self) -> RwLockReadGuard<SessionConfig> {
        self.session_config.read().unwrap()
    }

    pub fn session_config_mut(&self) -> Result<RwLockWriteGuard<SessionConfig>> {
        let sessions = crate::sessions();
        if sessions.groups().contains_key(&SessionConfig::to_sg_name(&self.id)) {
            bail!("The session config cannot be updated for user '{}' after the session has been created", &self.id);
        }
        Ok(self.session_config.write().unwrap())
    }

    pub fn ensure_session(&self, sessions: &mut Sessions, namespace: Option<&str>) -> Result<(bool, bool, String, String)> {
        let sc = self.session_config();
        let sg_name = SessionConfig::to_sg_name(&self.id);
        let was_group_added;
        if let Some(grp) = sessions.groups().get(&sg_name) {
            if grp.path() != &sc.resolved_path(&self)? {
                bail!("Session group '{}' does not match the session config for user '{}'", &sg_name, &self.id);
            }
            was_group_added = false;
        } else {
            sessions.add_group(&sg_name, &sc.resolved_root(&self)?, Some(sc.file_permissions.to_owned()))?;
            was_group_added = true;
        }
        let sname = namespace.unwrap_or(DEFAULT_USER_SESSION_STORE_NAME);
        let was_session_added = sessions.require_mut_group(&sg_name)?.ensure(sname)?;
        Ok((was_group_added, was_session_added, sg_name, sname.to_string()))
    }

    pub fn with_session_group<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&Sessions, &SessionGroup) -> Result<T>
    {
        let mut sessions = crate::sessions();
        let s = self.ensure_session(&mut sessions, None)?;
        let grp = sessions.require_group(&s.2)?;
        func(&sessions, &grp)
    }

    // TEST_NEEDED namespace tests
    pub fn with_session<T, F>(&self, namespace: Option<String>, mut func: F) -> Result<T>
    where
        F: FnMut(&Sessions, &SessionGroup, &SessionStore) -> Result<T>
    {
        self.with_session_group(|sessions, sg| {
            let s = sg.require(namespace.as_ref().map_or(DEFAULT_USER_SESSION_STORE_NAME, |v| v.as_str()))?;
            func(&sessions, &sg, &s)
        })
    }

    pub fn with_dataset<T, F>(&self, dataset: &str, mut func: F) -> Result<T>
    where
        F: FnMut(&Data) -> Result<T>,
    {
        let d = self.read_data(Some(dataset))?;
        let retn = func(&d)?;
        Ok(retn)
    }

    pub fn with_dataset_mut<T, F>(&self, dataset: &str, mut func: F) -> Result<T>
    where
        F: FnMut(&mut Data) -> Result<T>,
    {
        let mut d = self.write_data(Some(dataset))?;
        let retn = func(&mut d)?;
        Ok(retn)
    }

    pub fn for_all_datasets<T, F>(&self, mut func: F) -> Result<()>
    where
        F: FnMut(&Data) -> Result<T>,
    {
        for (_n, d) in self.data.iter() {
            func(&d.read().unwrap())?;
        }
        Ok(())
    }

    pub fn for_all_datasets_mut<T, F>(&self, mut func: F) -> Result<()>
    where
        F: FnMut(&mut Data) -> Result<T>,
    {
        for (_n, d) in self.data.iter() {
            func(&mut d.write().unwrap())?;
        }
        Ok(())
    }
}
