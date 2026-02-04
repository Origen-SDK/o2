use super::flow_options;
use super::{Condition, Group, Resources, Test, TestInvocation};
use super::tester_apis::IGXL;
use super::src_caller_meta;
use origen_metal::prog_gen::{flow_api, BinType, FlowCondition, FlowID, GroupType, ResourcesType, SupportedTester};
use origen_metal::Result;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple};
use std::collections::HashSet;
use std::str::FromStr;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "interface")?;
    subm.add_class::<PyInterface>()?;
    m.add_submodule(subm)?;
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

    #[setter]
    fn set_description(&self, desc: String) -> PyResult<()> {
        flow_api::flow_description(desc.trim().to_string(), None)?;
        Ok(())
    }

    #[setter]
    fn set_name_override(&self, name: String) -> PyResult<()> {
        flow_api::flow_name_override(name.trim().to_string(), None)?;
        Ok(())
    }

    #[setter(resources_filename)]
    fn _set_resources_filename(&self, path: String) -> PyResult<()> {
        Err(PyTypeError::new_err(format!(
            "The resources filename should be set like this: flow.set_resources_filename(\"{}\")",
            path
        )))
    }

    fn set_resources_filename(&mut self, path: String, kind: Option<String>) -> PyResult<()> {
        let kind = match kind {
            None => ResourcesType::All,
            Some(n) => match ResourcesType::from_str(&n) {
                Ok(n) => n,
                Err(e) => {
                    return Err(PyTypeError::new_err(format!(
                        "Illegal resources filename kind: {}",
                        e
                    )))
                }
            },
        };
        flow_api::set_resources_filename(path, kind, None)?;
        Ok(())
    }

    /// Add a test to the flow
    #[pyo3(signature=(test_obj, allow_missing=false, **kwargs))]
    fn add_test(&self, test_obj: &PyAny, allow_missing: bool, kwargs: Option<&PyDict>) -> PyResult<()> {
        let id = flow_options::get_flow_id(kwargs)?;
        let bin = flow_options::get_bin(kwargs)?;
        let softbin = flow_options::get_softbin(kwargs)?;
        let number = flow_options::get_number(kwargs)?;

        Ok(flow_options::wrap_in_conditions(kwargs, false, || {
            if let Ok(t) = test_obj.extract::<TestInvocation>() {
                flow_api::execute_test(t.id, id.clone(), src_caller_meta())?;
            } else if let Ok(t) = test_obj.extract::<Test>() {
                match t.tester {
                    SupportedTester::IGXL | SupportedTester::J750 | SupportedTester::ULTRAFLEX => {
                        let mut flow_line =
                            IGXL::new(Some(t.tester.to_string()))?.new_flow_line(allow_missing, kwargs)?;
                        flow_line.set_test_obj(t)?;
                        flow_api::execute_test(flow_line.id, id.clone(), src_caller_meta())?;
                    }
                    SupportedTester::V93K
                    | SupportedTester::V93KSMT7
                    | SupportedTester::V93KSMT8 => {
                        bail!("expected a Test Suite but was given a Test Method");
                    }
                    _ => {
                        bail!(
                            "add_test doesn't yet know how to handle a test object for '{}'",
                            t.tester
                        );
                    }
                }
            } else if let Ok(t) = test_obj.extract::<String>() {
                flow_api::execute_test_str(t, id.clone(), bin, softbin, number, src_caller_meta())?;
            } else {
                bail!(
                    "add_test must be given a valid test object, or a String, this is neither: {:?}",
                    test_obj
                );
            }

            if let Some(bin) = bin {
                let ref_id = flow_api::start_on_failed(id.clone(), None)?;
                self.bin(bin, softbin, None, false, None, None, None)?;
                flow_api::end_block(ref_id)?;
            }
            flow_options::on_fail(&id, kwargs)?;
            flow_options::on_pass(&id, kwargs)?;
            Ok(())
        })?.0)
    }

    /// Add a cz test to the flow
    #[pyo3(signature=(test_obj, cz_setup, **kwargs))]
    fn add_cz_test(
        &self,
        test_obj: &PyAny,
        cz_setup: String,
        kwargs: Option<&PyDict>,
    ) -> PyResult<()> {
        let id = flow_options::get_flow_id(kwargs)?;

        if let Ok(t) = test_obj.extract::<TestInvocation>() {
            flow_api::execute_cz_test(t.id, cz_setup, id, src_caller_meta())?;
        } else if let Ok(t) = test_obj.extract::<Test>() {
            flow_api::execute_cz_test(t.id, cz_setup, id, src_caller_meta())?;
        } else {
            return Err(PyTypeError::new_err(format!(
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

    #[pyo3(signature=(name, **kwargs))]
    fn group(&mut self, name: String, kwargs: Option<&PyDict>) -> PyResult<Group> {
        let id = flow_options::get_flow_id(kwargs)?;
        let (mut g, ref_ids) = flow_options::wrap_in_conditions(kwargs, true, || {
            let g = Group::new(name, None, GroupType::Flow, Some(id));
            Ok(g)
        })?;
        g.open_conditions = ref_ids;
        Ok(g)
    }

    fn resources(&mut self) -> PyResult<Resources> {
        let r = Resources::new();
        Ok(r)
    }

    #[pyo3(signature=(*jobs, **_kwargs))]
    fn if_job(&mut self, jobs: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(jobs) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfJob(v))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*jobs, **_kwargs))]
    fn unless_job(&mut self, jobs: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(jobs) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessJob(v))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*flags, **_kwargs))]
    fn if_enable(&mut self, flags: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(flags) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfEnable(v))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*flags, **_kwargs))]
    fn unless_enable(&mut self, flags: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(flags) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessEnable(v))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*flags, **_kwargs))]
    fn if_enabled(&mut self, flags: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(flags) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfEnable(v))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*flags, **_kwargs))]
    fn unless_enabled(&mut self, flags: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(flags) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessEnable(v))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*ids, **_kwargs))]
    fn if_passed(&mut self, ids: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfPassed(
                v.iter().map(|id| FlowID::from_str(id)).collect(),
            ))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*ids, **_kwargs))]
    fn unless_passed(&mut self, ids: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfFailed(
                v.iter().map(|id| FlowID::from_str(id)).collect(),
            ))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*ids, **_kwargs))]
    fn if_failed(&mut self, ids: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfFailed(
                v.iter().map(|id| FlowID::from_str(id)).collect(),
            ))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*ids, **_kwargs))]
    fn unless_failed(&mut self, ids: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfPassed(
                v.iter().map(|id| FlowID::from_str(id)).collect(),
            ))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*ids, **_kwargs))]
    fn if_ran(&mut self, ids: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfRan(
                v.iter().map(|id| FlowID::from_str(id)).collect(),
            ))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*ids, **_kwargs))]
    fn unless_ran(&mut self, ids: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessRan(
                v.iter().map(|id| FlowID::from_str(id)).collect(),
            ))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*ids, **_kwargs))]
    fn if_flag(&mut self, ids: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfFlag(v))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*ids, **_kwargs))]
    fn unless_flag(&mut self, ids: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match extract_to_string_vec(ids) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessFlag(v))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(*flags))]
    fn volatile(&mut self, flags: &PyTuple) -> PyResult<()> {
        match extract_to_string_vec(flags) {
            Ok(v) => Ok(flow_api::set_volatile_flags(v, None)?),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature=(tester))]
    /// Returns a list of valid test invocation options for the given tester type.
    fn test_invocation_options(&self, tester: String) -> PyResult<HashSet<String>> {
        let tester = SupportedTester::from_str(&tester).map_err(|e| {
            PyTypeError::new_err(format!("Invalid tester type: {}", e))
        })?;
        let opts = origen_metal::prog_gen::test_invocation_options(tester)?;
        Ok(opts.into_iter().collect())
    }

    /// Bin out
    #[pyo3(signature=(hard_bin, soft_bin=None, softbin=None, good=false, description=None, priority=None, **kwargs))]
    fn bin(
        &self,
        hard_bin: usize,
        soft_bin: Option<usize>,
        softbin: Option<usize>,
        good: bool,
        description: Option<String>,
        priority: Option<usize>,
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
        if description.is_some() || priority.is_some() {
            if let Some(sbin) = sbin {
                flow_api::define_bin(
                    sbin,
                    true,
                    kind.clone(),
                    description.clone(),
                    priority,
                    None,
                )?;
            }
            flow_api::define_bin(hard_bin, false, kind.clone(), description, priority, None)?;
        }
        Ok(flow_options::wrap_in_conditions(kwargs, false, || {
            flow_api::bin(hard_bin, sbin, kind, None)?;
            Ok(())
        })?
        .0)
    }

    #[pyo3(signature=(hard_bin, soft_bin=None, softbin=None, description=None, priority=None, **kwargs))]
    fn good_die(
        &self,
        hard_bin: usize,
        soft_bin: Option<usize>,
        softbin: Option<usize>,
        description: Option<String>,
        priority: Option<usize>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<()> {
        self.bin(
            hard_bin,
            soft_bin,
            softbin,
            true,
            description,
            priority,
            kwargs,
        )
    }

    #[pyo3(signature=(hard_bin, soft_bin=None, softbin=None, description=None, priority=None, **kwargs))]
    fn bad_die(
        &self,
        hard_bin: usize,
        soft_bin: Option<usize>,
        softbin: Option<usize>,
        description: Option<String>,
        priority: Option<usize>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<()> {
        self.bin(
            hard_bin,
            soft_bin,
            softbin,
            false,
            description,
            priority,
            kwargs,
        )
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
            bail!("Expected a string or a list of strings, got '{}'", arg);
        }
    }
    Ok(clean)
}
