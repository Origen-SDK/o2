use crate::term;
/// Collects the main Origen configuration options from all origen.toml files
/// found in the application and Origen installation file system paths
///
/// # Examples
///
/// ```
/// use core::ORIGEN_CONFIG;
///
/// println!("Server: {}", &ORIGEN_CONFIG.pkg_server);  // => "Server: https://pkgs.company.net:9292"
/// ```
use crate::STATUS;
use config::{Environment, File};
use std::env;
use std::path::PathBuf;

lazy_static! {
    pub static ref CONFIG: Config = Config::default();
}

#[derive(Debug, Deserialize)]
// If you add an attribute to this you must also update:
// * to_py_dict function below to convert it to Python
// * default function below to define the default value (no nil in Rust)
// * add an example of it to src/app_generators/templates/app/config/origen.toml
pub struct Config {
    pub python_cmd: String,
    pub pkg_server: String,
    pub pkg_server_push: String,
    pub pkg_server_pull: String,
    pub some_val: u32,
}

impl Config {
    pub fn to_py_dict<'a>(self: &Self, py: &'a pyo3::Python) -> &'a pyo3::types::PyDict {
        let ret = pyo3::types::PyDict::new(*py);
        // Don't think an error can really happen here, so not handled
        let _ = ret.set_item("python_cmd", &CONFIG.python_cmd);
        let _ = ret.set_item("pkg_server", &CONFIG.pkg_server);
        let _ = ret.set_item("pkg_server_push", &CONFIG.pkg_server_push);
        let _ = ret.set_item("pkg_server_pull", &CONFIG.pkg_server_pull);
        let _ = ret.set_item("some_val", &CONFIG.some_val);
        ret
    }
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

        // Find all the origen.toml files
        let mut files: Vec<PathBuf> = Vec::new();

        if STATUS.is_app_present {
            let mut path = STATUS.root.join("config");
            let f = path.join("origen.toml");
            if f.exists() {
                files.push(f);
            }

            while path.pop() {
                let f = path.join("origen.toml");
                if f.exists() {
                    files.push(f);
                }
            }
        }

        // TODO: Should this be the Python installation dir?
        let mut path = env::current_exe().unwrap();
        let f = path.join("origen.toml");
        if f.exists() {
            files.push(f);
        }

        while path.pop() {
            let f = path.join("origen.toml");
            if f.exists() {
                files.push(f);
            }
        }

        // Now add in the files, with the last one found taking lowest priority
        for f in files.iter().rev() {
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
