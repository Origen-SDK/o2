use crate::utility::location::Location;
use crate::exit_on_bad_config;
use origen_metal::{config, scrub_path};
use origen_metal::config::{Environment, File};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::om::glob::glob;
use std::process::exit;
use super::target;

const PUBLISHER_OPTIONS: &[&str] = &["system", "package_app", "upload_app"];
const BYPASS_APP_CONFIG_ENV_VAR: &str = "origen_app_bypass_config_lookup";
const APP_CONFIG_PATHS: &str = "origen_app_config_paths";

macro_rules! use_app_config {
    () => {{
        !std::env::var_os($crate::core::application::config::BYPASS_APP_CONFIG_ENV_VAR).is_some()
    }}
}

#[derive(Debug, Deserialize)]
pub struct CurrentState {
    pub target: Option<Vec<String>>
}

impl CurrentState {
    pub fn build(root: &PathBuf) -> Self {
        let file = root.join(".origen").join("application.toml");
        let mut s = config::Config::builder().set_default("target", None::<Vec<String>>).unwrap();
        if file.exists() {
            s = s.add_source(File::with_name(&format!("{}", file.display())));
        }
        let cb = exit_on_bad_config!(s.build());
        let slf: Self = exit_on_bad_config!(cb.try_deserialize());
        slf
    }

    pub fn apply_to(&mut self, config: &mut Config) {
        if let Some(t) = self.target.as_ref() {
            config.target = Some(t.to_owned())
        } else {
            if let Some(t) = &config.target {
                let clean_defaults = target::set_at_root(t.iter().map( |s| s.as_str() ).collect(), config.root.as_ref().unwrap());
                self.target = Some(clean_defaults);
            }
        }
    }

    pub fn build_and_apply(config: &mut Config) {
        if use_app_config!() {
            let mut slf = Self::build(config.root.as_ref().unwrap());
            slf.apply_to(config);
        }
    }
}

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

            let mut files: Vec<PathBuf> = Vec::new();
            if let Some(paths) = std::env::var_os(APP_CONFIG_PATHS) {
                log_trace!("Found custom config paths: {:?}", paths);
                for path in std::env::split_paths(&paths) {
                    log_trace!("Looking for Origen app config file at '{}'", path.display());
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
                        let f = path.join("application.toml");
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

        if use_app_config!() {
            let file = root.join("config").join("application.toml");
            if file.exists() {
                files.push(file);
            }
        } else {
            // Bypass Origen's default configuration lookup - use only the enumerated configs
            log_trace!("Bypassing Origen's App Config Lookup");
        }

        for f in files.iter().rev() {
            log_trace!("Loading Origen config file from '{}'", f.display());
            s = s.add_source(File::with_name(&format!("{}", f.display())));
        }
        s = s.add_source(Environment::with_prefix("origen_app").list_separator(",").with_list_parse_key("target").with_list_parse_key("commands").try_parsing(true));

        let cb = exit_on_bad_config!(s.build());
        let mut c: Self = exit_on_bad_config!(cb.try_deserialize());
        c.root = Some(root.to_path_buf());
        // TODO
        // if let Some(l) = loc {
        //     c.website_release_location = Some(Location::new(&l));
        // }
        log_trace!("Completed building app config");
        c.validate_options();
        if !default_only {
            CurrentState::build_and_apply(&mut c);
        }

        c
    }

    pub fn validate_options(&self) {
        log_trace!("Validating available options...");

        if let Some(targets) = self.target.as_ref() {
            log_trace!("\tValidating default target...");
            for t in targets {
                target::clean_name(t, "targets", true, self.root.as_ref().unwrap());
            }
            log_trace!("\tValidating default target!");
        }

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
                    log_error!("Can not locate app commands file '{}'", scrub_path!(ct).display())
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
