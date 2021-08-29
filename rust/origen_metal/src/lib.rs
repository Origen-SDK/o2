#[macro_use]
extern crate lazy_static;
#[macro_use]
pub mod macros;

mod error;
pub mod logger;
pub mod terminal;
pub mod utils;
use std::fmt::Display;

pub use error::Error;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

lazy_static! {
    pub static ref LOGGER: logger::Logger = logger::Logger::default();
    pub static ref VERSION: &'static str = built_info::PKG_VERSION;
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
