macro_rules! _origen {
    ($py: expr) => {
        pyo3::types::PyModule::import($py, "_origen")?
    };
}

#[allow(unused_macros)]
macro_rules! origen {
    ($py: expr) => {
        pyo3::types::PyModule::import($py, "origen")?
    };
}

macro_rules! get_plugin {
    ($py: expr, $plugin: expr) => {{
        let m = pyo3::types::PyModule::import($py, "origen")?;
        let pls = m.getattr("plugins")?;
        pls.get_item($plugin)?
    }};
}
