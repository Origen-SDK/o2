mod data;
mod password_cache_options;
pub mod user;
pub mod users;

pub use user::{with_top_hierarchy, with_user_dataset, with_user_dataset_mut, User};
pub use users::Users;

use crate::utility::command_helpers::exec_and_capture;
use crate::{Result, ORIGEN_CONFIG};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn lookup_dataset_config<'a>(config: &str) -> Result<&'a HashMap<String, String>> {
    if let Some(configs) = ORIGEN_CONFIG.user__datasets.as_ref() {
        if let Some(c) = configs.get(config) {
            return Ok(c);
        }
    }
    error!("Could not lookup dataset config for {}", config)
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
