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

// To add a conversion from another type of error

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
