#[macro_export]
macro_rules! bail_with_runtime_error {
    ($message:expr) => {{
        Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>($message))
    }};
}

#[macro_export]
macro_rules! runtime_error {
    ($message:expr) => {{
        Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>($message))
    }};
}

#[macro_export]
macro_rules! runtime_exception {
    ($message:expr) => {{
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>($message)
    }};
}

#[macro_export]
macro_rules! type_error {
    ($message:expr) => {
        Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "{}",
            $message
        )))
    };
}

#[macro_export]
macro_rules! key_error {
    ($message:expr) => {
        Err(pyo3::exceptions::PyKeyError::new_err(format!(
            "{}",
            $message
        )))
    };
}

#[macro_export]
macro_rules! key_exception {
    ($message:expr) => {
        pyo3::exceptions::PyKeyError::new_err(format!(
            "{}",
            $message
        ))
    };
}

#[macro_export]
macro_rules! not_implemented_error {
    ($message:expr) => {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(format!(
            "{}",
            $message
        )))
    };
}

#[macro_export]
macro_rules! value_error {
    ($message:expr) => {
        Err(pyo3::exceptions::PyValueError::new_err(format!(
            "{}",
            $message
        )))
    };
}
