use crate::revision_control::git;
use crate::utility::command_helpers::exec_and_capture;
use crate::utility::ldap::LDAPs;
use crate::utility::{
    bytes_from_str_of_bytes, decrypt_with, encrypt_with, str_from_byte_array, str_to_bool,
};
use crate::{Error, Metadata, Result, ORIGEN_CONFIG};
use aes_gcm::aead::{
    generic_array::typenum::{U12, U32},
    generic_array::GenericArray,
};
use indexmap::IndexMap;
#[cfg(feature = "password-cache")]
use keyring::Keyring;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

const PASSWORD_KEY: &str = "user_password__";

#[allow(non_snake_case)]
pub fn user__password_reasons<'a>() -> &'a HashMap<String, String> {
    &ORIGEN_CONFIG.user__password_reasons
}

pub fn lookup_dataset_config<'a>(config: &str) -> Result<&'a HashMap<String, String>> {
    if let Some(c) = ORIGEN_CONFIG.user__datasets.get(config) {
        Ok(c)
    } else {
        error!("Could not lookup dataset config for {}", config)
    }
}

fn to_session_password<'a>(dataset: &str) -> String {
    format!("{}{}", PASSWORD_KEY, dataset)
}

pub fn get_current_email() -> Result<String> {
    crate::with_current_user(|u| u.get_email())
}

pub fn current_home_dir() -> Result<PathBuf> {
    crate::with_current_user(|u| u.home_dir())
}

pub fn get_current_id() -> Result<String> {
    crate::with_current_user(|u| Ok(u.id().to_string()))
}

