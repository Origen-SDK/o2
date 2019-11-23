pub mod config;
pub mod target;
pub mod pyapi;

use crate::application::config::Config;

lazy_static! {
    pub static ref APPLICATION_CONFIG: Config = Config::default();
}
