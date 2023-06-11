macro_rules! _origen {
    ($py: expr) => {
        pyo3::types::PyModule::import($py, "_origen")?
    };
}
