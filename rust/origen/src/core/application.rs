pub mod config;
pub mod target;

use crate::application::config::Config;

lazy_static! {
    pub static ref CONFIG: Config = Config::default();
}
