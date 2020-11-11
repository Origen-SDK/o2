use crate::prog_gen::{Condition, Group, Test, TestInvocation};
use origen::prog_gen::{flow_api, BinType, FlowCondition, GroupType};
use origen::Result;
use pyo3::exceptions::TypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple};
use std::path::Path;

#[pymodule]
pub fn interface(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyInterface>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyInterface {
    //python_testers: HashMap<String, PyObject>,
//instantiated_testers: HashMap<String, PyObject>,
//metadata: Vec<PyObject>,
}

#[pymethods]
impl PyInterface {
    #[new]
    fn new() -> Self {
        PyInterface {}
    }

    fn resolve_file_reference(&self, path: &str) -> PyResult<String> {
        let file = origen::with_current_job(|job| {
            job.resolve_file_reference(Path::new(path), Some(vec!["py"]))
        })?;
        Ok(file.to_str().unwrap().to_string())
    }

    /// Add a test to the flow
    #[args(id = "None", kwargs = "**")]
    #[allow(unused_variables)]
    fn add_test(
        &self,
        test_obj: &PyAny,
        id: Option<String>,
        //if_failed: Option<&PyAny>,
        //test_text: Option<String>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<()> {
        if let Ok(t) = test_obj.extract::<TestInvocation>() {
            flow_api::execute_test(t.id, id, None)?;
        } else if let Ok(t) = test_obj.extract::<Test>() {
            flow_api::execute_test(t.id, id, None)?;
        } else if let Ok(t) = test_obj.extract::<String>() {
            flow_api::execute_test_str(t, id, None)?;
        } else {
            return Err(TypeError::py_err(format!(
                "add_test must be given a valid test object, or a String, this is neither: {:?}",
                test_obj
            )));
        }
        Ok(())
    }

    /// Add a cz test to the flow
    #[args(kwargs = "**")]
    #[allow(unused_variables)]
    fn add_cz_test(
        &self,
        test_obj: &PyAny,
        cz_setup: String,
        kwargs: Option<&PyDict>,
    ) -> PyResult<()> {
        if let Ok(t) = test_obj.extract::<TestInvocation>() {
            flow_api::execute_cz_test(t.id, cz_setup, None)?;
        } else if let Ok(t) = test_obj.extract::<Test>() {
            flow_api::execute_cz_test(t.id, cz_setup, None)?;
        } else {
            return Err(TypeError::py_err(format!(
                "add_cz_test must be given a valid test object, this is something else: {:?}",
                test_obj
            )));
        }
        Ok(())
    }

    /// Render the given string directly to the current flow
    fn render_str(&self, text: String) -> PyResult<()> {
        flow_api::render(text, None)?;
        Ok(())
    }

    fn log(&self, text: String) -> PyResult<()> {
        flow_api::log(text, None)?;
        Ok(())
    }

    #[args(id = "None", kwargs = "**")]
    #[allow(unused_variables)]
    fn group(
        &mut self,
        name: String,
        id: Option<String>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Group> {
        let g = Group::new(name, None, GroupType::Flow, id);
        Ok(g)
    }

    #[args(jobs = "*")]
    fn if_job_block(&mut self, jobs: &PyTuple) -> PyResult<Condition> {
        match extract_to_string_vec(jobs) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfJob(v))),
            Err(e) => Err(TypeError::py_err(e.to_string())),
        }
    }

    #[args(jobs = "*")]
    fn unless_job_block(&mut self, jobs: &PyTuple) -> PyResult<Condition> {
        match extract_to_string_vec(jobs) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessJob(v))),
            Err(e) => Err(TypeError::py_err(e.to_string())),
        }
    }

    #[args(flags = "*")]
    fn if_enable_block(&mut self, flags: &PyTuple) -> PyResult<Condition> {
        match extract_to_string_vec(flags) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfEnable(v))),
            Err(e) => Err(TypeError::py_err(e.to_string())),
        }
    }

    #[args(flags = "*")]
    fn unless_enable_block(&mut self, flags: &PyTuple) -> PyResult<Condition> {
        match extract_to_string_vec(flags) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessEnable(v))),
            Err(e) => Err(TypeError::py_err(e.to_string())),
        }
    }

    #[args(ids = "*")]
    fn if_passed_block(&mut self, ids: &PyTuple) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfPassed(v))),
            Err(e) => Err(TypeError::py_err(e.to_string())),
        }
    }

    #[args(ids = "*")]
    fn unless_passed_block(&mut self, ids: &PyTuple) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessPassed(v))),
            Err(e) => Err(TypeError::py_err(e.to_string())),
        }
    }

    #[args(ids = "*")]
    fn if_failed_block(&mut self, ids: &PyTuple) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfFailed(v))),
            Err(e) => Err(TypeError::py_err(e.to_string())),
        }
    }

    #[args(ids = "*")]
    fn unless_failed_block(&mut self, ids: &PyTuple) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessFailed(v))),
            Err(e) => Err(TypeError::py_err(e.to_string())),
        }
    }

    #[args(ids = "*")]
    fn if_ran_block(&mut self, ids: &PyTuple) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfRan(v))),
            Err(e) => Err(TypeError::py_err(e.to_string())),
        }
    }

    #[args(ids = "*")]
    fn unless_ran_block(&mut self, ids: &PyTuple) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessRan(v))),
            Err(e) => Err(TypeError::py_err(e.to_string())),
        }
    }

    /// Bin out
    #[args(soft_bin = "None", softbin = "None", good = "false", kwargs = "**")]
    #[allow(unused_variables)]
    fn bin(
        &self,
        hard_bin: usize,
        soft_bin: Option<usize>,
        softbin: Option<usize>,
        good: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<()> {
        let sbin = match soft_bin {
            Some(n) => Some(n),
            None => match softbin {
                Some(n) => Some(n),
                None => None,
            },
        };
        let kind = match good {
            true => BinType::Good,
            false => BinType::Bad,
        };
        flow_api::bin(hard_bin, sbin, kind, None)?;
        Ok(())
    }

    #[args(soft_bin = "None", softbin = "None", kwargs = "**")]
    fn good_die(
        &self,
        hard_bin: usize,
        soft_bin: Option<usize>,
        softbin: Option<usize>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<()> {
        self.bin(hard_bin, soft_bin, softbin, true, kwargs)
    }

    #[args(soft_bin = "None", softbin = "None", kwargs = "**")]
    fn bad_die(
        &self,
        hard_bin: usize,
        soft_bin: Option<usize>,
        softbin: Option<usize>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<()> {
        self.bin(hard_bin, soft_bin, softbin, false, kwargs)
    }
}

fn extract_to_string_vec(args: &PyTuple) -> Result<Vec<String>> {
    let mut clean: Vec<String> = vec![];
    for arg in args {
        if let Ok(a) = arg.extract::<String>() {
            clean.push(a);
        } else if let Ok(items) = arg.extract::<Vec<String>>() {
            for item in items {
                clean.push(item);
            }
        } else {
            return error!("Expected a string or a list of strings, got '{}'", arg);
        }
    }
    Ok(clean)
}
