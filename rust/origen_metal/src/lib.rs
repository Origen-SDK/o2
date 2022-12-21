#[macro_use]
pub extern crate lazy_static;
pub extern crate config;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate pest_derive;
#[macro_use]
pub mod macros;
#[macro_use]
pub extern crate cfg_if;
#[macro_use]
extern crate enum_display_derive;
pub mod _utility;

pub mod prelude;

pub mod ast;
mod error;
pub mod framework;
pub mod frontend;
pub mod stil;
pub mod utils;
use std::fmt::Display;
use std::sync::Mutex;

pub use error::Error;
pub use framework::sessions::{sessions, Sessions};
pub use framework::typed_value::Map as TypedValueMap;
pub use framework::typed_value::TypedValue;
pub use framework::typed_value::TypedValueVec;
pub use frontend::{with_frontend, with_optional_frontend};
pub use utils::file;
pub use utils::outcome::{Outcome, OutcomeState, OutcomeSubTypes, OutcomeSubtypes};
// TODO make a prelude out of this?
pub use framework::users::users::{
    add_user, clear_current_user, get_current_user_email, get_current_user_home_dir,
    get_current_user_id, get_initial_user_id, require_current_user_email,
    require_current_user_home_dir, require_current_user_id, set_current_user,
    try_lookup_and_set_current_user, try_lookup_current_user, users, users_mut, with_current_user,
    with_current_user_session, with_user, with_user_mut, with_users, with_users_mut, with_user_or_current,
};
// TODO and this?
pub use framework::users::user::{
    add_dataset_to_user, register_dataset_with_user, with_user_dataset, with_user_dataset_mut,
    with_user_hierarchy, with_user_motive_or_default
};
pub use framework::users::users::unload as unload_users;
pub use utils::os::on_linux as running_on_linux;
pub use utils::os::on_windows as running_on_windows;

use self::framework::users::users::Users;
use self::frontend::Frontend;
use std::sync::RwLock;

pub type RefStrAble = dyn AsRef<str> + Send + Sync;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

lazy_static! {
    pub static ref LOGGER: framework::logger::Logger = framework::logger::Logger::default();
    pub static ref VERSION: &'static str = built_info::PKG_VERSION;
    pub static ref FRONTEND: RwLock<Frontend> = RwLock::new(Frontend::new());
    pub static ref SESSIONS: Mutex<Sessions> = Mutex::new(Sessions::new());
    pub static ref USERS: RwLock<Users> = RwLock::new(Users::default());
}

pub fn unload() -> Result<()> {
    sessions().unload()?;
    // TODO add users unload, and probably frontend too
    users_mut().unload(true)?;
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
