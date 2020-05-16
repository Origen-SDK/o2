use crate::core::term;
/// Exposes the application configuration options from config/application.toml
/// which will include the currently selected target settings form the workspace
use crate::STATUS;
use config::File;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
// If you add an attribute to this you must also update:
// * pyapi/src/lib.rs to convert it to Python
// * default function below to define the default value
// * add an example of it to src/app_generators/templates/app/config/application.toml
pub struct Config {
    pub name: String,
    pub target: Option<Vec<String>>,
    pub mode: String,
    pub output_directory: Option<String>,
    pub website_output_directory: Option<String>,
    pub website_source_directory: Option<String>,
}

impl Config {
    pub fn refresh(&mut self) {
        let latest = Self::default();
        self.name = latest.name;
        self.target = latest.target;
        self.mode = latest.mode;
        self.output_directory = latest.output_directory;
        self.website_output_directory = latest.website_output_directory;
        self.website_source_directory = latest.website_source_directory;
    }
}

impl Default for Config {
    fn default() -> Config {
        let mut s = config::Config::new();

        // Start off by specifying the default values for all attributes, seems fine
        // not to handle these errors
        let _ = s.set_default("target", None::<Vec<String>>);
        let _ = s.set_default("mode", "development".to_string());

        if STATUS.is_app_present {
            // Find all the application.toml files
            let mut files: Vec<PathBuf> = Vec::new();

            let file = STATUS.root.join("config").join("application.toml");
            if file.exists() {
                files.push(file);
            }
            let file = STATUS.root.join(".origen").join("application.toml");
            if file.exists() {
                files.push(file);
            }

            // Now add in the files, with the last one found taking highest priority
            for file in files.iter() {
                match s.merge(File::with_name(&format!("{}", file.display()))) {
                    Ok(_) => {}
                    Err(error) => {
                        term::redln(&format!("Malformed config file: {}", file.display()));
                        term::redln(&format!("{}", error));
                        std::process::exit(1);
                    }
                }
            }
        }
        s.try_into().unwrap()
    }
}
