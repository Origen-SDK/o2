pub mod tester_apis;
mod test_invocation;
mod flow_options;
mod test;
mod group;
mod pattern_group;
mod condition;
mod resources;
pub mod interface;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use test_invocation::TestInvocation;
use test::Test;
use group::Group;
use pattern_group::PatternGroup;
use condition::Condition;
use resources::Resources;

use origen_metal::ast::{Meta, Node};
use pyo3::types::PyAny;
use origen_metal::{Result, Error, FLOW};
use origen_metal::prog_gen::{PGM, ParamType, ParamValue, UniquenessOption};
use pyo3::prelude::*;
use origen_metal::prog_gen::{flow_api, FlowCondition, SupportedTester};
use std::result::Result as StdResult;
use origen_metal::prog_gen::test_ids::define as define_test_ids;

#[derive(Debug)]
pub struct FrameInfo {
    filename: String,
    lineno: usize,
    #[allow(dead_code)]
    function: String,
    #[allow(dead_code)]
    code_context: Option<Vec<String>>,
    #[allow(dead_code)]
    index: Option<usize>,
}

impl FrameInfo {
    /// Turns the frame into an AST meta object, consuming self in the process
    pub fn to_meta(self) -> Meta {
        Meta {
            filename: Some(self.filename),
            lineno: Some(self.lineno),
        }
    }
}

enum Filter<'a> {
    None,
    #[allow(dead_code)]
    StartsWith(&'a str),
    Contains(&'a str),
}

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "prog_gen")?;
    define_test_ids(py, subm)?;
    subm.add_wrapped(wrap_pyfunction!(start_new_flow))?;
    subm.add_wrapped(wrap_pyfunction!(end_flow))?;
    subm.add_wrapped(wrap_pyfunction!(reset))?;
    subm.add_wrapped(wrap_pyfunction!(render_program_for))?;
    subm.add_wrapped(wrap_pyfunction!(start_eq_block))?;
    subm.add_wrapped(wrap_pyfunction!(end_eq_block))?;
    subm.add_wrapped(wrap_pyfunction!(start_neq_block))?;
    subm.add_wrapped(wrap_pyfunction!(end_neq_block))?;
    subm.add_wrapped(wrap_pyfunction!(set_debugging))?;
    subm.add_wrapped(wrap_pyfunction!(start_src_file))?;
    subm.add_wrapped(wrap_pyfunction!(end_src_file))?;
    subm.add_wrapped(wrap_pyfunction!(processed_ast))?;
    subm.add_wrapped(wrap_pyfunction!(processed_ast_str))?;
    subm.add_wrapped(wrap_pyfunction!(ast))?;
    subm.add_wrapped(wrap_pyfunction!(ast_str))?;
    subm.add_wrapped(wrap_pyfunction!(set_test_template_load_path))?;
    subm.add_wrapped(wrap_pyfunction!(set_uniqueness_option))?;
    subm.add_wrapped(wrap_pyfunction!(set_namespace))?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyfunction]
fn set_test_template_load_path(load_path: Vec<PathBuf>) -> PyResult<()> {
    origen_metal::PROG_GEN_CONFIG.set_test_template_load_path(load_path);
    Ok(())
}

#[pyfunction]
fn set_uniqueness_option(option: String) -> PyResult<()> {
    match UniquenessOption::from_str(&option) {
        Ok(uo) => {
            origen_metal::PROG_GEN_CONFIG.set_uniqueness_option(uo);
        }
        Err(e) => {
            return Err(PyErr::from(Error::new(&format!(
                "Failed to set the uniqueness option to '{}': {}",
                option, e
            ))))
        }
    }
    Ok(())
}

#[pyfunction]
fn set_debugging(value: bool) -> PyResult<()> {
    origen_metal::PROG_GEN_CONFIG.set_debug_enabled(value);
    Ok(())
}

#[pyfunction]
/// Clears all flows and starts program generation from scratch
fn reset() -> PyResult<()> {
    origen_metal::FLOW.reset();
    Ok(())
}

#[pyfunction]
fn render_program_for(tester: &str, output_dir: &str) -> PyResult<Vec<PathBuf>> {
    let t = match origen_metal::prog_gen::SupportedTester::from_str(tester) {
        Ok(t) => t,
        Err(e) => {
            return Err(PyErr::from(Error::new(&format!(
                "Failed to identify a supported tester type from '{}': {}",
                tester, e
            ))))
        }
    };
    let output_dir = Path::new(output_dir).to_path_buf();
    let r = origen_metal::prog_gen::render_program(t, &output_dir)?;
    Ok(r.0)
}

