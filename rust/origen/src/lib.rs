#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
pub mod core;

use self::core::application::config::Config as AppConfig;
use self::core::config::Config as OrigenConfig;
use self::core::status::Status;
use self::core::utility::logger::Logger;

/// The available Origen runtime modes
pub const MODES: &'static [&'static str] = &["production", "development"];

lazy_static! {
    /// Provides status information derived from the runtime environment, e.g. if an app is present
    pub static ref STATUS: Status = Status::default();
    /// Provides configuration information derived from origen.toml files found in the Origen
    /// installation and application file system paths
    pub static ref ORIGEN_CONFIG: OrigenConfig = OrigenConfig::default();
    /// Provides configuration information derived from application.toml and any workspace
    /// overrides e.g. from running origen t command to set a default target
    pub static ref APPLICATION_CONFIG: AppConfig = AppConfig::default();
    pub static ref LOGGER: Logger = Logger::default();
}

// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// Sanitizes the given mode string and returns it, but will exit the process if it is invalid
pub fn clean_mode(name: &str) -> String {
    let mut matches: Vec<String> = Vec::new();

    for i in 0..MODES.len() {
        if MODES[i].contains(name) {
            matches.push(MODES[i].to_string());
        }
    }

    if matches.len() == 0 {
        println!("No mode found matching '{}', here are the available modes:", name);
        for i in 0..MODES.len() {
            println!("    {}", MODES[i].to_string());
        }
    } else if matches.len() > 1 {
        println!("'{}' is an ambiguous mode name, please try again to narrow it down to one of these:", name);
        for m in matches.iter() {
            println!("    {}", m.to_string());
        }
    } else {
        return matches[0].to_string();
    }
    std::process::exit(1);
}
