use crate::{Metadata, Result};
use indexmap::IndexMap;
use std::path::PathBuf;

use super::password_cache_options::PASSWORD_KEY;
use super::User;

/// Struct to hold user data for a given dataset.
/// At least in the backend, each data instance is independent
/// with an dataset-dependent features (e.g., username, password, retrieval)
/// handled by the owning User struct.
#[derive(Default, Debug)]
pub struct Data {
    pub dataset_name: String,
    pub password: Option<String>,
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
    pub authenticated: bool,
    pub populated: bool,

    #[allow(dead_code)]
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
        parent
            .password_cache_option()
            .clear_cached_password(parent, self)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Roles {
    User(Option<IndexMap<String, crate::Metadata>>),
    // Service(Option<IndexMap<String, crate::Metadata>>),
}

impl std::default::Default for Roles {
    fn default() -> Self {
        Self::User(Option::None)
    }
}
