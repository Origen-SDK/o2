mod user;

use pyo3::prelude::*;

#[pyfunction]
pub fn ping() -> PyResult<String> {
    Ok("pong".to_string())
}

#[pymodule]
fn _origen_metal(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ping, m)?)?;
    user::register(py, m)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
