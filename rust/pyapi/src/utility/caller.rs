//! Provides helper functions to get the caller information from Python
#![allow(dead_code)]

use origen::generator::ast::Meta;
use origen::STATUS;
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

/// Returns the last caller that was a test program flow (and possibly a pattern in future, hence the
/// generic function name)
pub fn src_caller() -> Option<FrameInfo> {
    let file = origen::with_current_job(|job| {
        if let Some(f) = job.files.last() {
            Ok(Some(format!("{}", f.display())))
        } else {
            Ok(None)
        }
    })
    .unwrap_or(None);
    if let Some(f) = file {
        caller_containing(&f)
    } else {
        None
    }
}

/// Same as src_caller() but returns an AST metadata
pub fn src_caller_meta() -> Option<Meta> {
    if STATUS.is_debug_enabled() {
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

fn _get_stack(max_depth: Option<usize>, filter: Filter) -> Result<Vec<FrameInfo>, PyErr> {
    let gil = Python::acquire_gil();
    let py = gil.python();
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
}
