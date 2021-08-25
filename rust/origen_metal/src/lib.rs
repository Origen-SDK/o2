#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate anyhow;
#[macro_use]
pub mod macros;

pub mod logger;
pub mod terminal;
pub mod utils;

lazy_static! {
    pub static ref LOGGER: logger::Logger = logger::Logger::default();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
