mod prog_gen_api;

use origen::testers::SupportedTester;
use pyo3::exceptions;
use pyo3::prelude::*;

#[pyclass(subclass)]
#[derive(Debug)]
/// Python interface for the tester backend.
pub struct IGXL {
    tester: SupportedTester,
}

#[pymethods]
impl IGXL {
    #[new]
    pub fn new(tester: Option<String>) -> PyResult<Self> {
        Ok(IGXL {
            tester: match &tester {
                None => SupportedTester::IGXL,
                Some(t) => {
                    let t = t.to_uppercase().replace("_", "");
                    match t.as_str() {
                        "IGXL" => SupportedTester::IGXL,
                        "J750" => SupportedTester::J750,
                        "ULTRAFLEX" => SupportedTester::ULTRAFLEX,
                        _ => {
                            return Err(PyErr::new::<exceptions::PyRuntimeError, _>(format!(
                                "IGXL tester must be 'J750' or 'ULTRAFLEX', '{}' is not supported",
                                t
                            )))
                        }
                    }
                }
            },
        })
    }
}