#[pyfunction]
fn start_eq_block(testers: Vec<&str>) -> PyResult<(usize, Vec<String>)> {
    let mut ts: Vec<SupportedTester> = vec![];
    let mut clean_testers: Vec<String> = vec![];
    for t in testers {
        let st = SupportedTester::new(t)?;
        clean_testers.push(st.to_string());
        ts.push(st);
    }
    let n = node!(PGM::TesterEq, ts);
    let ref_id = FLOW.push_and_open(n)?;
    Ok((ref_id, clean_testers))
}

#[pyfunction]
fn end_eq_block(ref_id: usize) -> PyResult<()> {
    FLOW.close(ref_id)?;
    Ok(())
}

#[pyfunction]
fn start_neq_block(testers: Vec<&str>) -> PyResult<usize> {
    let mut ts: Vec<SupportedTester> = vec![];
    for t in testers {
        let st = SupportedTester::new(t)?;
        ts.push(st);
    }
    let n = node!(PGM::TesterNeq, ts);
    let ref_id = FLOW.push_and_open(n)?;
    Ok(ref_id)
}

#[pyfunction]
fn end_neq_block(ref_id: usize) -> PyResult<()> {
    FLOW.close(ref_id)?;
    Ok(())
}

#[pyfunction]
fn start_new_flow(
    name: &str,
    sub_flow: Option<bool>,
    bypass_sub_flows: Option<bool>,
    add_flow_enable: Option<&str>,
) -> PyResult<Vec<usize>> {
    let sub_flow = match sub_flow {
        None => false,
        Some(x) => x,
    };
    let mut refs = vec![];
    if sub_flow {
        refs.push(flow_api::start_sub_flow(name, None, None)?);
    } else {
        FLOW.start(name)?;
        refs.push(0);
        if let Some(bypass) = bypass_sub_flows {
            if bypass {
                refs.push(flow_api::start_bypass_sub_flows(None)?);
            }
        }
        if let Some(enable) = add_flow_enable {
            let flag = format!("{}_enable", name);
            refs.push(flow_api::start_condition(
                FlowCondition::IfEnable(vec![flag.clone()]),
                None,
            )?);
            if enable.to_lowercase() == "enabled" {
                flow_api::set_default_flag_state(flag, true, None)?;
            } else if enable.to_lowercase() == "disabled" {
                flow_api::set_default_flag_state(flag, false, None)?;
            } else {
                return Err(PyErr::from(Error::new(&format!(
                    "The add_flow_enable argument must be either None (default), \"enabled\" or \"disabled\", got '{}'",
                    enable
                ))));
            }
        }
    }
    refs.reverse();
    Ok(refs)
}

#[pyfunction]
fn set_namespace(namespace: String) -> PyResult<()> {
    flow_api::set_namespace(namespace, None)?;
    Ok(())
}

#[pyfunction]
fn end_flow(ref_ids: Vec<usize>) -> PyResult<()> {
    for ref_id in ref_ids {
        if ref_id == 0 {
            FLOW.end()?;
        } else {
            flow_api::end_block(ref_id)?;
        }
    }
    Ok(())
}

#[pyfunction]
/// Returns the AST for the current flow after it has been fully processed for the given tester type
fn processed_ast(tester: String) -> PyResult<Vec<u8>> {
    Ok(_processed_ast(&tester)?.to_pickle())
}

fn _processed_ast(tester: &str) -> Result<Node<PGM>> {
    let tester = match SupportedTester::from_str(&tester) {
        Ok(t) => t,
        Err(e) => {
            bail!(
                "Failed to identify a supported tester type from '{}': {}",
                tester, e
            )
        }
    };
    match tester {
        SupportedTester::V93KSMT7 => {
            let ast = FLOW.with_selected_flow(|flow| {
                let model = origen_metal::prog_gen::Model::new(tester);
                let (ast, _m) = origen_metal::prog_gen::process_flow(flow, model, tester, false)?;
                Ok(ast)
            })?;
            Ok(ast)
        }
        _ => bail!(
            "The tester type '{}' is not supported yet for processed AST generation",
            tester
        )
    }
}   

#[pyfunction]   
fn processed_ast_str(tester: String) -> PyResult<String> {
    Ok(origen_metal::ast::to_string(&_processed_ast(&tester)?))
}

/// Returns the raw AST for the current flow in Python
#[pyfunction]
fn ast() -> PyResult<Vec<u8>> {
    Ok(FLOW.to_pickle())
}

#[pyfunction]
fn ast_str() -> PyResult<String> {
    Ok(origen_metal::ast::to_string(&FLOW.to_node()))
}

#[pyfunction]
fn start_src_file(file: PathBuf) -> PyResult<()> {
    origen_metal::PROG_GEN_CONFIG.start_src_file(file)?;
    Ok(())
}

