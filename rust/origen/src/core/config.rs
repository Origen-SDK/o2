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

use super::term;
use crate::STATUS;
use config::{Environment, File};
use std::collections::HashMap;
use std::path::PathBuf;

lazy_static! {
    pub static ref CONFIG: Config = Config::default();
}

/// Default keys generated from crate::utility::mod::tests::check_default_key_values
/// default_encryption_key: !<<<---Origen StandardKey--->>>!
pub static DEFAULT_ENCRYPTION_KEY: &str =
    "213c3c3c2d2d2d4f726967656e205374616e646172644b65792d2d2d3e3e3e21";
/// default_encryption_nonce: ORIGEN NONCE
pub static DEFAULT_ENCRYPTION_NONCE: &str = "4f524947454e204e4f4e4345";

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
    pub mailer__server: Option<String>,
    pub mailer__port: Option<i64>,
    pub mailer__auth_method: Option<String>,
    pub mailer__service_user: Option<String>,
    pub mailer__domain: Option<String>,
    pub mailer__auth_email: Option<String>,
    pub mailer__auth_password: Option<String>,
    pub mailer__timeout_seconds: u64,
    pub mailer__maillists_dirs: Vec<String>,

    // LDAPs
    pub ldaps: HashMap<String, HashMap<String, String>>,

    // Very Basic Encryption
    pub default_encryption_key: String,
    pub default_encryption_nonce: String,

    // Various user config
    pub user__data_lookup_hierarchy: Option<Vec<String>>,
    pub user__datasets: Option<HashMap<String, HashMap<String, String>>>,
    pub user__password_auth_attempts: u8,
    pub user__password_cache_option: String,
    pub user__password_reasons: HashMap<String, String>,
    pub password_encryption_key: Option<String>,
    pub password_encryption_nonce: Option<String>,
    // pre-populated users, generally for explicit purposes such as an LDAP service user
    // or regression test launcher
    pub user__dataset_mappings: HashMap<String, HashMap<String, String>>,
    pub service_users: HashMap<String, HashMap<String, String>>,
}

impl Default for Config {
    fn default() -> Config {
        log_trace!("Instantiating Origen config");
        let mut s = config::Config::new();

        // Start off by specifying the default values for all attributes, seems fine
        // not to handle these errors
        let _ = s.set_default("python_cmd", "");
        let _ = s.set_default("pkg_server", "");
        let _ = s.set_default("pkg_server_push", "");
        let _ = s.set_default("pkg_server_pull", "");
        let _ = s.set_default("some_val", 3);
        let _ = s.set_default("mailer__server", None::<String>);
        let _ = s.set_default("mailer__port", None::<i64>);
        let _ = s.set_default("mailer__auth_method", None::<String>);
        let _ = s.set_default("mailer__domain", None::<String>);
        let _ = s.set_default("mailer__auth_email", None::<String>);
        let _ = s.set_default("mailer__auth_password", None::<String>);
        let _ = s.set_default("mailer__service_user", None::<String>);
        let _ = s.set_default("mailer__timeout_seconds", 60);
        let _ = s.set_default("mailer__maillists_dirs", Vec::<String>::new());
        let _ = s.set_default("ldaps", {
            let h: HashMap<String, HashMap<String, String>> = HashMap::new();
            h
        });
        let _ = s.set_default("user__data_lookup_hierarchy", None::<Vec<String>>);
        let _ = s.set_default("user__password_auth_attempts", 3);
        let _ = s.set_default("user__password_cache_option", "keyring");
        let _ = s.set_default("user__datasets", {
            let h: Option<HashMap<String, HashMap<String, String>>> = None;
            h
        });
        let _ = s.set_default("user__dataset_mappings", {
            let h: HashMap<String, HashMap<String, String>> = HashMap::new();
            h
        });
        let _ = s.set_default("user__password_reasons", {
            let h: HashMap<String, String> = HashMap::new();
            h
        });
        let _ = s.set_default("service_users", {
            let h: HashMap<String, HashMap<String, String>> = HashMap::new();
            h
        });

        let _ = s.set_default("default_encryption_key", DEFAULT_ENCRYPTION_KEY);
        let _ = s.set_default("default_encryption_nonce", DEFAULT_ENCRYPTION_NONCE);

        // Encryption keys specifically for passwords
        let _ = s.set_default("password_encryption_key", None::<String>);
        let _ = s.set_default("password_encryption_nonce", None::<String>);

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
                            display_redln!(
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
                    display_redln!(
                        "Config path {} either does not exists or is not accessible",
                        path.display()
                    );
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
            let mut c = config::Config::new();
            match c.merge(File::with_name(&format!("{}", f.display()))) {
                Ok(_) => {}
                Err(error) => {
                    term::redln(&format!("Malformed config file: {}", f.display()));
                    term::redln(&format!("{}", error));
                    std::process::exit(1);
                }
            }
            process_config_pre_merge(&mut c, f);
            match s.merge(c) {
                Ok(_) => {}
                Err(error) => {
                    term::redln(&format!("Malformed config file: {}", f.display()));
                    term::redln(&format!("{}", error));
                    std::process::exit(1);
                }
            }
        }

        // Add in settings from the environment (with a prefix of ORIGEN), not sure how this
        // can really fail, so not handled
        let _ = s.merge(Environment::with_prefix("origen"));

        s.try_into().unwrap()
    }
}

/// Do any processing to the config that needs to happen before attempting to merge
fn process_config_pre_merge(c: &mut config::Config, src: &PathBuf) {
    match c.get("mailer__maillists_dirs") {
        // Update any relative paths in this parameter to be relative to the config in which it was found
        Ok(mut paths) => {
            match c.set(
                "mailer__maillists_dirs",
                _update_relative_paths(&mut paths, &src.parent().unwrap().to_path_buf()),
            ) {
                Ok(_) => {}
                Err(e) => display_redln!(
                    "Error setting maillist dir: '{}': {}",
                    src.display(),
                    e.to_string()
                ),
            }
        }
        Err(e) => match e {
            config::ConfigError::NotFound(_) => {}
            _ => {
                term::redln(&format!("Malformed config file: {}", src.display()));
                term::redln(&format!("{}", e));
                std::process::exit(1);
            }
        },
    }
}

fn _update_relative_paths(paths: &mut Vec<String>, relative_to: &PathBuf) -> Vec<String> {
    for i in 0..paths.len() {
        let pb = PathBuf::from(&paths[i]);
        match crate::utility::file_utils::to_abs_path(&pb, relative_to) {
            Ok(resolved) => {
                paths[i] = resolved.display().to_string();
            }
            Err(e) => display_redln!("Unable to process maillist '{}': {}", pb.display(), e),
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
