//! Implements Python bindings for program generation data structures and functions

mod condition;
pub mod flow_options;
pub mod group;
pub mod interface;
mod pattern_group;
mod resources;
mod test;
mod test_invocation;

pub use condition::Condition;
pub use group::Group;
use origen::core::reference_files;
use origen::core::tester::TesterSource;
use origen::prog_gen::{
    flow_api, FlowCondition, Model, ParamType, ParamValue, PatternReferenceType,
};
use origen::utility::differ::{ASCIIDiffer, Differ};
use origen::utility::file_utils::to_relative_path;
use origen::{Error, Result, FLOW};
pub use pattern_group::PatternGroup;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::wrap_pyfunction;
use resources::Resources;
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::thread;
pub use test::Test;
pub use test_invocation::TestInvocation;

#[pymodule]
/// Implements the module _origen.prog_gen in Python
pub fn prog_gen(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(start_new_flow))?;
    m.add_wrapped(wrap_pyfunction!(end_flow))?;
    m.add_wrapped(wrap_pyfunction!(render))?;
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

// Called automatically by Origen once all test program source files have been executed
#[pyfunction]
fn render(py: Python) -> PyResult<Vec<String>> {
    let continue_on_fail = true;
    py.allow_threads(|| {
        let targets = {
            let tester = origen::tester();
            tester.targets().clone()
        };
        let threads: Vec<_> = targets.iter().enumerate().map(|(i, t)| {
            let t = t.to_owned();
            thread::spawn(move || {
                match t {
                    TesterSource::External(g) => {
                        log_error!("Python based tester targets are not supported for program generation yet, no action taken for target: {}", g);
                        Ok((vec![], Model::new()))
                    }
                    _ => {
                        let mut tester = origen::tester();
                        let files = tester.render_program_for_target_at(i, true);
                        match files {
                            Err(e) => {
                                let msg = e.to_string();
                                if continue_on_fail {
                                    origen::STATUS.inc_unhandled_error_count();
                                    log_error!("{}", &msg);
                                    Ok((vec![], Model::new()))
                                } else {
                                    Err(e)
                                }
                            }
                            Ok(paths_and_model) => Ok(paths_and_model)
                        }
                    }
                }
            })
        }).collect();
        let mut generated_files: Vec<String> = vec![];
        let mut referenced_patterns: HashSet<String> = HashSet::new();
        for thread in threads {
            match thread.join() {
                Err(_e) => log_error!("Something has gone wrong when doing the final program render"),
                Ok(v) => match v {
                    Err(e) => log_error!("{}", e),
                    Ok(paths_and_model) => {
                        for path in &paths_and_model.0 {
                            generated_files.push(format!("{}", path.display()));
                        }
                        // Extract the referenced patterns and add to global set
                        for pat in &paths_and_model.1.patterns {
                            if pat.reference_type == PatternReferenceType::All || pat.reference_type == PatternReferenceType::Origen {
                                let _ = referenced_patterns.insert(pat.path.to_owned());
                            }
                        }
                    }
                }
            }
        }

        let dir = origen::app().unwrap().root.join("list");
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        let list = dir.join("referenced.list");
        let mut f = std::fs::File::create(&list)?;
        let mut pats: Vec<_> = referenced_patterns.into_iter().collect();
        pats.sort();
        for pat in pats {
            writeln!(&mut f, "{}", pat)?;
        }

        // TODO: This diffing/reporting should be abstracted into the core:
        if let Ok(p) = to_relative_path(&list, None) {
            display!("Created: {}", p.display());
        } else {
            display!("Created: {}", list.display());
        }
        if let Some(ref_dir) = origen::STATUS.reference_dir() {
            let ref_list = ref_dir.join("referenced.list");
            display!(" - ");
            if ref_list.exists() {
                let mut differ = ASCIIDiffer::new(&ref_list, &list);
                differ.ignore_comments("#")?;
                if differ.has_diffs()? {
                    if let Err(e) = reference_files::create_changed_ref(Path::new("referenced.list"), &list, &ref_list) {
                        log_error!("{}", e);
                    }
                    origen::tester().stats.changed_program_files += 1;
                    display_redln!("Diffs found");
                    let old = to_relative_path(&ref_list, None).unwrap_or(ref_list);
                    let new = to_relative_path(&list, None).unwrap_or(list.to_owned());
                    let diff_tool = std::env::var("ORIGEN_DIFF_TOOL").unwrap_or("tkdiff".to_string());
                    displayln!("  {} {} {} &", &diff_tool, old.display(), new.display());
                    display!("  origen save_ref referenced.list");
                } else {
                    display_green!("No diffs");
                }
            } else {
                origen::tester().stats.new_program_files += 1;
                if let Err(e) = reference_files::create_new_ref(Path::new("referenced.list"), &list, &ref_list) {
                    log_error!("{}", e);
                }
                display_cyanln!("New file");
                display!("  origen save_ref referenced.list");
            }
        }
        displayln!("");

        // Could hand over the model here in future to allow the app to generate additional output from it

        Ok(generated_files)
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
                error!("Illegal value, expected a Boolean, got: '{}'", value)
            }
        }
        ParamType::Int => {
            if let Ok(v) = value.extract::<i64>() {
                Ok(ParamValue::Int(v))
            } else {
                error!("Illegal value, expected an Integer, got: '{}'", value)
            }
        }
        ParamType::UInt => {
            if let Ok(v) = value.extract::<u64>() {
                Ok(ParamValue::UInt(v))
            } else {
                error!(
                    "Illegal value, expected an Unsigned Integer, got: '{}'",
                    value
                )
            }
        }
        ParamType::Float => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Float(v))
            } else {
                error!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Current => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Current(v))
            } else {
                error!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Voltage => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Voltage(v))
            } else {
                error!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Time => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Time(v))
            } else {
                error!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::Frequency => {
            if let Ok(v) = value.extract::<f64>() {
                Ok(ParamValue::Frequency(v))
            } else {
                error!("Illegal value, expected a Float, got: '{}'", value)
            }
        }
        ParamType::String => {
            if let Ok(v) = value.extract::<String>() {
                Ok(ParamValue::String(v))
            } else {
                error!("Illegal value, expected a String, got: '{}'", value)
            }
        }
        ParamType::Any => Ok(ParamValue::Any(format!("{}", value.str()?))),
    }
}
