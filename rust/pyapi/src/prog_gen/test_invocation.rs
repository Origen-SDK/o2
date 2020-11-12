use super::to_param_value;
use super::Test;
use crate::utility::caller::src_caller_meta;
use origen::prog_gen::{flow_api, ParamValue};
use origen::testers::SupportedTester;
use origen::Result;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::exceptions::AttributeError;
use pyo3::prelude::*;

/// A test invocation models a particular call (or invocation) of a test from a test flow,
/// on the V93K a TestInvocation maps to a TestSuite, while a Test maps to a TestMethod.
/// See the Test description for comments on the multiple ID fields.
#[pyclass]
#[derive(Debug, Clone)]
pub struct TestInvocation {
    pub name: String,
    pub tester: SupportedTester,
    pub id: usize,
}

#[pymethods]
impl TestInvocation {
    pub fn set_test_obj(&mut self, test: Test) -> PyResult<()> {
        if !test.tester.is_compatible_with(&self.tester) {
            return Err(AttributeError::py_err(format!(
                "Attempted to associate a test for '{}' with an invocation for '{}'",
                test.tester, self.tester
            )));
        }
        flow_api::assign_test_to_invocation(self.id, test.id, None)?;
        Ok(())
    }
}

impl TestInvocation {
    pub fn new(name: String, tester: SupportedTester) -> Result<TestInvocation> {
        let id = flow_api::define_test_invocation(&name, &tester, src_caller_meta())?;

        Ok(TestInvocation {
            name: name,
            tester: tester,
            id: id,
        })
    }

    pub fn set_attr(&self, name: &str, value: ParamValue) -> Result<()> {
        flow_api::set_test_attr(self.id, name, value, src_caller_meta())?;
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for TestInvocation {
    //fn __repr__(&self) -> PyResult<String> {
    //    Ok("Hello".to_string())
    //}

    //fn __getattr__(&self, _query: &str) -> PyResult<()> {
    //    Ok(())
    //}

    fn __setattr__(&mut self, name: &str, value: &PyAny) -> PyResult<()> {
        // Specials for platform specific attributes
        if name == "test_method"
            && (self.tester == SupportedTester::V93KSMT7
                || self.tester == SupportedTester::V93KSMT8
                || self.tester == SupportedTester::V93K)
        {
            let test = value.extract::<Test>()?;
            return self.set_test_obj(test);
        }
        self.set_attr(name, to_param_value(value)?)?;
        Ok(())

        //if origen::with_prog_mut(|p| {
        //    // Try and set the attribute on the test invocation
        //    let t = &mut p.tests[self.id];
        //    if t.has_param(name) {
        //        set_value(t, name, value)?;
        //        return Ok(true);
        //    } else {
        //        // Try and set the attribute on the test (if present)
        //        if let Some(id) = self.test_id {
        //            let t = &mut p.tests[id];
        //            if t.has_param(name) {
        //                set_value(t, name, value)?;
        //                Ok(true)
        //            } else {
        //                Ok(false)
        //            }
        //        } else {
        //            Ok(false)
        //        }
        //    }
        //})? {
        //    return Ok(());
        //}
        //// Tried our best
        //let msg = match self.test_id {
        //    Some(_id) => format!(
        //        "Neither the {} '{}' or its {} '{}' has an attribute called '{}'",
        //        name_of_test_invocation(&self.tester),
        //        &self.name()?,
        //        name_of_test(&self.tester),
        //        &self.test_name()?.unwrap(),
        //        name
        //    ),
        //    None => format!(
        //        "The {} '{}' has no attribute called '{}'",
        //        name_of_test_invocation(&self.tester),
        //        &self.name()?,
        //        name
        //    ),
        //};
        //Err(AttributeError::py_err(msg))
    }
}
