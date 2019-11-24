pub mod config;
pub mod pyapi;
pub mod target;

use crate::application::config::Config;

lazy_static! {
    pub static ref APPLICATION_CONFIG: Config = Config::default();
}
