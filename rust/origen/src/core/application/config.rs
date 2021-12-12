use crate::core::application::target::matches;
use crate::core::term;
use crate::utility::location::Location;
use config::{Environment, File};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const PUBLISHER_OPTIONS: &[&str] = &["system", "package_app", "upload_app"];

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
    pub website_release_location: Option<Location>,
    pub website_release_name: Option<String>,
    pub root: Option<PathBuf>,
    pub revision_control: Option<HashMap<String, String>>,
    pub unit_tester: Option<HashMap<String, String>>,
    pub publisher: Option<HashMap<String, String>>,
    pub linter: Option<HashMap<String, String>>,
    pub release_scribe: Option<HashMap<String, String>>,
    pub app_session_root: Option<String>,
}

impl Config {
    pub fn refresh(&mut self) {
        let latest = Self::build(self.root.as_ref().unwrap(), false);
        self.name = latest.name;
        self.target = latest.target;
        self.mode = latest.mode;
        self.reference_directory = latest.reference_directory;
        self.website_output_directory = latest.website_output_directory;
        self.website_source_directory = latest.website_source_directory;
        self.website_release_location = latest.website_release_location;
        self.website_release_name = latest.website_release_name;
        self.revision_control = latest.revision_control;
        self.unit_tester = latest.unit_tester;
        self.publisher = latest.publisher;
        self.linter = latest.linter;
        self.release_scribe = latest.release_scribe;
        self.app_session_root = latest.app_session_root;
    }

    pub fn check_defaults(root: &Path) {
        let defaults = Self::build(root, true);

        // Do some quick default checks here:
        //  * Target - tl;dr: have a better error message on invalid default targets.
        //             If the default target moves or is otherwise invalid, the app won't boot.
        //             This isn't necessarily bad (having an invalid default target is bad) but it may not be obvious,
        //             especially to newer users, as to why the app all of a sudden doesn't boot.
        //             This can be overcome by setting the target (or fixing the default), but add, remove, etc., the commands
        //             users will probably go to when encountering target problems, won't work.
        // * Stack up others as needed.
        if let Some(targets) = defaults.target {
            for t in targets.iter() {
                let m = matches(t, "targets");
                if m.len() != 1 {
                    term::redln(&format!(
                        "Error present in default target '{}' (in config/application.toml)",
                        t
                    ));
                }
            }
        }
    }

    /// Builds a new config from all application.toml files found at the given app root
    pub fn build(root: &Path, default_only: bool) -> Config {
        log_trace!("Building app config");
        let mut s = config::Config::new();

        // Start off by specifying the default values for all attributes, seems fine
        // not to handle these errors
        let _ = s.set_default("target", None::<Vec<String>>);
        let _ = s.set_default("mode", "development".to_string());
        let _ = s.set_default("revision_control", None::<HashMap<String, String>>);
        let _ = s.set_default("unit_tester", None::<HashMap<String, String>>);
        let _ = s.set_default("publisher", None::<HashMap<String, String>>);
        let _ = s.set_default("linter", None::<HashMap<String, String>>);
        let _ = s.set_default("release_scribe", None::<HashMap<String, String>>);
        let _ = s.set_default("app_session_root", None::<String>);

        // Find all the application.toml files
        let mut files: Vec<PathBuf> = Vec::new();

        let file = root.join("config").join("application.toml");
        if file.exists() {
            files.push(file);
        }

        if !default_only {
            let file = root.join(".origen").join("application.toml");
            if file.exists() {
                files.push(file);
            }
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
        let _ = s.merge(Environment::with_prefix("origen_app"));

        // Couldn't figure out how to get the config::Config to recognize the Location struct since the
        // underlying converter to config::value::ValueKind is private.
        // Instead, just pluck it out as string and set it to none before casting to our Config (Self)
        // Then, after the cast, put it back in as the type we want (Location)
        let loc;
        match s.get_str("website_release_location") {
            Ok(l) => loc = Some(l),
            Err(_) => loc = None,
        }
        s.set("website_release_location", None::<String>).unwrap();
        let mut c: Self = s.try_into().unwrap();
        c.root = Some(root.to_path_buf());
        if let Some(l) = loc {
            c.website_release_location = Some(Location::new(&l));
        }
        log_trace!("Completed building app config");
        c.validate_options();

        c
    }

    pub fn validate_options(&self) {
        log_trace!("Validating available options...");
        log_trace!("\tValidating publisher options...");
        for unknown in self.validate_publisher_options() {
            log_warning!("Unknown Publisher Option '{}'", unknown);
        }
        log_trace!("\tFinished validating publisher options");
        log_trace!("Finished checking configs!");
    }

    pub fn validate_publisher_options(&self) -> Vec<String> {
        let mut unknowns: Vec<String> = vec![];
        if let Some(p) = &self.publisher {
            for (opt, _) in p.iter() {
                if !PUBLISHER_OPTIONS.contains(&opt.as_str()) {
                    unknowns.push(opt.clone());
                }
            }
        }
        unknowns
    }
}
