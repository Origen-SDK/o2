use crate::core::term;
/// Exposes the application configuration options from config/application.toml
/// which will include the currently selected target settings form the workspace
use crate::STATUS;
use config::File;
use std::path::PathBuf;
use crate::core::utility::location::Location;

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
    pub website_release_location: Option<Location>,
    pub website_release_name: Option<String>,
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
        self.website_release_location = latest.website_release_location;
        self.website_release_name = latest.website_release_name;
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

        // Couldn't figure out how to get the config::Config to recongize the Location struct since the
        // underlying converter to config::value::ValueKind is private.
        // Instead, just pluck it out as string and set it to none before casting to our Config (Self)
        // Then, after the cast, put it back in as the type we want (Location)
        let loc;
        match s.get_str("website_release_location") {
            Ok(l) => loc = Some(l),
            Err(_) => loc = None
        }
        s.set("website_release_location", None::<String>).unwrap();
        let mut c: Self = s.try_into().unwrap();
        if let Some(l) = loc {
            c.website_release_location = Some(Location::new(&l));
        }
        c
    }
}
