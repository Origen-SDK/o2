use super::data::Data;
use super::User;
use crate::Result;
use crate::_utility::{bytes_from_str_of_bytes, str_from_byte_array};
use crate::utils::encryption::{decrypt_with, encrypt_with};
use std::fmt;

pub const PASSWORD_KEY: &str = "user_password__";

fn to_session_password<'a>(dataset: &str) -> String {
    format!("{}{}", PASSWORD_KEY, dataset)
}

#[derive(Debug, Clone)]
pub enum PasswordCacheOptions {
    Session,
    Keyring,
    None,
}

impl PasswordCacheOptions {
    pub fn cache_password(&self, user: &User, password: &str, dataset: &str) -> Result<bool> {
        match self {
            // TEST_NEEDED for session caching
            Self::Session => {
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
                log_trace!("Checking for password in session store...");
                // Check if the password is cached in the user's session
                user.with_session(None, |_, _, s| {
                    if let Some(p) = s.retrieve(&to_session_password(dataset))? {
                        // Password should be encrypted (to avoid storing as plaintext)
                        // Decrypt the password
                        let pw = decrypt_with(
                            &bytes_from_str_of_bytes(&p.as_string()?)?,
                            crate::into_aes_gcm_generic_array!(
                                crate::users().password_encryption_key()
                            ),
                            crate::into_aes_gcm_generic_array!(
                                crate::users().password_encryption_nonce()
                            ),
                        )?;
                        Ok(Some(pw.to_string()))
                    } else {
                        Ok(None)
                    }
                })
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
            Self::None => Ok(None),
        }
    }

    pub fn clear_cached_password(&self, user: &User, dataset: &Data) -> Result<()> {
        match self {
            Self::Session => {
                let k = dataset.password_key();
                log_trace!("Clearing password {} from user session", k);
                user.with_session(None, |_, _, s| s.delete(&k))?;
            }
            Self::Keyring => {
                let k = keyring::Entry::new(&dataset.dataset_name, &user.id());
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

impl fmt::Display for PasswordCacheOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Session => "session",
            Self::Keyring => "keyring",
            Self::None => "none",
        })
    }
}

impl From<&PasswordCacheOptions> for Option<String> {
    fn from(value: &PasswordCacheOptions) -> Option<String> {
        match value {
            PasswordCacheOptions::None => None,
            _ => Some(value.to_string())
        }
    }
}

impl From<PasswordCacheOptions> for Option<String> {
    fn from(value: PasswordCacheOptions) -> Option<String> {
        PasswordCacheOptions::into(value)
    }
}

impl TryFrom<Option<&str>> for PasswordCacheOptions {
    type Error = crate::Error;

    fn try_from(value: Option<&str>) -> Result<PasswordCacheOptions> {
        if let Some(v) = value {
            Ok(match v.to_lowercase().as_str() {
                "session" | "session_store" => PasswordCacheOptions::Session,
                "keyring" => PasswordCacheOptions::Keyring,
                "none" => PasswordCacheOptions::None,
                _ => bail!("Invalid password cache option: '{}'", v)
            })
        } else {
            Ok(PasswordCacheOptions::None)
        }
    }
}
