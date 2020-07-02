use crate::core::term;
use config::File;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
// If you add an attribute to this you must also update:
// * pyapi/src/lib.rs to convert it to Python
// * default function below to define the default value
// * add an example of it to src/app_generators/templates/app/config/application.toml
pub struct Config {
    pub name: String,
    pub target: Option<Vec<String>>,
    pub mode: String,
    /// Don't use this unless you know what you're doing, use origen::STATUS::output_dir() instead, since
    /// that accounts for the output directory being overridden by the current command
    pub output_directory: Option<String>,
    /// Don't use this unless you know what you're doing, use origen::STATUS::reference_dir() instead, since
    /// that accounts for the reference directory being overridden by the current command
    pub reference_directory: Option<String>,
    pub website_output_directory: Option<String>,
    pub website_source_directory: Option<String>,
    root: Option<PathBuf>,
}

impl Config {
    pub fn refresh(&mut self) {
        let latest = Self::build(self.root.as_ref().unwrap());
        self.name = latest.name;
        self.target = latest.target;
        self.mode = latest.mode;
        self.reference_directory = latest.reference_directory;
        self.website_output_directory = latest.website_output_directory;
        self.website_source_directory = latest.website_source_directory;
    }

    /// Builds a new config from all application.toml files found at the given app root
    pub fn build(root: &Path) -> Config {
        log_trace!("Building app config");
        let mut s = config::Config::new();

        // Start off by specifying the default values for all attributes, seems fine
        // not to handle these errors
        let _ = s.set_default("target", None::<Vec<String>>);
        let _ = s.set_default("mode", "development".to_string());

        // Find all the application.toml files
        let mut files: Vec<PathBuf> = Vec::new();

        let file = root.join("config").join("application.toml");
        if file.exists() {
            files.push(file);
        }
        let file = root.join(".origen").join("application.toml");
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
        let mut config: Config = s.try_into().unwrap();
        config.root = Some(root.to_path_buf());
        log_trace!("Completed building app config");
        config
    }
}
