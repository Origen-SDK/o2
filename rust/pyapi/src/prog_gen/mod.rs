//! Implements Python bindings for program generation data structures and functions

pub mod interface;

use origen::testers::SupportedTester;
use origen::PROG;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::prelude::*;
use pyo3::types::PyAny;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Test {
    pub id: usize,
    pub name: String,
    pub tester: SupportedTester,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct TestInvocation {
    pub id: usize,
    pub test_id: Option<usize>,
    pub tester: SupportedTester,
}

#[pymethods]
impl TestInvocation {
    fn set_test_obj(&mut self, test: Test) -> PyResult<()> {
        if test.tester != self.tester {
            //return error!("Blah");
        }
        log_info!("Set test object called!");
        self.test_id = Some(test.id);
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for TestInvocation {
    //fn __repr__(&self) -> PyResult<String> {
    //    Ok("Hello".to_string())
    //}
    fn __getattr__(&self, query: &str) -> PyResult<()> {
        dbg!(query);
        Ok(())
    }

    fn __setattr__(&mut self, name: &str, value: &PyAny) -> PyResult<()> {
        if name == "test_method"
            && (self.tester == SupportedTester::V93KSMT7
                || self.tester == SupportedTester::V93KSMT8)
        {
            self.set_test_obj(value.extract::<Test>()?)?;
            return Ok(());
        }
        //if PROG::foor

        dbg!(name);
        dbg!(value);
        Ok(())
    }
}

impl TestInvocation {}
