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

lazy_static! {
    pub static ref LOGGER: logger::Logger = logger::Logger::default();
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Context<T, E> {
    fn context(self, msg: &str) -> crate::Result<T>;
}

impl<T, E> Context<T, E> for std::result::Result<T, E>
where
    E: Display,
{
    fn context(self, msg: &str) -> crate::Result<T> {
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
