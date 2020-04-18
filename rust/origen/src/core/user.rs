use crate::{Error, Result};
#[cfg(feature = "password-cache")]
use keyring::Keyring;
use std::sync::RwLock;

pub struct User {
    current: bool,
    // All user data is stored behind a RW lock so that it can be lazily loaded
    // from the environment and cached behind the scenes
    data: RwLock<Data>,
}

#[derive(Default, Debug)]
struct Data {
    password: Option<String>,
    id: Option<String>,
}

impl User {
    pub fn current() -> User {
        User {
            current: true,
            data: RwLock::new(Data::default()),
        }
    }

    pub fn id(&self) -> Option<String> {
        if self.current {
            {
                // Important, this is to release the read lock
                {
                    let data = self.data.read().unwrap();
                    if let Some(p) = &data.id {
                        return Some(p.clone());
                    }
                }
                let id = whoami::username();
                let mut data = self.data.write().unwrap();
                data.id = Some(id.clone());
                Some(id)
            }
        } else {
            let data = self.data.read().unwrap();
            data.id.clone()
        }
    }

    pub fn password(&self, reason: Option<String>, force: bool) -> Result<String> {
        if self.current {
            if !force {
                // Important, this is to release the read lock
                {
                    let data = self.data.read().unwrap();
                    if let Some(p) = &data.password {
                        return Ok(p.clone());
                    }
                }
                #[cfg(feature = "password-cache")]
                {
                    if let Some(username) = self.id() {
                        if let Some(x) = self.get_cached_password(&username) {
                            // Locally cache for next time to save accessing the external service
                            let mut data = self.data.write().unwrap();
                            data.password = Some(x.clone());
                            return Ok(x);
                        }
                    }
                }
            }
            let msg = match reason {
                Some(x) => format!("Please enter your password {}: ", x),
                None => "Please enter your password: ".to_string(),
            };
            let pass = rpassword::read_password_from_tty(Some(&msg)).unwrap();
            #[cfg(feature = "password-cache")]
            {
                if let Some(username) = self.id() {
                    self.cache_password(&username, &pass);
                }
            }
            let mut data = self.data.write().unwrap();
            data.password = Some(pass.clone());
            Ok(pass)
        } else {
            Err(Error::new(
                "Can't get the password for a user which is not the current user",
            ))
        }
    }

    #[cfg(feature = "password-cache")]
    fn cache_password(&self, username: &str, password: &str) {
        if let Some(username) = self.id() {
            let service = "rust-keyring";
            let keyring = Keyring::new(&service, &username);
            let _e = keyring.set_password(&password);
            println!("{:?}", _e);
        }
    }

    #[cfg(feature = "password-cache")]
    fn get_cached_password(&self, username: &str) -> Option<String> {
        let service = "rust-keyring";
        let keyring = Keyring::new(&service, &username);
        match keyring.get_password() {
            Ok(p) => Some(p),
            Err(_e) => {
                println!("{:?}", _e);
                None
            }
        }
    }
}
