#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate pest_derive;
#[macro_use]
pub mod macros;

pub mod prelude;

pub mod ast;
mod error;
pub mod framework;
pub mod frontend;
pub mod stil;
pub mod utils;
use std::fmt::Display;

pub use error::Error;
pub use utils::outcome::Outcome;

use self::frontend::Frontend;
use std::sync::RwLock;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

lazy_static! {
    pub static ref LOGGER: framework::logger::Logger = framework::logger::Logger::default();
    pub static ref VERSION: &'static str = built_info::PKG_VERSION;
    pub static ref FRONTEND: RwLock<Frontend> = RwLock::new(Frontend::new());
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Context<T, E, M> {
    fn context(self, msg: M) -> crate::Result<T>;
}

impl<T, E, M> Context<T, E, M> for std::result::Result<T, E>
where
    E: Display,
    M: Display,
{
    fn context(self, msg: M) -> crate::Result<T> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(crate::Error {
                msg: format!("{}\n - {}", e, msg),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
