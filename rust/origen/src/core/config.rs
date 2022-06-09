//! Collects the main Origen configuration options from all origen.toml files
//! found in the application and Origen CLI installation file system paths
//!
//! # Examples
//!
//! ```
//! use origen::ORIGEN_CONFIG;
//!
//! println!("Server: {}", &ORIGEN_CONFIG.pkg_server);  // => "Server: https://pkgs.company.net:9292"
//! ```

use crate::STATUS;
use origen_metal::config;
use origen_metal::config::{Environment, File};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;

// TODO Get from a prelude?
use crate::om;
use om::framework::users::DatasetConfig as OMDatasetConfig;

lazy_static! {
    pub static ref CONFIG: Config = Config::default();
}

#[derive(Debug, Deserialize)]
pub struct LDAPConfig {
    pub server: String,
    pub base: String,
    pub auth: Option<HashMap<String, config::Value>>,
    pub continuous_bind: Option<bool>,
    pub populate_user_config: Option<HashMap<String, config::Value>>,
    pub timeout: Option<i32>,
}

impl std::convert::From<LDAPConfig> for config::ValueKind {
    fn from(_value: LDAPConfig) -> Self {
        Self::Nil
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatasetConfig {
    category: Option<String>,
    data_store: Option<String>,
    auto_populate: Option<bool>,
    should_validate_password: Option<bool>,
}

impl std::convert::TryFrom<&DatasetConfig> for OMDatasetConfig {
    type Error = om::Error;

    fn try_from(value: &DatasetConfig) -> Result<Self, Self::Error> {
        OMDatasetConfig::new(
            value.category.clone(),
            value.data_store.clone(),
            value.auto_populate,
            value.should_validate_password,
        )
    }
}

impl std::convert::From<DatasetConfig> for config::ValueKind {
    fn from(_value: DatasetConfig) -> Self {
        Self::Nil
    }
}

// TODO likely need a user config
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct UserConfig {
//     pub user__data_lookup_hierarchy: Option<Vec<String>>,
//     pub user__datasets: Option<HashMap<String, HashMap<String, String>>>,
//     pub user__password_auth_attempts: u8,
//     pub user__password_cache_option: String,
//     pub user__password_reasons: HashMap<String, String>,
//     pub password_encryption_key: Option<String>,
//     pub password_encryption_nonce: Option<String>,
//     // pre-populated users, generally for explicit purposes such as an LDAP service user
//     // or regression test launcher
//     pub user__dataset_mappings: HashMap<String, HashMap<String, String>>,
//     pub service_users: HashMap<String, HashMap<String, String>>,
//     // User session root path
//     pub session__user_root: Option<String>,
// }

#[macro_export]
macro_rules! exit_on_bad_config {
    ($result: expr) => {
        match $result {
            Ok(r) => r,
            Err(e) => {
                log_error!("Malformed config file");
                log_error!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
// If you add an attribute to this you must also update:
// * pyapi/src/lib.rs to convert it to Python
// * default function below to define the default value (no nil in Rust)
// * add an example of it to src/app_generators/templates/app/config/origen.toml
pub struct Config {
    pub python_cmd: String,
    pub pkg_server: String,
    pub pkg_server_push: String,
    pub pkg_server_pull: String,
    pub some_val: u32,

    // Mailer
    pub mailer__maillists_dirs: Vec<String>,
    pub mailer: Option<HashMap<String, String>>,

    // LDAPs
    pub ldaps: HashMap<String, LDAPConfig>,

    // Very Basic Encryption
    // TEST_NEEDED ensure these get set in OM
    pub default_encryption_key: Option<String>,
    pub default_encryption_nonce: Option<String>,

    // Various user config
    pub user__data_lookup_hierarchy: Option<Vec<String>>,
    pub user__datasets: Option<HashMap<String, DatasetConfig>>,
    pub user__password_auth_attempts: u8,
    pub user__password_cache_option: String,
    pub user__current_user_lookup_function: Option<String>,
    // pub user__password_reasons: HashMap<String, String>,
    pub user__dataset_motives: HashMap<String, String>,
    pub password_encryption_key: Option<String>,
    pub password_encryption_nonce: Option<String>,
    // pre-populated users, generally for explicit purposes such as an LDAP service user
    // or regression test launcher
    pub user__dataset_mappings: HashMap<String, HashMap<String, String>>,
    pub service_users: HashMap<String, HashMap<String, String>>,
    // User session root path
    pub session__user_root: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        log_trace!("Instantiating Origen config");
        let mut s = config::Config::builder();

        // Start off by specifying the default values for all attributes, seems fine
        // not to handle these errors
        s = s.set_default("python_cmd", "").unwrap();
        s = s.set_default("pkg_server", "").unwrap();
        s = s.set_default("pkg_server_push", "").unwrap();
        s = s.set_default("pkg_server_pull", "").unwrap();
        s = s.set_default("some_val", 3).unwrap();
        s = s
            .set_default("mailer__maillists_dirs", Vec::<String>::new())
            .unwrap();
        s = s
            .set_default("mailer", None::<HashMap<String, String>>)
            .unwrap();
        s = s
            .set_default("ldaps", {
                let h: HashMap<String, LDAPConfig> = HashMap::new();
                h
            })
            .unwrap();
        s = s
            .set_default("user__data_lookup_hierarchy", None::<Vec<String>>)
            .unwrap();
        s = s.set_default("user__password_auth_attempts", 3).unwrap();
        s = s
            .set_default("user__password_cache_option", "keyring")
            .unwrap();
        s = s
            .set_default("user__datasets", {
                let h: Option<HashMap<String, DatasetConfig>> = None;
                h
            })
            .unwrap();
        s = s
            .set_default("user__dataset_mappings", {
                let h: HashMap<String, HashMap<String, String>> = HashMap::new();
                h
            })
            .unwrap();
        s = s
            .set_default("user__dataset_motives", {
                let h: HashMap<String, String> = HashMap::new();
                h
            })
            .unwrap();
        s = s
            .set_default("service_users", {
                let h: HashMap<String, HashMap<String, String>> = HashMap::new();
                h
            })
            .unwrap();

        s = s
            .set_default("default_encryption_key", None::<String>)
            .unwrap();
        s = s
            .set_default("default_encryption_nonce", None::<String>)
            .unwrap();
        s = s.set_default("user__current_user_lookup_function", None::<String>).unwrap();

        // Encryption keys specifically for passwords
        s = s
            .set_default("password_encryption_key", None::<String>)
            .unwrap();
        s = s
            .set_default("password_encryption_nonce", None::<String>)
            .unwrap();

        // Session setup
        s = s.set_default("session__user_root", None::<String>).unwrap();

        // Find all the origen.toml files
        let mut files: Vec<PathBuf> = Vec::new();

        // Highest priority are any files added by the user from the command line
        // If a .toml is given directly, add this file
        // If its a directory, add non-recursively search for an origen.toml
        // Multiple paths are allowed, separated by whatever the OS's separator is
        if let Some(paths) = std::env::var_os("origen_config_paths") {
            log_trace!("Found custom config paths: {:?}", paths);
            for path in std::env::split_paths(&paths) {
                log_trace!("Looking for Origen config file at '{}'", path.display());
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "toml" {
                            files.push(path);
                        } else {
                            log_error!(
                                "Expected file {} to have extension '.toml'. Found '{}'",
                                path.display(),
                                ext.to_string_lossy()
                            )
                        }
                    } else {
                        // accept a file without an extension. will be interpreted as a .toml
                        files.push(path);
                    }
                } else if path.is_dir() {
                    let f = path.join("origen.toml");
                    if f.exists() {
                        files.push(f);
                    }
                } else {
                    log_error!(
                        "Config path {} either does not exists or is not accessible",
                        path.display()
                    );
                    exit(1);
                }
            }
        }

        if std::env::var_os("origen_bypass_config_lookup").is_some() {
            // Bypass Origen's default configuration lookup - use only the enumerated configs
            log_trace!("Bypassing Origen's Config Lookup");
        } else {
            log_trace!("Looking for Origen Configs");
            if let Some(app) = &STATUS.app {
                let mut path = app.root.join("config");
                let f = path.join("origen.toml");
                log_trace!("Looking for Origen config file at '{}'", f.display());
                if f.exists() {
                    files.push(f);
                }

                while path.pop() {
                    let f = path.join("origen.toml");
                    log_trace!("Looking for Origen config file at '{}'", f.display());
                    if f.exists() {
                        files.push(f);
                    }
                }
            }

            // TODO: Should this be the Python installation dir?
            if let Some(mut path) = STATUS.cli_location() {
                let f = path.join("origen.toml");
                log_trace!("Looking for Origen config file at '{}'", f.display());
                if f.exists() {
                    files.push(f);
                }

                while path.pop() {
                    let f = path.join("origen.toml");
                    log_trace!("Looking for Origen config file at '{}'", f.display());
                    if f.exists() {
                        files.push(f);
                    }
                }
            }
        }

        // Now add in the files, with the last one found taking lowest priority
        for f in files.iter().rev() {
            log_trace!("Loading Origen config file from '{}'", f.display());
            s = s.add_source(File::with_name(&format!("{}", f.display())));
            let built = exit_on_bad_config!(s.build_cloned());

            match built.get_array("mailer__maillists_dirs") {
                // Update any relative paths in this parameter to be relative to the config in which it was found
                Ok(paths) => {
                    if !paths.is_empty() {
                        match s.set_override(
                            "mailer__maillists_dirs",
                            _update_relative_paths(
                                &mut paths.iter().map( |m| exit_on_bad_config!(m.clone().into_string())).collect::<Vec<String>>(),
                                &f.parent().unwrap().to_path_buf()
                            )
                        ) {
                            Ok(new) => s = new,
                            Err(e) => {
                                log_error!(
                                    "Error setting maillist dir: '{}': {}",
                                    f.display(),
                                    e.to_string()
                                );
                                exit(1);
                            }
                        }
                    }
                }
                Err(e) => match e {
                    config::ConfigError::NotFound(_) => {}
                    _ => {
                        log_error!("Malformed config file: {}", f.display());
                        log_error!("{}", e);
                        exit(1);
                    }
                },
            }

            match built.get::<Option<PathBuf>>("session__user_root") {
                Ok(root) => {
                    if let Some(r) = root.as_ref() {
                        match s.set_override(
                            "session__user_root",
                            match crate::utility::file_utils::to_abs_path(r, &f.parent().unwrap().to_path_buf()) {
                                Ok(resolved) => Some(resolved.display().to_string()),
                                Err(e) => {
                                    log_error!(
                                        "Unable to process config value for 'session__user_root' (in '{}'): {}",
                                        f.display(),
                                        e
                                    );
                                    exit(1);
                                }
                            }
                        ) {
                            Ok(new) => s = new,
                            Err(e) => {
                                log_error!(
                                    "Error setting user session root: '{}': {}",
                                    f.display(),
                                    e.to_string()
                                );
                                exit(1);
                            }
                        }
                    }
                }
                Err(e) => match e {
                    config::ConfigError::NotFound(_) => {}
                    _ => {
                        log_error!("Malformed config file: {}", f.display());
                        log_error!("{}", e);
                        exit(1);
                    }
                },
            }
        }

        // Add in settings from the environment (with a prefix of ORIGEN)
        s = s.add_source(Environment::with_prefix("origen"));

        exit_on_bad_config!(exit_on_bad_config!(s.build()).try_deserialize())
    }
}

fn _update_relative_paths(paths: &mut Vec<String>, relative_to: &PathBuf) -> Vec<String> {
    for i in 0..paths.len() {
        let pb = PathBuf::from(&paths[i]);
        match crate::utility::file_utils::to_abs_path(&pb, relative_to) {
            Ok(resolved) => {
                paths[i] = resolved.display().to_string();
            }
            Err(e) => log_error!("Unable to process maillist '{}': {}", pb.display(), e),
        }
    }
    paths.to_vec()
}

impl Config {
    pub fn get_service_user(
        &self,
        username: &str,
    ) -> crate::Result<Option<&HashMap<String, String>>> {
        if let Some(u) = self.service_users.get(username) {
            Ok(Some(u))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ORIGEN_CONFIG;

    #[test]
    fn struct_is_created() {
        assert_eq!(ORIGEN_CONFIG.python_cmd, "");
    }
}
