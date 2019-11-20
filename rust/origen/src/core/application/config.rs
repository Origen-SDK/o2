/// Exposes the application configuration options from config/application.toml
/// which will include the currently selected target/environment settings form the workspace
use crate::STATUS;
use config::{File};
use std::path::PathBuf;
use crate::term;
use crate::application::CONFIG;

#[derive(Debug, Deserialize)]
// If you add an attribute to this you must also update:
// * to_py_dict function below to convert it to Python
// * default function below to define the default value (no nil in Rust)
// * add an example of it to src/app_generators/templates/app/config/application.toml
pub struct Config {
    pub name: String,
    pub target: String,
    pub environment: String,
}

impl Config {
    pub fn to_py_dict<'a>(self: &Self, py: &'a pyo3::Python) -> &'a pyo3::types::PyDict {
        let ret = pyo3::types::PyDict::new(*py);
        // Don't think an error can really happen here, so not handled
        let _ = ret.set_item("name", &CONFIG.name);
        let _ = ret.set_item("target", &CONFIG.target);
        let _ = ret.set_item("environment", &CONFIG.environment);
        ret
    }
}

impl Default for Config {
    fn default() -> Config {
        let mut s = config::Config::new();

        // Start off by specifying the default values for all attributes, seems fine
        // not to handle these errors
        let _ = s.set_default("name", "");
        let _ = s.set_default("target", "");
        let _ = s.set_default("environment", "");

        if STATUS.is_app_present {
            // Find all the application.toml files
            let mut files: Vec<PathBuf> = Vec::new();

            let f = STATUS.root.join("config").join("application.toml");
            if f.exists() {  files.push(f); }
            let f = STATUS.root.join("config").join(".origen").join("application.toml");
            if f.exists() { files.push(f); }

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
        }
        s.try_into().unwrap()
    }
}
