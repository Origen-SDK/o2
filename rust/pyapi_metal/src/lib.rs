mod utils;

use pyo3::prelude::*;
use pyo3::py_run;

#[pymodule]
fn _origen_metal(py: Python, m: &PyModule) -> PyResult<()> {
    utils::define(py, m)?;
    Ok(())
}

fn py_submodule<F>(py: Python, parent: &PyModule, path: &str, func: F) -> PyResult<()>
where
    F: FnOnce(&PyModule) -> PyResult<()>,
{
    let m = PyModule::new(py, "differ")?;
    func(m)?;
    // py_run! is quick-and-dirty; should be replaced by PyO3 API calls in actual code
    py_run!(py, m, &format!("import sys; sys.modules['{}'] = m", path));
    parent.add_submodule(m)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
