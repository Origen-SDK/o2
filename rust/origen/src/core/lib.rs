#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

pub mod config;
pub mod python;
pub mod term;
pub mod application;
pub mod status;

use crate::config::Config;
use crate::status::Status;

lazy_static! {
    /// Provides status information derived from the runtime environment, e.g. if an app is present
    pub static ref STATUS: Status = Status::default();
    /// Provides configuration information derived from origen.toml files found in the Origen
    /// installation and application file system paths
    pub static ref CONFIG: Config = Config::default();
}

// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