#[pyfunction]
fn end_src_file() -> PyResult<()> {
    origen_metal::PROG_GEN_CONFIG.end_src_file();
    Ok(())
}

/// Returns the last caller that was a test program flow
pub fn src_caller() -> Option<FrameInfo> {
    if let Some(f) = origen_metal::PROG_GEN_CONFIG.current_src_file() {
        caller_containing(&format!("{}", f.display()))
    } else {
        None
    }
}

/// Same as src_caller() but returns an AST metadata
pub fn src_caller_meta() -> Option<Meta> {
    if origen_metal::PROG_GEN_CONFIG.debug_enabled() {
        let c = src_caller();
        let m = match c {
            Some(m) => Some(m.to_meta()),
            None => None,
        };
        m
    } else {
        None
    }
}

/// Returns the last caller where the filename contains the given text
pub fn caller_containing(text: &str) -> Option<FrameInfo> {
    let mut stack = match _get_stack(Some(1), Filter::Contains(text)) {
        Err(_e) => return None,
        Ok(x) => x,
    };
    stack.pop()
}

/// Returns the full Python stack, including calls from app code, plugin code and Origen core
/// Returns None if an error occurred extracting the stack info.
pub fn stack() -> Option<Vec<FrameInfo>> {
    match _get_stack(None, Filter::None) {
        Err(_e) => {
            //log_debug!("{:?}", e);
            //let gil = Python::acquire_gil();
            //let py = gil.python();
            //e.print(py);
            None
        }
        Ok(x) => Some(x),
    }
}

fn _get_stack(max_depth: Option<usize>, filter: Filter) -> StdResult<Vec<FrameInfo>, PyErr> {
    Python::with_gil(|py| {
        let inspect = PyModule::import(py, "inspect")?;
        let stack: Vec<Vec<&PyAny>> = inspect.getattr("stack")?.call0()?.extract()?;
        let mut frames: Vec<FrameInfo> = vec![];
        for f in stack {
            let filename: String = f[1].extract()?;
            let include = match filter {
                Filter::None => true,
                Filter::StartsWith(s) => filename.starts_with(s),
                Filter::Contains(s) => filename.contains(s),
            };
            if include {
                frames.push(FrameInfo {
                    filename: filename,
                    lineno: f[2].extract()?,
                    function: f[3].extract()?,
                    code_context: f[4].extract()?,
                    index: f[5].extract()?,
                });

                if let Some(x) = max_depth {
                    if x == frames.len() {
                        break;
                    }
                }
            }
        }
        Ok(frames)
    })
}

pub fn to_param_value(value: &PyAny) -> Result<Option<ParamValue>> {
    Ok(if let Ok(v) = value.extract::<bool>() {
        Some(ParamValue::Bool(v))
    } else if let Ok(v) = value.extract::<u64>() {
        Some(ParamValue::UInt(v))
    } else if let Ok(v) = value.extract::<i64>() {
        Some(ParamValue::Int(v))
    } else if let Ok(v) = value.extract::<f64>() {
        Some(ParamValue::Float(v))
    } else if let Ok(v) = value.extract::<String>() {
        Some(ParamValue::String(v))
    } else if let Ok(None) = value.extract::<Option<String>>() {
        None
    } else {
        Some(ParamValue::Any(format!("{}", value.str()?)))
    })
}

#[allow(dead_code)] // Could be used in future
pub fn to_param_value_with_type(ptype: &ParamType, value: &PyAny) -> Result<ParamValue> {
    match ptype {
        ParamType::Bool => {
            if let Ok(v) = value.extract::<bool>() {
                Ok(ParamValue::Bool(v))
            } else {
                bail!("Illegal value, expected a Boolean, got: '{}'", value)
            }
        }
        ParamType::Int => {
            if let Ok(v) = value.extract::<i64>() {
                Ok(ParamValue::Int(v))
            } else {
                bail!("Illegal value, expected an Integer, got: '{}'", value)
            }
        }
        ParamType::UInt => {
            if let Ok(v) = value.extract::<u64>() {
                Ok(ParamValue::UInt(v))
            } else {
                bail!(
                    "Illegal value, expected an Unsigned Integer, got: '{}'",
                    value
                )
            }
        }
        ParamType::Float => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Float(v))
            } else {
                bail!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Current => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Current(v))
            } else {
                bail!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Voltage => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Voltage(v))
            } else {
                bail!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Time => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Time(v))
            } else {
                bail!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Frequency => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Frequency(v))
            } else {
                bail!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::String => {
            if let Ok(v) = value.extract::<String>() {
                Ok(ParamValue::String(v))
            } else {
                bail!("Illegal value, expected a String, got: '{}'", value)
            }
        }
        ParamType::Any => Ok(ParamValue::Any(format!("{}", value.str()?))),
    }
}
