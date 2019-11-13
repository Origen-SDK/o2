#[macro_use]
extern crate lazy_static;

pub mod workspace;

lazy_static! {
    pub static ref CONFIG: workspace::Config = workspace::Config::default();
}
