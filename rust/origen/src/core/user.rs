use crate::revision_control::git;
use crate::utility::command_helpers::exec_and_capture;
use crate::{Error, Result};
#[cfg(feature = "password-cache")]
use keyring::Keyring;
use std::sync::{Mutex, RwLock};

pub struct User {
    current: bool,
    // All user data is stored behind a RW lock so that it can be lazily loaded
    // from the environment and cached behind the scenes
    data: RwLock<Data>,
    password_semaphore: Mutex<u8>,
}

#[derive(Default, Debug)]
struct Data {
    password: Option<String>,
    id: Option<String>,
    name: Option<String>,
    // Will be set after trying to get a missing name, e.g. from the
    // Git config to differentiate between an name which has not been
    // looked up and name which has been looked up but which could not
    // be found.
    name_tried: bool,
    email: Option<String>,
    email_tried: bool,
}

impl User {
    pub fn current() -> User {
        User {
            current: true,
            data: RwLock::new(Data::default()),
            password_semaphore: Mutex::new(0),
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
                // The whoami crate returned a garbage user name when compiled into a release binary,
                // so doing it the old fashioned way. Hopefully it still works on Windows!
                let mut id = "".to_string();
                if cfg!(unix) {
                    let output = exec_and_capture("whoami", None);
                    if let Ok((status, mut lines, _stderr)) = output {
                        if status.success() {
                            id = lines.pop().unwrap();
                        } else {
                            log_debug!("Failed to run 'whoami'");
                        }
                    } else {
                        log_debug!("Failed to run 'whoami'");
                        return None;
                    }
                    log_debug!("User ID read from the system: '{}'", &id);
                } else {
                    id = whoami::username();
                    log_debug!("User ID read from whoami: '{}'", &id);
                }
                let mut data = self.data.write().unwrap();
                data.id = Some(id.clone());
                Some(id)
            }
        } else {
            let data = self.data.read().unwrap();
            data.id.clone()
        }
    }

    pub fn name(&self) -> Option<String> {
        if self.current {
            {
                let mut data = self.data.write().unwrap();

                if let Some(name) = &data.name {
                    return Some(name.to_string());
                }
                if data.name_tried {
                    return None;
                }
                let name = git::config("name");
                data.name_tried = true;
                data.name = name.clone();
                return name;
            }
        } else {
            let data = self.data.read().unwrap();
            data.name.clone()
        }
    }

    pub fn set_name(&self, name: &str) {
        let mut data = self.data.write().unwrap();
        data.name = Some(name.to_string());
    }

    pub fn email(&self) -> Option<String> {
        if self.current {
            {
                let mut data = self.data.write().unwrap();

                if let Some(email) = &data.email {
                    return Some(email.to_string());
                }
                if data.email_tried {
                    return None;
                }
                let email = git::config("email");
                data.email_tried = true;
                data.email = email.clone();
                return email;
            }
        } else {
            let data = self.data.read().unwrap();
            data.email.clone()
        }
    }

    pub fn set_email(&self, email: &str) {
        let mut data = self.data.write().unwrap();
        data.email = Some(email.to_string());
    }

    pub fn password(&self, reason: Option<&str>, failed_password: Option<&str>) -> Result<String> {
        if self.current {
            // In a multi-threaded scenario, this prevents concurrent threads from prompting the user for
            // the password at the same time.
            // Instead the first thread to arrive will do it, then by the time the lock is released awaiting
            // threads will be able to used the cached value instead of prompting the user.
            let _lock = self.password_semaphore.lock().unwrap();
            // Important, this is to release the read lock
            {
                let data = self.data.read().unwrap();
                if let Some(p) = &data.password {
                    match failed_password {
                        None => return Ok(p.clone()),
                        Some(fp) => {
                            if p != fp {
                                return Ok(p.clone());
                            }
                        }
                    }
                }
            }
            #[cfg(feature = "password-cache")]
            {
                let mut password: Option<String> = None;
                if let Some(username) = self.id() {
                    if let Some(p) = self.get_cached_password(&username) {
                        match failed_password {
                            None => password = Some(p),
                            Some(fp) => {
                                if p != fp {
                                    password = Some(p)
                                }
                            }
                        }
                    }
                    if let Some(p) = password {
                        // Locally cache for next time to save accessing the external service
                        let mut data = self.data.write().unwrap();
                        data.password = Some(p.clone());
                        return Ok(p);
                    }
                }
            }
            let msg = match reason {
                Some(x) => format!("\nPlease enter your password {}: ", x),
                None => "\nPlease enter your password: ".to_string(),
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
        let service = "rust-keyring";
        let keyring = Keyring::new(&service, &username);
        let _e = keyring.set_password(&password);
        println!("{:?}", _e);
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
