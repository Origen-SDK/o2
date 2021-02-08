use crate::Result;
use std::error::Error as BaseError;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    pub msg: String,
}

impl Error {
    pub fn new(msg: &str) -> Error {
        Error {
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl BaseError for Error {
    fn description(&self) -> &str {
        &self.msg
    }
}

// a test function that returns our error result
pub fn raises_error(yes: bool) -> Result<()> {
    if yes {
        Err(Error::new("Something has gone wrong!"))
    } else {
        Ok(())
    }
}

// To add a conversion from other type of errors
use pyo3::{exceptions, PyErr};

//impl std::convert::Into<PyErr> for Error {
//    fn into(self) -> PyErr {
//        exceptions::OSError::py_err(self.msg)
//    }
//}

impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> Self {
        exceptions::OSError::py_err(err.msg)
    }
}

impl std::convert::From<PyErr> for Error {
    fn from(err: PyErr) -> Self {
        Error::new(&format!("{:?}", err))
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<shellexpand::LookupError<std::env::VarError>> for Error {
    fn from(err: shellexpand::LookupError<std::env::VarError>) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<Error> for git2::Error {
    fn from(err: Error) -> Self {
        git2::Error::from_str(&err.msg)
    }
}

impl std::convert::From<git2::Error> for Error {
    fn from(err: git2::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<semver::SemVerError> for Error {
    fn from(err: semver::SemVerError) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<std::string::String> for Error {
    fn from(err: std::string::String) -> Self {
        Error::new(&err)
    }
}

impl std::convert::From<lettre::address::AddressError> for Error {
    fn from(err: lettre::address::AddressError) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<num_bigint::ParseBigIntError> for Error {
    fn from(err: num_bigint::ParseBigIntError) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<ldap3::LdapError> for Error {
    fn from(err: ldap3::LdapError) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<aes_gcm::Error> for Error {
    fn from(err: aes_gcm::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::new(&err.to_string())
    }
}

// On failure, the original OS string is returned
// https://doc.rust-lang.org/std/ffi/struct.OsString.html#method.into_string
impl std::convert::From<std::ffi::OsString> for Error {
    fn from(os_string: std::ffi::OsString) -> Self {
        Error::new(&format!(
            "Unable to convert OsString {:?} to String",
            os_string
        ))
    }
}
