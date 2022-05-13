mod data;
mod password_cache_options;
pub mod user;
pub mod users;

use crate::Result;

pub use data::{Data, DatasetConfig};
pub use user::{PopulateUserReturn, SessionConfig, User};
pub use users::PopulateUsersReturn;

pub fn whoami() -> Result<String> {
    let id = whoami::username();
    log_debug!("User ID read from whoami: '{}'", &id);
    Ok(id)
}

fn invalid_dataset_hierarchy_closure(items: &Vec<&&String>) -> String {
    format!(
        "The following datasets do not exists and cannot be used in the data lookup hierarchy: {}",
        items
            .iter()
            .map(|i| format!("'{}'", i))
            .collect::<Vec<String>>()
            .join(", ")
    )
}

fn duplicate_dataset_hierarchy_closure(item: &String, first: usize, second: usize) -> String {
    format!(
        "Dataset '{}' can only appear once in the dataset hierarchy (first appearance at index {} - duplicate at index {})",
        item,
        first,
        second
    )
}

/*
// TODO need to support this stuff again
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
*/
