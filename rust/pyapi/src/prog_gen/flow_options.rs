//! Functions concerned with extracting known values from kwargs

use crate::utility::caller::src_caller_meta;
use origen::prog_gen::{flow_api, FlowCondition, FlowID};
use origen::Result;
use pyo3::types::PyDict;

pub fn is_flow_option(key: &str) -> bool {
    match key {
        "id" => true,
        "bin" => true,
        "softbin" => true,
        "soft_bin" => true,
        _ => false,
    }
}

pub fn wrap_in_conditions<T, F>(kwargs: Option<&PyDict>, func: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    if let Some(kwargs) = kwargs {
        let mut ref_ids = vec![];
        if let Some(ids) = extract_condition("if_enable", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::IfEnable(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("if_enabled", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::IfEnable(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("unless_enable", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::UnlessEnable(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("unless_enabled", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::UnlessEnable(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("if_job", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::IfJob(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("unless_job", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::UnlessJob(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("if_ran", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::IfRan(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("unless_ran", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::UnlessRan(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("if_passed", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::IfPassed(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("unless_passed", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::UnlessPassed(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("if_failed", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::IfFailed(ids),
                src_caller_meta(),
            )?);
        }
        if let Some(ids) = extract_condition("unless_failed", kwargs)? {
            ref_ids.push(flow_api::start_condition(
                FlowCondition::UnlessFailed(ids),
                src_caller_meta(),
            )?);
        }
        let r = func();
        ref_ids.reverse();
        for id in ref_ids {
            flow_api::end_block(id)?;
        }
        r
    } else {
        func()
    }
}

fn extract_condition(name: &str, kwargs: &PyDict) -> Result<Option<Vec<String>>> {
    if let Some(v) = kwargs.get_item(name) {
        if let Ok(v) = v.extract::<String>() {
            Ok(Some(vec![v]))
        } else if let Ok(v) = v.extract::<Vec<String>>() {
            Ok(Some(v))
        } else {
            error!(
                "Illegal '{}' value, expected a String or a List of Strings, got: '{}'",
                name, v
            )
        }
    } else {
        Ok(None)
    }
}

/// Returns a FlowID object from an "id" field present in the args, or else
/// a generated ID.
pub fn get_flow_id(kwargs: Option<&PyDict>) -> Result<FlowID> {
    if let Some(kwargs) = kwargs {
        if let Some(id) = kwargs.get_item("id") {
            if let Ok(v) = id.extract::<String>() {
                return Ok(FlowID::from_str(&v));
            } else if let Ok(v) = id.extract::<usize>() {
                return Ok(FlowID::from_int(v));
            } else {
                return error!(
                    "Illegal 'id' value, expected a String or an Integer, got: '{}'",
                    id
                );
            }
        }
    }
    Ok(FlowID::new())
}

pub fn get_bin(kwargs: Option<&PyDict>) -> Result<Option<usize>> {
    if let Some(kwargs) = kwargs {
        if let Some(bin) = kwargs.get_item("bin") {
            if let Ok(v) = bin.extract::<usize>() {
                return Ok(Some(v));
            } else {
                return error!("Illegal 'bin' value, expected an Integer, got: '{}'", bin);
            }
        }
    }
    Ok(None)
}

pub fn get_softbin(kwargs: Option<&PyDict>) -> Result<Option<usize>> {
    if let Some(kwargs) = kwargs {
        if let Some(bin) = kwargs.get_item("softbin") {
            if let Ok(v) = bin.extract::<usize>() {
                return Ok(Some(v));
            } else {
                return error!(
                    "Illegal 'softbin' value, expected an Integer, got: '{}'",
                    bin
                );
            }
        }
        if let Some(bin) = kwargs.get_item("soft_bin") {
            if let Ok(v) = bin.extract::<usize>() {
                return Ok(Some(v));
            } else {
                return error!(
                    "Illegal 'soft_bin' value, expected an Integer, got: '{}'",
                    bin
                );
            }
        }
    }
    Ok(None)
}
