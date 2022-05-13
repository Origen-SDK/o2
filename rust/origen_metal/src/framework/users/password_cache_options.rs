// TODO clean up
use super::data::Data;
use super::User;
use crate::Result;
use crate::_utility::str_from_byte_array;
// use crate::utils::encryption::{decrypt_with, encrypt_with};
use crate::utils::encryption::encrypt_with;
#[cfg(feature = "password-cache")]
use keyring::Keyring;

pub const PASSWORD_KEY: &str = "user_password__";

fn to_session_password<'a>(dataset: &str) -> String {
    format!("{}{}", PASSWORD_KEY, dataset)
}

#[derive(Debug)]
pub enum PasswordCacheOptions {
    Session,
    Keyring,
    None,
}

impl PasswordCacheOptions {
    /*
        pub fn from_config() -> Result<Self> {
            let opt = &crate::ORIGEN_CONFIG.user__password_cache_option;
            match opt.as_str() {
                "session" | "session_store" => Ok(Self::Session),
                "keyring" | "true" => Ok(Self::Keyring),
                "none" | "false" => Ok(Self::None),
                _ => error!(
                    "'user__password_cache_option' option '{}' is not known!",
                    opt
                ),
            }
        }
    */
    pub fn cache_password(&self, user: &User, password: &str, dataset: &str) -> Result<bool> {
        match self {
            Self::Session => {
                // TODO
                // todo!("Session password store not supported yet!")
                log_trace!("Caching password in session store...");
                user.with_session(None, |_, _, s| {
                    s.store(
                        to_session_password(dataset),
                        str_from_byte_array(&encrypt_with(
                            password,
                            crate::into_aes_gcm_generic_array!(
                                crate::users().password_encryption_key()
                            ),
                            crate::into_aes_gcm_generic_array!(
                                crate::users().password_encryption_nonce()
                            ),
                        )?)?
                        .into(),
                    )
                })?;
                // let mut s = crate::sessions();
                // let sess = s.user_session(None)?;
                // sess.store(
                //     to_session_password(dataset),
                //     Typedvalue::String(str_from_byte_array(&encrypt_with(
                //         password,
                //         user.get_password_encryption_key()?,
                //         user.get_password_encryption_nonce()?,
                //     )?)?),
                // )?;
                Ok(true)
            }
            Self::Keyring => {
                log_trace!("Caching password in keyring...");
                let k = keyring::Entry::new(dataset, &user.id());
                k.set_password(password)?;
                Ok(true)
            }
            Self::None => {
                log_trace!("Password caching unavailable");
                Ok(false)
            }
        }
    }

    pub fn get_password(&self, user: &User, dataset: &str) -> Result<Option<String>> {
        match self {
            Self::Session => {
                // TODO
                todo!("Support getting password from session")
                // log_trace!("Checking for password in session store...");
                // Check if the password is cached in the user's session
                // let mut s = crate::sessions();
                // let sess = s.user_session(None)?;
                // if let Some(p) = sess.retrieve(&to_session_password(dataset))? {
                //     // Password should be encrypted (to avoid storing as plaintext)
                //     // Decrypt the password
                //     let pw = decrypt_with(
                //         &bytes_from_str_of_bytes(&p.as_string()?)?,
                //         user.get_password_encryption_key()?,
                //         user.get_password_encryption_nonce()?,
                //     )?;
                //     Ok(Some(pw.to_string()))
                // } else {
                //     Ok(None)
                // }
            }
            Self::Keyring => {
                log_trace!("Checking for password in keyring...");
                let k = keyring::Entry::new(dataset, &user.id());
                match k.get_password() {
                    Ok(password) => Ok(Some(password)),
                    Err(e) => match e {
                        keyring::Error::NoEntry => Ok(None),
                        _ => bail!("{}", e),
                    },
                }
            }
            Self::None => bail!("Cannot get password when password caching is unavailable!"),
        }
    }

    pub fn clear_cached_password(&self, parent: &User, dataset: &Data) -> Result<()> {
        match self {
            Self::Session => {
                // TODO
                todo!("Session passwords not suppported yet!")
                // let k = dataset.password_key();
                // if parent.is_current() {
                //     log_trace!("Clearing password {} from user session", k);
                //     crate::with_user_session(None, |session| session.delete(&k))?;
                // }
            }
            Self::Keyring => {
                let k = keyring::Entry::new(&dataset.dataset_name, &parent.id());
                match k.delete_password() {
                    Ok(_) => {}
                    Err(e) => match e {
                        keyring::Error::NoEntry => {}
                        _ => bail!("{}", e),
                    },
                }
            }
            Self::None => {}
        }
        Ok(())
    }

    pub fn is_session_store(&self) -> bool {
        match self {
            Self::Session => true,
            _ => false,
        }
    }

    pub fn is_keyring(&self) -> bool {
        match self {
            Self::Keyring => true,
            _ => false,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }
}
