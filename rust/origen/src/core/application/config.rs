use crate::core::application::target::matches;
use crate::utility::location::Location;
use crate::exit_on_bad_config;
use origen_metal::config;
use origen_metal::config::{Environment, File};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::om::glob::glob;

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
    pub commands: Option<Vec<String>>,
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
        self.commands = latest.commands;
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
                    log_error!("Error present in default target '{}' (in config/application.toml)", t);
                }
            }
        }
    }

    /// Builds a new config from all application.toml files found at the given app root
    pub fn build(root: &Path, default_only: bool) -> Config {
        log_trace!("Building app config");
        let mut s = config::Config::builder()
            .set_default("target", None::<Vec<String>>)
            .unwrap()
            .set_default("mode", "development".to_string())
            .unwrap()
            .set_default("revision_control", None::<HashMap<String, String>>)
            .unwrap()
            .set_default("unit_tester", None::<HashMap<String, String>>)
            .unwrap()
            .set_default("publisher", None::<HashMap<String, String>>)
            .unwrap()
            .set_default("linter", None::<HashMap<String, String>>)
            .unwrap()
            .set_default("release_scribe", None::<HashMap<String, String>>)
            .unwrap()
            .set_default("app_session_root", None::<String>)
            .unwrap()
            .set_default("commands", None::<Vec<String>>)
            .unwrap();

        let file = root.join("config").join("application.toml");
        if file.exists() {
            s = s.add_source(File::with_name(&format!("{}", file.display())));
        }

        if !default_only {
            let file = root.join(".origen").join("application.toml");
            if file.exists() {
                s = s.add_source(File::with_name(&format!("{}", file.display())));
            }
        }
        s = s.add_source(Environment::with_prefix("origen_app").list_separator(",").with_list_parse_key("commands").try_parsing(true));

        let cb = exit_on_bad_config!(s.build());
        let mut c: Self = exit_on_bad_config!(cb.try_deserialize());
        c.root = Some(root.to_path_buf());
        // TODO
        // if let Some(l) = loc {
        //     c.website_release_location = Some(Location::new(&l));
        // }
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

    pub fn cmd_paths(&self) -> Vec<PathBuf> {
        let mut retn = vec!();
        if let Some(cmds) = self.commands.as_ref() {
            // Load in only the commands explicitly given
            for cmds_toml in cmds {
                let ct = self.root.as_ref().unwrap().join("config").join(cmds_toml);
                if ct.exists() {
                    retn.push(ct.to_owned());
                } else {
                    log_error!("Can not locate app commands file '{}'", ct.display())
                }
            }
        } else {
            // Load in any commands from:
            // 1) app_root/commands.toml
            // 2) app_root/commands/*/**.toml
            let commands_toml = self.root.as_ref().unwrap().join("config").join("commands.toml");
            // println!("commands toml: {}", commands_toml.display());
            if commands_toml.exists() {
                retn.push(commands_toml);
            }
            let mut commands_dir = self.root.as_ref().unwrap().join("config/commands");
            if commands_dir.exists() {
                commands_dir = commands_dir.join("**/*.toml");
                for entry in glob(commands_dir.to_str().unwrap()).unwrap() {
                    match entry {
                        Ok(e) => retn.push(e),
                        Err(e) => log_error!("Error processing commands toml: {}", e)
                    }
                }
            }
        }
        retn
    }
}
