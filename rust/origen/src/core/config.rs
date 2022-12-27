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

use crate::STATUS;
use origen_metal::config;
use origen_metal::config::{Environment, File};
use origen_metal::prelude::config::{MailerTOMLConfig, MaillistsTOMLConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;

// TODO Get from a prelude?
use crate::om;
use om::framework::users::DatasetConfig as OMDatasetConfig;

#[derive(Debug, Deserialize)]
pub struct LDAPConfig {
    pub server: String,
    pub base: String,
    pub auth: Option<HashMap<String, config::Value>>,
    pub continuous_bind: Option<bool>,
    pub populate_user_config: Option<HashMap<String, config::Value>>,
    pub timeout: Option<i32>,
}

impl std::convert::From<LDAPConfig> for config::ValueKind {
    fn from(_value: LDAPConfig) -> Self {
        Self::Nil
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatasetConfig {
    category: Option<String>,
    data_store: Option<String>,
    auto_populate: Option<bool>,
    should_validate_password: Option<bool>,
}

impl std::convert::TryFrom<&DatasetConfig> for OMDatasetConfig {
    type Error = om::Error;

    fn try_from(value: &DatasetConfig) -> Result<Self, Self::Error> {
        OMDatasetConfig::new(
            value.category.clone(),
            value.data_store.clone(),
            value.auto_populate,
            value.should_validate_password,
        )
    }
}

impl std::convert::From<DatasetConfig> for config::ValueKind {
    fn from(_value: DatasetConfig) -> Self {
        Self::Nil
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefaultUserConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    // TODO support custom parameters (placed in "other")
    // pub full_name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub auto_populate: Option<bool>,
    pub should_validate_passwords: Option<bool>,
}

impl std::convert::From<DefaultUserConfig> for config::ValueKind {
    fn from(_value: DefaultUserConfig) -> Self {
        Self::Nil
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InitialUserConfig {
    pub initialize: Option<bool>,
    pub init_home_dir: Option<bool>,
}

impl std::convert::From<InitialUserConfig> for config::ValueKind {
    fn from(_value: InitialUserConfig) -> Self {
        Self::Nil
    }
}

// TODO likely need a user config
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct UserConfig {
//     pub user__data_lookup_hierarchy: Option<Vec<String>>,
//     pub user__datasets: Option<HashMap<String, HashMap<String, String>>>,
//     pub user__password_auth_attempts: u8,
//     pub user__password_cache_option: String,
//     pub user__password_reasons: HashMap<String, String>,
//     pub password_encryption_key: Option<String>,
//     pub password_encryption_nonce: Option<String>,
//     // pre-populated users, generally for explicit purposes such as an LDAP service user
//     // or regression test launcher
//     pub user__dataset_mappings: HashMap<String, HashMap<String, String>>,
//     pub service_users: HashMap<String, HashMap<String, String>>,
//     // User session root path
//     pub session__user_root: Option<String>,
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginConfig {
    pub name: String,
}

impl std::convert::From<PluginConfig> for config::ValueKind {
    fn from(_value: PluginConfig) -> Self {
        Self::Nil
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PluginsConfig {
    pub collect: Option<bool>,
    pub load: Option<Vec<PluginConfig>>,
}

impl PluginsConfig {
    pub fn collect(&self) -> bool {
        self.collect.unwrap_or(true)
    }

    pub fn should_collect_any(&self) -> bool {
        self.collect() || self.load.is_some()
    }
}

impl std::convert::From<PluginsConfig> for config::ValueKind {
    fn from(_value: PluginsConfig) -> Self {
        Self::Nil
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuxillaryCommandsTOML {
    pub name: Option<String>,
    pub path: String,
}

impl AuxillaryCommandsTOML {
    pub fn set_override<St: config::builder::BuilderState>(&self, mut config: config::ConfigBuilder<St>, i: usize) -> config::ConfigBuilder<St> {
        config = config.set_override(format!("auxillary_commands[{}].path", i), self.path.to_string()).unwrap();
        config = config.set_override(format!("auxillary_commands[{}].name", i), self.name.to_owned()).unwrap();
        config
    }

    pub fn path(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.path);
        path.set_extension("toml");
        path
    }
}

impl std::convert::From<AuxillaryCommandsTOML> for config::ValueKind {
    fn from(_value: AuxillaryCommandsTOML) -> Self {
        Self::Nil
    }
}

#[macro_export]
macro_rules! exit_on_bad_config {
    ($result: expr) => {
        match $result {
            Ok(r) => r,
            Err(e) => {
                log_error!("Malformed config file");
                log_error!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

#[derive(Default)]
pub struct ConfigMetadata {
    pub files: Vec<PathBuf>,
    pub aux_cmd_sources: Vec<PathBuf>,
}

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

    // Plugins
    pub plugins: Option<PluginsConfig>,

    // Mailer
    pub maillists: Option<MaillistsTOMLConfig>,
    pub mailer: Option<MailerTOMLConfig>,

    // LDAPs
    pub ldaps: HashMap<String, LDAPConfig>,

    // Very Basic Encryption
    // TEST_NEEDED ensure these get set in OM
    pub default_encryption_key: Option<String>,
    pub default_encryption_nonce: Option<String>,

    // Various user config
    pub user__data_lookup_hierarchy: Option<Vec<String>>,
    pub user__datasets: Option<HashMap<String, DatasetConfig>>,
    pub user__password_auth_attempts: u8,
    pub user__password_cache_option: Option<String>,
    pub user__current_user_lookup_function: Option<String>,
    // pub user__password_reasons: HashMap<String, String>,
    pub user__dataset_motives: HashMap<String, String>,
    pub password_encryption_key: Option<String>,
    pub password_encryption_nonce: Option<String>,
    // pre-populated users, generally for explicit purposes such as an LDAP service user
    // or regression test launcher
    pub user__dataset_mappings: HashMap<String, HashMap<String, String>>,
    pub default_users: HashMap<String, DefaultUserConfig>,
    pub initial_user: Option<InitialUserConfig>,
    // User session root path
    pub session__user_root: Option<String>,

    pub additional_config_dirs: Option<Vec<String>>,
    pub additional_configs: Option<Vec<String>>,
    pub auxillary_commands: Option<Vec<AuxillaryCommandsTOML>>,
}

impl Config {
    fn append_configs(mut starting_path: PathBuf, file_list: &mut Vec<PathBuf>) {
        let f = starting_path.join("origen.toml");
        log_trace!("Looking for Origen config file at '{}'", f.display());
        if f.exists() {
            file_list.push(f);
        }

        while starting_path.pop() {
            let f = starting_path.join("origen.toml");
            log_trace!("Looking for Origen config file at '{}'", f.display());
            if f.exists() {
                file_list.push(f);
            }
        }
    }

    pub fn should_collect_plugins(&self) -> bool {
        self.plugins.as_ref().map_or(true, |pl_config| pl_config.should_collect_any())
    }
}

impl Default for Config {
    fn default() -> Config {
        log_trace!("Instantiating Origen config");
        let mut s = config::Config::builder();

        // Start off by specifying the default values for all attributes, seems fine
        // not to handle these errors
        s = s.set_default("python_cmd", "").unwrap();
        s = s.set_default("pkg_server", "").unwrap();
        s = s.set_default("pkg_server_push", "").unwrap();
        s = s.set_default("pkg_server_pull", "").unwrap();
        s = s.set_default("some_val", 3).unwrap();
        s = s.set_default("plugins", None::<PluginsConfig>).unwrap();
        s = s.set_default("maillists", None::<MaillistsTOMLConfig>).unwrap();
        s = s.set_default("mailer", None::<MailerTOMLConfig>).unwrap();
        s = s
            .set_default("ldaps", {
                let h: HashMap<String, LDAPConfig> = HashMap::new();
                h
            })
            .unwrap();
        s = s
            .set_default("user__data_lookup_hierarchy", None::<Vec<String>>)
            .unwrap();
        s = s.set_default("user__password_auth_attempts", 3).unwrap();
        s = s
            .set_default("user__password_cache_option", None::<String>)
            .unwrap();
        s = s
            .set_default("user__datasets", {
                let h: Option<HashMap<String, DatasetConfig>> = None;
                h
            })
            .unwrap();
        s = s
            .set_default("user__dataset_mappings", {
                let h: HashMap<String, HashMap<String, String>> = HashMap::new();
                h
            })
            .unwrap();
        s = s
            .set_default("user__dataset_motives", {
                let h: HashMap<String, String> = HashMap::new();
                h
            })
            .unwrap();
        s = s
            .set_default("default_users", {
                let h: HashMap<String, DefaultUserConfig> = HashMap::new();
                h
            })
            .unwrap();
        
        s = s.set_default("initial_user", None::<InitialUserConfig>).unwrap();

        s = s
            .set_default("default_encryption_key", None::<String>)
            .unwrap();
        s = s
            .set_default("default_encryption_nonce", None::<String>)
            .unwrap();
        s = s.set_default("user__current_user_lookup_function", None::<String>).unwrap();

        // Encryption keys specifically for passwords
        s = s
            .set_default("password_encryption_key", None::<String>)
            .unwrap();
        s = s
            .set_default("password_encryption_nonce", None::<String>)
            .unwrap();

        // Session setup
        s = s.set_default("session__user_root", None::<String>).unwrap();
        s = s.set_default("auxillary_commands", None::<Vec<AuxillaryCommandsTOML>>).unwrap();

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
                    let f = path.join("origen.toml");
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
            } else {
                match std::env::current_dir() {
                    Ok(path) => {
                        let f = path.join("origen.toml");
                        log_trace!("Looking for Origen config file in current working directory at '{}'", f.display());
                        if f.exists() {
                            files.push(f);
                        }
                    }
                    Err(e) => {
                        log_error!("Failed to lookup current working directory: {}", e.to_string())
                    }
                }
            }


            // Check for configs in the Python install directory and its parents
            if let Some(path) = STATUS.fe_exe_loc().as_ref() {
                log_trace!("Looking for Origen config files from frontend install directory: '{}'", path.display());
                Self::append_configs(path.to_owned(), &mut files);
            }

            // Check for configs in the Origen package directory and its parents
            // Depending on the virtual env setups, this could be different
            if let Some(path) = STATUS.fe_pkg_loc().as_ref() {
                log_trace!("Looking for Origen config files from frontend package directory: '{}'", path.display());
                Self::append_configs(path.to_owned(), &mut files);
            }

            // Check for configs in the CLI directory and its parents
            if let Some(mut path) = STATUS.cli_location() {
                log_trace!("Looking for Origen config files from the CLI directory: '{}'", path.display());
                Self::append_configs(path, &mut files);
            }
        }

        let mut all_cmds: Vec<AuxillaryCommandsTOML> = vec!();
        let mut aux_cmd_sources: Vec<PathBuf> = vec!();
        // Now add in the files, with the last one found taking lowest priority
        for f in files.iter().rev() {
            log_trace!("Loading Origen config file from '{}'", f.display());
            s = s.add_source(File::with_name(&format!("{}", f.display())));
            let built = exit_on_bad_config!(s.build_cloned());

            match built.get::<Option<MaillistsTOMLConfig>>("maillists") {
                // Update any relative paths in this parameter to be relative to the config in which it was found
                Ok(r) => {
                    if let Some(_mls_config) = r {
                        match s.set_override("maillists.src_dir", f.parent().unwrap().display().to_string()) {
                            Ok(new) => s = new,
                            Err(e) => {
                                log_error!(
                                    "Error processing maillists config from '{}': {}",
                                    f.display(),
                                    e.to_string()
                                );
                                exit(1);
                            }
                        }
                    }
                }
                Err(e) => match e {
                    config::ConfigError::NotFound(_) => {}
                    _ => {
                        log_error!("Malformed config file: {}", f.display());
                        log_error!("{}", e);
                        exit(1);
                    }
                },
            }

            match built.get::<Option<PathBuf>>("session__user_root") {
                Ok(root) => {
                    if let Some(r) = root.as_ref() {
                        match s.set_override(
                            "session__user_root",
                            om::_utility::file_utils::to_abs_path(r.display(), &f.parent().unwrap().to_path_buf()).display().to_string()
                        ) {
                            Ok(new) => s = new,
                            Err(e) => {
                                log_error!(
                                    "Error setting user session root: '{}': {}",
                                    f.display(),
                                    e.to_string()
                                );
                                exit(1);
                            }
                        }
                    }
                }
                Err(e) => match e {
                    config::ConfigError::NotFound(_) => {}
                    _ => {
                        log_error!("Malformed config file: {}", f.display());
                        log_error!("{}", e);
                        exit(1);
                    }
                },
            }

            match built.get::<Option<Vec<AuxillaryCommandsTOML>>>("auxillary_commands") {
                Ok(r) => {
                    if let Some(cmds) = r {
                        for (i, cmd) in cmds.iter().enumerate() {
                            let mut cmd_clone = cmd.clone();
                            cmd_clone.path = om::_utility::file_utils::to_abs_path(&cmd.path, &f.parent().unwrap().to_path_buf()).display().to_string();
                            aux_cmd_sources.push(f.to_path_buf());
                            all_cmds.push(cmd_clone);
                        }
                    }
                }
                Err(e) => match e {
                    config::ConfigError::NotFound(_) => {}
                    _ => {
                        log_error!("Malformed config file: {}", f.display());
                        log_error!("{}", e);
                        exit(1);
                    }
                },
            }
        }

        // Add in settings from the environment (with a prefix of ORIGEN)
        s = s.add_source(Environment::with_prefix("origen"));
        crate::set_origen_config_metadata(ConfigMetadata {
            files: files,
            aux_cmd_sources: aux_cmd_sources,
        });
        if !all_cmds.is_empty() {
            for (i, cmd) in all_cmds.iter().enumerate() {
                s = cmd.set_override(s, i);
            }
        }
        exit_on_bad_config!(exit_on_bad_config!(s.build()).try_deserialize())
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
