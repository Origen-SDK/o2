use std::sync::RwLock;
use crate::{Result, Error};
use rpassword;

pub struct User {
    current: bool,
    data: RwLock<Data>,
}

#[derive(Default, Debug)]
struct Data {
    password: Option<String>
}

impl User {
    pub fn current() -> User {
        User {
            current: true,
            data: RwLock::new(Data::default())
        }
    }

    pub fn password(&self, reason: Option<String>, force: bool) -> Result<String> {
        dbg!(force);
        if self.current {
            {
                let data = self.data.read().unwrap();
                dbg!(&data);
                if !force {
                    if let Some(p) = &data.password {
                        return Ok(p.clone())
                    }
                }
            }
            let msg = match reason {
                Some(x) => format!("Please enter your password {}: ", x),
                None => "Please enter your password: ".to_string(),
            };
            let pass = rpassword::read_password_from_tty(Some(&msg)).unwrap();
            let mut data = self.data.write().unwrap();
            data.password = Some(pass.clone());
            Ok(pass)
        } else {
            Err(Error::new("Can't get the password for a user which is not the current user"))
        }
    }
}