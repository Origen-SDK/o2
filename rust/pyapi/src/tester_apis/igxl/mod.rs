mod test_instances;

use origen::testers::SupportedTester;
use pyo3::exceptions;
use pyo3::prelude::*;

use test_instances::TestInstances;

#[pyclass(subclass)]
#[derive(Debug)]
/// Python interface for the tester backend.
pub struct IGXL {
    tester: SupportedTester,
}

#[pymethods]
impl IGXL {
    #[new]
    fn new(tester: Option<String>) -> PyResult<Self> {
        Ok(IGXL {
            tester: match tester {
                None => SupportedTester::IGXL,
                Some(t) => {
                    let t = t.to_uppercase().replace("_", "");
                    match t.as_str() {
                        "J750" => SupportedTester::J750,
                        "ULTRAFLEX" => SupportedTester::ULTRAFLEX,
                        _ => {
                            return Err(PyErr::new::<exceptions::RuntimeError, _>(format!(
                                "IGXL tester must be 'J750' or 'ULTRAFLEX', '{}' is not supported",
                                t
                            )))
                        }
                    }
                }
            },
        })
    }

    #[getter]
    pub fn test_instances(&self) -> PyResult<TestInstances> {
        Ok(TestInstances {
            tester: self.tester.clone(),
        })
    }
}
