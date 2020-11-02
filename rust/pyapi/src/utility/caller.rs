//! Provides helper functions to get the caller information from Python
#![allow(dead_code)]

use pyo3::prelude::*;
//use origen::Result;

#[derive(Debug)]
pub struct FrameInfo {
    filename: String,
    lineno: usize,
    function: String,
    code_context: Option<Vec<String>>,
    index: Option<usize>,
}

enum Filter<'a> {
    None,
    StartsWith(&'a str),
    Contains(&'a str),
}

/// Returns the final caller from Python, equivalent to caller::stack[0] but quicker to return if
/// that's all you need from the stack.
/// Returns None if an error occurred extracting the stack info.
pub fn caller() -> Option<FrameInfo> {
    let mut stack = match _get_stack(Some(1), Filter::None) {
        Err(_e) => return None,
        Ok(x) => x,
    };
    stack.pop()
}

/// Returns the last caller from Python application code, screening out any later calls through Origen
/// core or plugin code
pub fn app_caller() -> Option<FrameInfo> {
    if let Some(app) = origen::app() {
        if let Some(root) = app.root.to_str() {
            let mut stack = match _get_stack(Some(1), Filter::StartsWith(root)) {
                Err(_e) => return None,
                Ok(x) => x,
            };
            return stack.pop();
        }
    }
    None
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

fn _get_stack(max_depth: Option<usize>, filter: Filter) -> Result<Vec<FrameInfo>, PyErr> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inspect = PyModule::import(py, "inspect")?;
    let stack: Vec<Vec<&PyAny>> = inspect.call0("stack")?.extract()?;
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
}
