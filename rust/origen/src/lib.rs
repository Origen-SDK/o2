#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

pub mod core;

use self::core::application::config::Config as AppConfig;
use self::core::config::Config as OrigenConfig;
use self::core::status::Status;

lazy_static! {
    /// Provides status information derived from the runtime environment, e.g. if an app is present
    pub static ref STATUS: Status = Status::default();
    /// Provides configuration information derived from origen.toml files found in the Origen
    /// installation and application file system paths
    pub static ref ORIGEN_CONFIG: OrigenConfig = OrigenConfig::default();
    /// Provides configuration information from application.toml
    pub static ref APPLICATION_CONFIG: AppConfig = AppConfig::default();
}

// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
