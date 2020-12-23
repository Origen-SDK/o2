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
use std::path::PathBuf;
use std::collections::HashMap;

lazy_static! {
    pub static ref CONFIG: Config = Config::default();
}

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
    pub mailer_server: Option<String>,
    pub mailer_port: Option<i64>,
    pub mailer_auth: Option<String>,
    pub mailer_domain: Option<String>,
    pub mailer_auth_email: Option<String>,
    pub mailer_auth_password: Option<String>,
    pub ldaps: HashMap<String, HashMap<String, String>>,
}

impl Default for Config {
    fn default() -> Config {
        let mut s = config::Config::new();

        // Start off by specifying the default values for all attributes, seems fine
        // not to handle these errors
        let _ = s.set_default("python_cmd", "");
        let _ = s.set_default("pkg_server", "");
        let _ = s.set_default("pkg_server_push", "");
        let _ = s.set_default("pkg_server_pull", "");
        let _ = s.set_default("some_val", 3);
        let _ = s.set_default("mailer_server", None::<String>);
        let _ = s.set_default("mailer_port", None::<i64>);
        let _ = s.set_default("mailer_auth", None::<String>);
        let _ = s.set_default("mailer_domain", None::<String>);
        let _ = s.set_default("mailer_auth_email", None::<String>);
        let _ = s.set_default("mailer_auth_password", None::<String>);
        let _ = s.set_default("ldaps", {
            let t: HashMap<String, HashMap<String, String>> = HashMap::new();
            t
        });

        // Find all the origen.toml files
        let mut files: Vec<PathBuf> = Vec::new();

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

        // Now add in the files, with the last one found taking lowest priority
        for f in files.iter().rev() {
            log_trace!("Loading Origen config file from '{}'", f.display());
            match s.merge(File::with_name(&format!("{}", f.display()))) {
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

#[cfg(test)]
mod tests {
    use crate::ORIGEN_CONFIG;

    #[test]
    fn struct_is_created() {
        assert_eq!(ORIGEN_CONFIG.python_cmd, "");
    }
}
