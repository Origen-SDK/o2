use std::error::Error as BaseError;
use std::fmt;

#[derive(Debug, Clone)]
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

// To add a conversion from other type of errors

#[cfg(feature = "python")]
impl std::convert::From<Error> for pyo3::PyErr {
    fn from(err: Error) -> pyo3::PyErr {
        pyo3::exceptions::PyRuntimeError::new_err(err.to_string())
    }
}

#[cfg(feature = "python")]
impl std::convert::From<pyo3::PyErr> for Error {
    fn from(err: pyo3::PyErr) -> Self {
        pyo3::Python::with_gil(|py| {
            Error::new(&format!(
                "Encountered Exception '{}' with message: {}{}",
                err.get_type(py).name().unwrap(),
                {
                    let r = err.value(py).call_method0("__str__").unwrap();
                    r.extract::<String>().unwrap()
                },
                {
                    let tb = err.traceback(py);
                    let m = py.import("traceback").unwrap();
                    let temp = pyo3::types::PyTuple::new(py, &[tb]);
                    let et = m.call_method1("extract_tb", temp).unwrap();

                    let temp = pyo3::types::PyTuple::new(py, &[et]);
                    let text_list = m.call_method1("format_list", temp).unwrap();
                    let text = text_list.extract::<Vec<String>>().unwrap();

                    format!("\nWith traceback:\n{}", text.join(""))
                }
            ))
        })
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

//impl std::convert::From<walkdir::Error> for Error {
//    fn from(err: walkdir::Error) -> Self {
//        Error::new(&err.to_string())
//    }
//}

impl std::convert::From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<semver::Error> for Error {
    fn from(err: semver::Error) -> Self {
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

impl std::convert::From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::new(err)
    }
}

impl std::convert::From<lettre::address::AddressError> for Error {
    fn from(err: lettre::address::AddressError) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<email_address::Error> for Error {
    fn from(err: email_address::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
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

impl std::convert::From<keyring::Error> for Error {
    fn from(err: keyring::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
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

impl std::convert::From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<octocrab::Error> for Error {
    fn from(err: octocrab::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl<T> std::convert::From<std::sync::PoisonError<T>> for Error {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<std::str::ParseBoolError> for Error {
    fn from(err: std::str::ParseBoolError) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<tera::Error> for Error {
    fn from(err: tera::Error) -> Self {
        let mut msg = format!("{:?}", err.kind);
        if err.source().is_some() {
            msg += "\n";
            msg += &unchain_error(&err).join("\n");
        }
        Error::new(&msg)
    }
}

impl std::convert::From<glob::PatternError> for Error {
    fn from(err: glob::PatternError) -> Self {
        Error::new(&err.to_string())
    }
}

impl std::convert::From<minijinja::Error> for Error {
    fn from(err: minijinja::Error) -> Self {
        Error::new(&err.to_string())
    }
}

fn unchain_error(err: &dyn std::error::Error) -> Vec<String> {
    let mut msg = vec!();
    let mut inner = err.source();
    loop {
        if let Some(s) = inner {
            msg.push(s.to_string());
            inner = s.source();
        } else {
            break;
        }
    }
    msg
}