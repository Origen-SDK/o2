use super::flow_options;
use super::src_caller_meta;
use super::tester_apis::IGXL;
use super::{Condition, Group, Loop, Resources, Test, TestInvocation};
use origen_metal::prog_gen::{
    flow_api, BinType, FlowCondition, FlowID, GroupType, LoopCount, ResourcesType, SupportedTester,
};
use origen_metal::Result;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyDict, PyLong, PyString, PyTuple};
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
    fn add_test(
        &self,
        test_obj: &PyAny,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<()> {
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
                        if let Some(number) = number {
                            flow_line.set_attr(
                                "number",
                                Some(origen_metal::prog_gen::ParamValue::UInt(number as u64)),
                                false,
                            )?;
                        }
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
        let mut ref_ids = ref_ids.unwrap_or_default();
        let mut bypass = false;
        if let Some(kwargs) = kwargs {
            if let Some(v) = kwargs.get_item("bypass")? {
                bypass = v.is_truthy()?;
            }
        }
        if bypass {
            // Opened inside any conditions, so it is closed first (ref_ids is innermost first)
            ref_ids.insert(0, flow_api::start_bypass_sub_flows(src_caller_meta())?);
        }
        g.open_conditions = if ref_ids.is_empty() {
            None
        } else {
            Some(ref_ids)
        };
        Ok(g)
    }

    /// Repeats the flow content generated within the returned context manager count times.
    /// The count is a number from 1, or the name of a flow variable which holds it when the loop
    /// starts. Like a hand-written V93K SMT8 for loop, a loop to a variable is not guarded, the
    /// variable must be at least 1 when the loop starts.
    /// The iteration number, counting up from start through count values (or down to start when
    /// descending), is available within the loop as the given flow variable.
    /// Flow options, e.g. if_enable, are applied to the whole loop.
    #[pyo3(signature=(count, variable="index".to_string(), descending=false, start=1, **kwargs))]
    fn r#loop(
        &mut self,
        count: &PyAny,
        variable: String,
        descending: bool,
        start: i64,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Loop> {
        let count = loop_count(count)?;
        let (mut l, ref_ids) = flow_options::wrap_in_conditions(kwargs, true, || {
            Ok(Loop::new(count, variable, descending, start))
        })?;
        l.open_conditions = ref_ids;
        Ok(l)
    }

    /// Set the given flow flag, or clear it when state is false
    #[pyo3(signature=(name, state=true))]
    fn set_flag(&mut self, name: String, state: bool) -> PyResult<()> {
        flow_api::set_flag(name, state, src_caller_meta())?;
        Ok(())
    }

    /// Assign a value, a number or a tester expression, to the given flow variable
    fn set_variable(&mut self, name: String, value: String) -> PyResult<()> {
        flow_api::set_variable(name, value, src_caller_meta())?;
        Ok(())
    }

    /// Declare the given flow variable, keeping the case of its name, as an output of the
    /// current flow without assigning it, e.g. a variable which a test method sets natively.
    /// Must be called at flow level, not within a condition or loop.
    fn declare_output_variable(&mut self, name: String) -> PyResult<()> {
        flow_api::declare_output_variable(name, src_caller_meta())?;
        Ok(())
    }

    /// Declare the given flow variable, keeping the case of its name, as an input of the
    /// current flow without reading it, e.g. a value which the flow only passes down to its
    /// sub-flows. A sub-flow receives it only if it reads or declares it too.
    /// Must be called at flow level, not within a condition or loop.
    fn declare_input_variable(&mut self, name: String) -> PyResult<()> {
        flow_api::declare_input_variable(name, src_caller_meta())?;
        Ok(())
    }

    /// Declare that the given flag, when set in the current flow or in any of its sub-flows,
    /// is local to the flows which read it: it is passed out of the flow which sets it only as
    /// far as the highest enclosing flow in which another flow reads it, rather than to every
    /// enclosing flow. Must be called at flow level, not within a condition or loop.
    fn declare_local_flag(&mut self, name: String) -> PyResult<()> {
        flow_api::declare_local_flag(name, src_caller_meta())?;
        Ok(())
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

    /// Generates the content of the returned context manager only when the given tester
    /// expression is true
    #[pyo3(signature=(expression, **_kwargs))]
    fn if_expr(&mut self, expression: &PyAny, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match flow_options::to_expression("if_expr", expression) {
            Ok(v) => Ok(Condition::new(FlowCondition::IfExpr(v))),
            Err(e) => Err(PyTypeError::new_err(e.to_string())),
        }
    }

    /// Generates the content of the returned context manager only when the given tester
    /// expression is false
    #[pyo3(signature=(expression, **_kwargs))]
    fn unless_expr(&mut self, expression: &PyAny, _kwargs: Option<&PyDict>) -> PyResult<Condition> {
        match flow_options::to_expression("unless_expr", expression) {
            Ok(v) => Ok(Condition::new(FlowCondition::UnlessExpr(v))),
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
        let tester = SupportedTester::from_str(&tester)
            .map_err(|e| PyTypeError::new_err(format!("Invalid tester type: {}", e)))?;
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

/// Returns the count of a loop given as a number, or as the name of a flow variable
fn loop_count(count: &PyAny) -> PyResult<LoopCount> {
    if let Ok(name) = count.downcast::<PyString>() {
        let name = name.to_str()?;
        // Checked here as well as when the loop starts, so that an invalid count doesn't
        // leave the conditions of the loop's flow options open
        if !flow_api::is_identifier(name) {
            return Err(PyValueError::new_err(format!(
                "loop count '{}' must be the name of a flow variable (letters, digits and \
                 underscores, not starting with a digit)",
                name
            )));
        }
        return Ok(LoopCount::Variable(name.to_string()));
    }
    // A bool is also an int in Python
    if count.downcast::<PyLong>().is_ok() && count.downcast::<PyBool>().is_err() {
        return match count.extract::<u32>() {
            Ok(n) if n > 0 => Ok(LoopCount::Fixed(n)),
            _ => Err(PyValueError::new_err(format!(
                "loop count {} must be an integer from 1 to {}, or the name of a flow variable",
                count,
                u32::MAX
            ))),
        };
    }
    Err(PyTypeError::new_err(format!(
        "loop count must be an int or the name of a flow variable (str), not '{}'",
        count.get_type().name()?
    )))
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