pub fn whoami() -> Result<String> {
    let id;
    if cfg!(unix) {
        let output = exec_and_capture("whoami", None);
        if let Ok((status, mut lines, _stderr)) = output {
            if status.success() {
                id = lines.pop().unwrap();
            } else {
                return error!("Failed to run 'whoami'");
            }
        } else {
            return error!("Failed to run 'whoami'");
        }
        log_debug!("User ID read from the system: '{}'", &id);
    } else {
        id = whoami::username();
        log_debug!("User ID read from whoami: '{}'", &id);
    }
    Ok(id)
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

/// Initiates the password dialog for the current user for all the given datasets.
/// If None, the current dataset is used.
pub fn set_passwords(datasets: Option<Vec<&str>>) -> Result<()> {
    if let Some(datasets) = datasets {
        let mut err_str = "".to_string();
        for ds in datasets {
            match crate::with_current_user(|u| u._password_dialog(&ds.to_string(), None)) {
                Ok(_) => {}
                Err(e) => {
                    err_str.push_str(&format!("{}\n", e.msg));
                }
            }
        }
        if err_str != "" {
            error!("{}", err_str)
        } else {
            Ok(())
        }
    } else {
        crate::with_current_user(|u| u._password_dialog(&u.top_datakey()?, None))?;
        Ok(())
    }
}

pub fn set_all_passwords() -> Result<()> {
    let users = crate::users();
    let u = users.current_user()?;
    let datasets = u.datasets().keys();
    set_passwords(Some(datasets.map(|s| s.as_str()).collect()))
}

pub fn clear_passwords(datasets: Option<Vec<&str>>) -> Result<()> {
    if let Some(datasets) = datasets {
        let mut err_str = "".to_string();
        for ds in datasets {
            log_trace!("Clearing password for {}", ds);
            match crate::with_current_user(|u| u.clear_cached_password(Some(&ds))) {
                Ok(_) => {}
                Err(e) => {
                    err_str.push_str(&format!("{}\n", e.msg));
                }
            }
        }
        if err_str != "" {
            error!("{}", err_str)
        } else {
            Ok(())
        }
    } else {
        log_trace!("Clearing password for current dataset");
        crate::with_current_user(|u| u.clear_cached_password(None))?;
        Ok(())
    }
}

pub fn clear_all_passwords() -> Result<()> {
    log_trace!("Clearing all cached passwords for current user");
    crate::with_current_user(|u| u.clear_cached_passwords())
}

#[derive(Debug, Clone)]
pub enum Roles {
    User(Option<IndexMap<String, crate::Metadata>>),
    Service(Option<IndexMap<String, crate::Metadata>>),
}

impl std::default::Default for Roles {
    fn default() -> Self {
        Self::User(Option::None)
    }
}

pub const DEFAULT_KEY: &str = "default";

#[derive(Debug)]
pub enum PasswordCacheOptions{
    Session,
    Keyring,
    None,
}

impl PasswordCacheOptions {
    pub fn from_config()-> Result<Self> {
        let opt = &crate::ORIGEN_CONFIG.user__password_cache_option;
        match opt.as_str() {
            "session" | "session_store" => Ok(Self::Session),
            "keyring" | "true" => Ok(Self::Keyring),
            "none" | "false" => Ok(Self::None),
            _ => error!("'user__password_cache_option' option '{}' is not known!", opt)
        }
    }

    pub fn cache_password(&self, user: &User, password: &str, dataset: &str) -> Result<bool> {
        match self {
            Self::Session => {
                log_trace!("Caching password in session store...");
                let mut s = crate::sessions();
                let sess = s.user_session(None)?;
                sess.store(
                    to_session_password(dataset),
                    crate::Metadata::String(str_from_byte_array(&encrypt_with(
                        password,
                        user.get_password_encryption_key()?,
                        user.get_password_encryption_nonce()?,
                    )?)?),
                )?;
                Ok(true)
            }
            Self::Keyring => {
                log_trace!("Caching password in keyring...");
                let k = keyring::Keyring::new(dataset, &user.id);
                k.set_password(password)?;
                Ok(true)
            }
            Self::None => {
                log_trace!("Password caching unavailable");
                Ok(false)
            }
        }
    }

    pub fn get_password(&self, user: &User, dataset: &str) -> Result<Option<String>> {
        match self {
            Self::Session => {
                log_trace!("Checking for password in session store...");
                // Check if the password is cached in the user's session
                let mut s = crate::sessions();
                let sess = s.user_session(None)?;
                if let Some(p) = sess.retrieve(&to_session_password(dataset))? {
                    // Password should be encrypted (to avoid storing as plaintext)
                    // Decrypt the password
                    let pw = decrypt_with(
                        &bytes_from_str_of_bytes(&p.as_string()?)?,
                        user.get_password_encryption_key()?,
                        user.get_password_encryption_nonce()?,
                    )?;
                    Ok(Some(pw.to_string()))
                } else {
                    Ok(None)
                }
            }
            Self::Keyring => {
                log_trace!("Checking for password in keyring...");
                let k = keyring::Keyring::new(dataset, &user.id);
                match k.get_password() {
                    Ok(password) => Ok(Some(password)),
                    Err(e) => match e {
                        keyring::KeyringError::NoPasswordFound => Ok(None),
                        _ => error!("{}", e)
                    }
                }
            }
            Self::None => error!("Cannot get password when password caching is unavailable!")
        }
    }

    pub fn clear_cached_password(&self, parent: &User, dataset: &Data) -> Result<()> {
        match self {
            Self::Session => {
                let k = dataset.password_key();
                if parent.is_current() {
                    log_trace!("Clearing password {} from user session", k);
                    crate::with_user_session(None, |session| session.delete(&k))?;
                }
            }
            Self::Keyring => {
                let k = keyring::Keyring::new(&dataset.dataset_name, &parent.id);
                match k.delete_password() {
                    Ok(_) => {},
                    Err(e) => match e {
                        keyring::KeyringError::NoPasswordFound => {},
                        _ => return error!("{}", e)
                    }
                }
            }
            Self::None => {}
        }
        Ok(())
    }

    pub fn is_session_store(&self) -> bool {
        match self {
            Self::Session => true,
            _ => false
        }
    }

    pub fn is_keyring(&self) -> bool {
        match self {
            Self::Keyring => true,
            _ => false
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false
        }
    }
}

pub struct Users {
    users: IndexMap<String, User>,
    current_id: String,
    // initial_id: String,
}

impl Users {
    pub fn current_user(&self) -> Result<&User> {
        Ok(self.users.get(&self.current_id).unwrap())
    }

    pub fn current_user_id(&self) -> Result<String> {
        Ok(self.current_id.clone())
    }

    pub fn user(&self, u: &str) -> Result<&User> {
        if let Some(user) = self.users.get(u).as_ref() {
            Ok(&user)
        } else {
            error!("No user '{}' has been added", u)
        }
    }

    pub fn user_mut(&mut self, u: &str) -> Result<&mut User> {
        if let Some(user) = self.users.get_mut(u) {
            Ok(user)
        } else {
            error!("No user '{}' has been added", u)
        }
    }

    pub fn users(&self) -> &IndexMap<String, User> {
        &self.users
    }

    pub fn add(&mut self, id: &str) -> Result<()> {
        if self.users.contains_key(id) {
            error!("User '{}' has already been added", id)
        } else {
            self.users.insert(id.to_string(), User::new(id));
            Ok(())
        }
    }
}

impl Default for Users {
    fn default() -> Self {
        let u = User::current();
        let id = u.id().to_string();
        let users = Self {
            users: {
                let mut i = IndexMap::new();
                i.insert(id.clone(), u);
                i
            },
            current_id: id.clone(),
            // initial_id: id
        };
        users
    }
}

#[derive(Debug)]
pub struct User {
    // All user data is stored behind a RW lock so that it can be lazily loaded
    // from the environment and cached behind the scenes
    data: HashMap<String, RwLock<Data>>,
    data_lookup_hierarchy: Vec<String>,
    password_semaphore: Mutex<u8>,
    id: String,
    password_cache_option: PasswordCacheOptions,
}

#[derive(Default, Debug)]
pub struct Data {
    dataset_name: String,
    password: Option<String>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub other: IndexMap<String, Metadata>,
    pub home_dir: PathBuf,
    // Will be set after trying to get a missing name, e.g. from the
    // Git config to differentiate between an name which has not been
    // looked up and name which has been looked up but which could not
    // be found.
    pub name_tried: bool,
    pub email: Option<String>,
    pub email_tried: bool,

    // Authentication
    authenticated: bool,
    pub populated: bool,

    roles: Vec<Roles>,
}

impl Data {
    pub fn new(dataset_name: &str) -> Self {
        Self {
            dataset_name: dataset_name.to_string(),
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

    pub fn password_key(&self) -> String {
        format!("{}{}", PASSWORD_KEY, self.dataset_name)
    }

    pub fn clear_cached_password(&mut self, parent: &User) -> Result<()> {
        let k = self.password_key();
        log_trace!("Clearing cached password for {}", k);
        self.password = None;
        self.authenticated = false;
        parent.password_cache_option.clear_cached_password(parent, self)?;
        Ok(())
    }
}

impl User {
    pub fn data_lookup_hierarchy_from_config<'a>() -> Vec<String> {
        crate::ORIGEN_CONFIG.user__data_lookup_hierarchy.clone()
    }

    pub fn top_datakey(&self) -> Result<&str> {
        if let Some(key) = self.data_lookup_hierarchy.first() {
            Ok(key)
        } else {
            error!("Data lookup hierarchy for user '{}' is empty", self.id)
        }
    }

    pub fn data_lookup_hierarchy(&self) -> &Vec<String> {
        &self.data_lookup_hierarchy
    }

    /// Returns the data lookup hierarchy or an error, if the hierarchy is empty
    pub fn data_lookup_hierarchy_or_err(&self) -> Result<&Vec<String>> {
        if self.data_lookup_hierarchy.is_empty() {
            error!("Dataset hierarchy is empty! Data lookups must explicitly name the dataset to query")
        } else {
            Ok(&self.data_lookup_hierarchy)
        }
    }

    pub fn set_data_lookup_hierarchy(&mut self, hierarchy: Vec<String>) -> Result<()> {
        // Check that each item in hierarchy is valid and that there are no duplicates
        let mut indices = HashMap::new();
        for (i, d) in hierarchy.iter().enumerate() {
            if self.data.contains_key(d) {
                if let Some(idx) = indices.get(d) {
                    // Duplicate dataset
                    return error!(
                        "Dataset '{}' can only appear once in the dataset hierarchy (first appearance at index {} - duplicate at index {})",
                        d,
                        idx,
                        i
                    )
                }
                indices.insert(d, i);
            } else {
                return error!("No dataset '{}' defined! Cannot use this in the datakey hierarchy", d);
            }
        }
        self.data_lookup_hierarchy = hierarchy;
        Ok(())
    }

    fn write_data(&self, key: Option<&str>) -> Result<RwLockWriteGuard<Data>> {
        let k;
        if let Some(tmp) = key {
            k = tmp;
        } else {
            k = self.top_datakey()?;
        }
        if let Some(d) = self.data.get(k) {
            Ok(d.write().unwrap())
        } else {
            error!("Could not find user dataset {}", k)
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
            error!("Could not find user dataset {}", k)
        }
    }

    pub fn datasets(&self) -> &HashMap<String, RwLock<Data>> {
        &self.data
    }

    fn add_dataset_placeholder(&mut self, dataset: &str) -> Result<()> {
        self.data
            .insert(dataset.to_string(), RwLock::new(Data::new(dataset)));
        Ok(())
    }

    pub fn current() -> User {
        log_trace!("Building Current User...");
        let id = match whoami() {
            Ok(id) => id.to_string(),
            Err(e) => {
                display_redln!("{}", e.msg);
                "".to_string()
            }
        };
        let u = User::new(&id);
        {
            let mut data = u.write_data(None).unwrap();
            data.home_dir = super::status::get_home_dir();
        }
        log_trace!("Built Current User: {}", u.id());
        u
    }

    pub fn new(id: &str) -> User {
        let mut u = Self {
            id: id.to_string(),
            data: {
                let mut h = HashMap::new();
                h.insert(
                    Self::data_lookup_hierarchy_from_config().first().unwrap().to_string(),
                    RwLock::new(Data::default()),
                );
                h
            },
            password_semaphore: Mutex::new(0),
            data_lookup_hierarchy: Self::data_lookup_hierarchy_from_config(),
            password_cache_option: match PasswordCacheOptions::from_config() {
                Ok(opt) => opt,
                Err(e) => {
                    display_redln!("{}", e);
                    PasswordCacheOptions::None
                }
            },
        };
        for (name, config) in ORIGEN_CONFIG.user__datasets.iter() {
            u.add_dataset_placeholder(name).unwrap();

            // Default is to populate any datasets at creation time.
            if let Some(should_pop) = config.get("auto_populate") {
                match str_to_bool(should_pop) {
                    Ok(should_pop_bool) => {
                        if !should_pop_bool {
                            continue;
                        }
                    }
                    Err(e) => {
                        display_redln!("Errors occurred processing dataset config: {}", e.msg);
                        display_redln!("Unable to populate dataset '{}'", name);
                        continue;
                    }
                }
            }
            match u.populate(name, config, true) {
                Ok(popped) => {
                    if !popped {
                        // Errors occurred - did not populate
                        // (reason should have been printed in the populate function)
                        display_redln!("Unable to populate dataset '{}'", name);
                    }
                }
                Err(e) => {
                    // Uncaught error occurred (likely a backend problem)
                    display_redln!(
                        "Unable to populate dataset '{}'. Uncaught error occurred during population: {}",
                        name,
                        e.msg
                    );
                }
            }
        }
        u
    }

    pub fn is_current(&self) -> bool {
        match whoami() {
            Ok(current) => self.id().as_str() == current.as_str(),
            Err(e) => {
                display_redln!("Error retrieving the current user: {}", e.msg);
                false
            }
        }
    }

    pub fn id(&self) -> &String {
        &self.id
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

    pub fn email(&self) -> Result<Option<String>> {
        for dn in self.data_lookup_hierarchy_or_err()?.iter() {
            if let Some(e) = self.with_dataset(dn, |d| Ok(d.email.clone()))? {
                return Ok(Some(e))
            }
        }
        Ok(None)
    }

    pub fn get_email(&self) -> Result<String> {
        if let Some(e) = self.email()? {
            Ok(e)
        } else {
            error!(
                "Tried to retrieve email for user {} but none is has been set across any datasets!",
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
                return Ok(Some(n))
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
                return Ok(Some(n))
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

    pub fn home_dir(&self) -> Result<PathBuf> {
        Ok(self.read_data(None).unwrap().home_dir.clone())
    }

    pub fn home_dir_string(&self) -> Result<String> {
        Ok(self.home_dir()?.to_string_lossy().to_string())
    }

    pub fn set_home_dir(&self, new_dir: PathBuf) -> Result<()> {
        let mut data = self.write_data(None).unwrap();
        data.home_dir = new_dir;
        Ok(())
    }

    fn _cache_password(&self, password: &str, dataset: &str) -> Result<bool> {
        self.password_cache_option.cache_password(self, password, dataset)
    }

    fn _password_dialog(&self, dataset: &str, reason: Option<&str>) -> Result<String> {
        for _attempt in 0..ORIGEN_CONFIG.user__password_auth_attempts {
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
            if self._try_password(&pass, Some(dataset))? {
                self._cache_password(&pass, dataset)?;
                let mut data = self.write_data(Some(dataset)).unwrap();
                data.password = Some(pass.clone());
                return Ok(pass);
            } else {
                display_redln!("Sorry, that password is incorrect");
            }
        }
        display_redln!(
            "Maximum number of authentication attempts reached ({}), exiting...",
            ORIGEN_CONFIG.user__password_auth_attempts
        );
        error!(
            "Maximum number of authentication attempts reached ({})",
            ORIGEN_CONFIG.user__password_auth_attempts
        )
    }

    fn _try_password(&self, password: &str, dataset_name: Option<&str>) -> Result<bool> {
        // Check if we are even supposed to try the password or if its already been tried
        let dn = Some(dataset_name.unwrap_or(&self.top_datakey()?));
        if self.should_validate_password(dn)? && !self.read_data(dn).unwrap().authenticated {
            if let Some(dataset) = ORIGEN_CONFIG.user__datasets.get(dn.unwrap()) {
                if let Some(data_source) = dataset.get("data_source") {
                    if data_source == "ldap" {
                        if let Some(ldap_name) = dataset.get("data_lookup") {
                            // Attempt to bind to the ldap with this user
                            return LDAPs::try_password(
                                ldap_name,
                                &self
                                    .read_data(dn)?
                                    .username
                                    .as_ref()
                                    .unwrap_or(&self.username()?),
                                password,
                            );
                        } else {
                            return error!("A 'data_lookup' key corresponding to the ldap name is required to validate passwords against an LDAP");
                        }
                    } else {
                        return error!(
                            "Cannot verify user password for user data source {}",
                            data_source
                        );
                    }
                } else {
                    return error!(
                        "Cannot validate password without data source for dataset {}",
                        dn.unwrap()
                    );
                }
            } else {
                return error!("No dataset config given for {}", dn.unwrap());
            }
        } else {
            Ok(true)
        }
    }

    pub fn should_validate_password(&self, dataset_name: Option<&str>) -> Result<bool> {
        if let Some(name) = dataset_name {
            if let Some(dataset) = ORIGEN_CONFIG.user__datasets.get(name) {
                if let Some(ans) = dataset.get("try_password") {
                    match ans.as_str() {
                        "true" | "True" => Ok(true),
                        "false" | "False" => Ok(false),
                        _ => error!("Could not convert string {} to boolean value", ans),
                    }
                } else {
                    Ok(false)
                }
            } else {
                error!("No dataset config given for {}", name)
            }
        } else {
            Ok(false)
        }
    }

    pub fn dataset_for(&self, reason: &str) -> Option<&str> {
        if let Some(d) = user__password_reasons().get(reason) {
            Some(d)
        } else {
            None
        }
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

            if let Some(v) = validate {
                if v {
                    self._try_password(p, dataset)?;
                }
            } else {
                // Consult the config whether or not to validate the password
                self._try_password(p, dataset)?;
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
        // In a multi-threaded scenario, this prevents concurrent threads from prompting the user for
        // the password at the same time.
        // Instead the first thread to arrive will do it, then by the time the lock is released awaiting
        // threads will be able to used the cached value instead of prompting the user.
        let _lock = self.password_semaphore.lock().unwrap();
        let dataset;

        if let Some(rod) = reason_or_dataset.as_ref() {
            if reason_not_dataset {
                if let Some(mapped_dataset) = self.dataset_for(rod) {
                    // Use this dataset
                    dataset = mapped_dataset;
                } else {
                    // Reason wasn't mapped to a dataset. See if a default should be used
                    if let Some(d1) = default {
                        // Default was given
                        if let Some(d2) = d1 {
                            // A explicit dataset was given. Use that
                            dataset = d2;
                        } else {
                            // A default of None was given, meaning use the current dataset
                            dataset = &self.top_datakey()?;
                        }
                    } else {
                        // Raise an error
                        return error!("No password available for reason: '{}'", rod,);
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
                if self._try_password(p, Some(dataset))? {
                    return Ok(p.to_string());
                }
            }
        }

        // Need to lookup the password, but will only do this for the current user
        if self.is_current() {
            if let Some(pw) = self.password_cache_option.get_password(self, dataset)? {
                // Password was cached and retrieved - test password
                if self._try_password(&pw, Some(dataset))? {
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
            Err(Error::new(
                "Can't get the password for a user which is not the current user",
            ))
        }
    }

    pub fn authenticated(&self) -> bool {
        self.read_data(None).unwrap().authenticated
    }

    /// Clear the cached password for all datasets
    pub fn clear_cached_passwords(&self) -> Result<()> {
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
                let _ = crate::sessions();
            }
        }
        self.write_data(dataset)?.clear_cached_password(self)
    }

    /// Gets the user's password encryption key.
    /// Encryption here is more to just avoid storing the password as plaintext rather
    /// than for actual security, but can be made more secure allowing the config to
    /// encryption the key differently.
    /// A 'password_encryption_key' can be given in a config to change the key
    /// from Origen's default. Furthermore, users can set the ENV variable to
    /// not even store the key in text.
    /// If no particular password encryption key is given, the standard
    /// encryption key will be used.
    fn get_password_encryption_key(&self) -> Result<GenericArray<u8, U32>> {
        if let Some(k) = &crate::ORIGEN_CONFIG.password_encryption_key {
            Ok(*GenericArray::from_slice(&bytes_from_str_of_bytes(&k)?))
        } else {
            Ok(*GenericArray::from_slice(&bytes_from_str_of_bytes(
                &crate::ORIGEN_CONFIG.default_encryption_key,
            )?))
        }
    }

    /// Similar to get_password_encryption_key, but for nonce instead.
    fn get_password_encryption_nonce(&self) -> Result<GenericArray<u8, U12>> {
        if let Some(k) = &crate::ORIGEN_CONFIG.password_encryption_nonce {
            Ok(*GenericArray::from_slice(&bytes_from_str_of_bytes(&k)?))
        } else {
            Ok(*GenericArray::from_slice(&bytes_from_str_of_bytes(
                &crate::ORIGEN_CONFIG.default_encryption_nonce,
            )?))
        }
    }

    /// Populate any data fields
    pub fn populate(
        &self,
        name: &str,
        config: &HashMap<String, String>,
        allow_failures: bool,
    ) -> Result<bool> {
        fn error_or_failure(msg: &str, allow_failures: bool, popped: &mut bool) -> Result<()> {
            *popped = false;
            if allow_failures {
                display_redln!("{}", msg);
                Ok(())
            } else {
                error!("{}", msg)
            }
        }

        let mut popped = true;
        log_trace!("Populating user dataset {}", name);
        if let Some(s) = config.get("data_source") {
            match s.as_ref() {
                "ldap" | "LDAP" => {
                    if let Some(ldap_name) = config.get("data_lookup") {
                        let mut ldaps = crate::ldaps();
                        let ldap;
                        match ldaps._get_mut(ldap_name) {
                            Ok(l) => ldap = l,
                            Err(e) => {
                                error_or_failure(&e.msg, allow_failures, &mut popped)?;
                                return Ok(popped);
                            }
                        }
                        if let Err(e) = ldap.bind() {
                            error_or_failure(
                                &format!("LDAP bind failed with error: {}", e.msg),
                                allow_failures,
                                &mut popped,
                            )?;
                            return Ok(popped);
                        }
                        // See if a username has already been populated in this dataset. If so, use that.
                        // Otherwise, use the current id
                        let uname;
                        {
                            let data = self.read_data(Some(name))?;
                            if let Some(n) = &data.username {
                                uname = n.to_string();
                            } else {
                                uname = self.username()?;
                            }
                        }
                        // Grab all available fields
                        let fields = ldap
                            .single_filter_search(
                                &format!("{}={}", config.get("data_id").unwrap(), uname),
                                vec!["*"],
                            )?
                            .0;
                        let mut data = self.write_data(Some(name))?;
                        if let Some(mapping) = ORIGEN_CONFIG.user__dataset_mappings.get(name) {
                            for (key, val) in mapping.iter() {
                                if let Some(v) = fields.get(val) {
                                    if key == "name" {
                                        data.name = Some(v.first().unwrap().to_string());
                                    } else if key == "email" {
                                        data.email = Some(v.first().unwrap().to_string());
                                    } else if key == "username" {
                                        data.username = Some(v.first().unwrap().to_string());
                                    } else if key == "last_name" {
                                        data.last_name = Some(v.first().unwrap().to_string());
                                    } else if key == "first_name" {
                                        data.first_name = Some(v.first().unwrap().to_string());
                                    } else if key == "display_name" {
                                        data.display_name = Some(v.first().unwrap().to_string());
                                    } else {
                                        data.other.insert(
                                            key.to_string(),
                                            Metadata::String(v.first().unwrap().to_string()),
                                        );
                                    }
                                } else {
                                    error_or_failure(
                                        &format!(
                                            "Cannot find mapped value '{}' in LDAP {}",
                                            val, ldap_name
                                        ),
                                        allow_failures,
                                        &mut popped,
                                    )?
                                }
                            }
                        } else {
                            error_or_failure(
                                &format!("Cannot find dataset mapping for '{}'", name),
                                allow_failures,
                                &mut popped,
                            )?
                        }
                    } else {
                        error_or_failure(
                            "LDAP data source requires a 'data_lookup' key corresponding to the LDAP name",
                            allow_failures,
                            &mut popped
                        )?
                    }
                }
                "git" | "Git" => {
                    // Out of the git config, try to retrieve the email and username
                    let mut data = self.write_data(Some(name))?;
                    data.display_name = git::config("name");
                    data.email = git::config("email");
                }
                _ => error_or_failure(
                    &format!("Unknown dataset source {}", s),
                    allow_failures,
                    &mut popped,
                )?,
            }
        }
        let mut data = self.write_data(Some(name))?;
        data.populated = true;
        Ok(popped)
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
