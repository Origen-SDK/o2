#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
#[macro_use]
pub mod macros;
#[macro_use]
extern crate cfg_if;

pub mod prelude;
mod error;
pub mod framework;
pub mod frontend;
pub mod utils;
use std::fmt::Display;
use std::sync::Mutex;

pub use error::Error;
pub use utils::file;
pub use utils::outcome::Outcome;
pub use framework::typed_value::TypedValue;
pub use framework::sessions::{sessions, Sessions};

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
    pub static ref SESSIONS: Mutex<Sessions> = Mutex::new(Sessions::new());
}

pub fn unload() -> Result<()> {
    sessions().unload()?;
    Ok(())
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
    /// Get the caller name. Taken from this SO answer:
    /// https://stackoverflow.com/a/63904992/8533619
    #[macro_export]
    macro_rules! current_func {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);

            // Find and cut the rest of the path
            match &name[..name.len() - 3].rfind(':') {
                Some(pos) => &name[pos + 1..name.len() - 3],
                None => &name[..name.len() - 3],
            }
        }};
    }

    #[cfg(all(test, not(origen_skip_frontend_tests)))]
    pub fn run_python(code: &str) -> crate::Result<()> {
        let mut c = std::process::Command::new("poetry");
        c.arg("run");
        c.arg("python");
        c.arg("-c");
        c.arg(&format!("import origen_metal as om; {}", code));
        // Assume we're in the root of the Origen rust package
        let mut f = std::env::current_dir().unwrap();
        f.pop();
        f.pop();
        f.push("python/origen_metal");
        c.current_dir(f);
        println!("Running CMD: {:?}", c);
        let res = c.output().unwrap();
        println!("status: {}", res.status);
        println!("{:?}", std::str::from_utf8(&res.stdout).unwrap());
        println!("{:?}", std::str::from_utf8(&res.stderr).unwrap());
        assert_eq!(res.status.success(), true);
        Ok(())
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
