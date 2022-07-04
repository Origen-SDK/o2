#![allow(non_snake_case)]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyModule};
use crate::{runtime_error, cfg_if};

pub fn wrap_function(py: Python, fn_name: &str, args: Option<Vec<&str>>, scope: Option<&str>, locals: Option<&PyDict>, define_scope: Option<&str>) -> PyResult<PyObject> {
    wrap_context_manager(py, fn_name, args, scope, locals, define_scope)
}

pub fn wrap_instance_method(py: Python, fn_name: &str, args: Option<Vec<&str>>, locals: Option<&PyDict>) -> PyResult<PyObject> {
    wrap_context_manager(py, fn_name, args, Some("self"), locals, None)
}

pub fn wrap_class_method(py: Python, fn_name: &str, args: Option<Vec<&str>>, locals: Option<&PyDict>) -> PyResult<PyObject> {
    wrap_context_manager(py, fn_name, args, Some("cls"), locals, None)
}

pub fn wrap_context_manager(py: Python, fn_name: &str, args: Option<Vec<&str>>, scope: Option<&str>, locals: Option<&PyDict>, define_scope: Option<&str>) -> PyResult<PyObject> {
    let locals_ = locals.unwrap_or(PyDict::new(py));

    let (mut wrapping_ins_method, mut wrapping_cls_method) = (false, false);
    let scope_ = match scope {
        Some(s) => {
            if s == "self" {
                wrapping_ins_method = true;
            } else if s == "cls" {
                wrapping_cls_method = true;
            }
            format!("{}.", s)
        },
        None => "".to_string()
    };

    let (prototype_args, enter_args) = match args {
        Some(a) => {
            let args_ = a.join(",");
            (
                if wrapping_ins_method {
                    if a.len() > 0 {
                        format!("self, {}", args_)
                    } else {
                        "self".to_string()
                    }
                } else if wrapping_cls_method {
                    if a.len() > 0 {
                        format!("cls, {}", args_)
                    } else {
                        "cls".to_string()
                    }
                } else {
                    args_.clone()
                },
                args_
            )
        },
        None => {
            (
                if wrapping_ins_method {
                    "self".to_string()
                } else if wrapping_cls_method {
                    "cls".to_string()
                } else {
                    "".to_string()
                },
                "".to_string()
            )
        }
    };

    py.run(&format!(
        r#"
def __cm__{fn_name}({prototype_args}):
    {define_scope}
    yield_context, exit_context = {scope}__enter__{fn_name}({enter_args})

    try:
        yield yield_context
        yield_return = None
    except Exception as e:
        yield_return = e

    {scope}__exit__{fn_name}(yield_return, yield_context, exit_context)

    if isinstance(yield_return, Exception):
        raise(yield_return)

from contextlib import contextmanager
{fn_name} = contextmanager(__cm__{fn_name})

if "{scope}" == "cls.":
    {fn_name} = classmethod({fn_name})
    "#,
        fn_name=fn_name,
        prototype_args=prototype_args,
        enter_args=enter_args,
        scope=scope_,
        define_scope={
            match define_scope {
                Some(ds) => {
                    ds.split("\n").collect::<Vec<&str>>().join("\n    ")
                },
                None => "".to_string()
            }
        }
        ),
        None,
        Some(locals_),
    )?;
    match locals_.get_item(fn_name) {
        Some(f) => Ok(f.to_object(py)),
        None => runtime_error!(format!("Unable to find wrapped context manager for {}", fn_name))
    }
}

cfg_if! {

    if #[cfg(debug_assertions)] {
        use pyo3::types::{PyType};

        pub(crate) fn define_tests(py: Python, test_mod: &PyModule) -> PyResult<()> {
            let subm = PyModule::new(py, "contextlib")?;
            subm.add_class::<TestClass>()?;
            subm.add_wrapped(wrap_pyfunction!(__enter__context_wrapped_function))?;
            subm.add_wrapped(wrap_pyfunction!(__exit__context_wrapped_function))?;

            let locals = PyDict::new(py);
            locals.set_item("pyapi_metal_contextlib", subm)?;
            subm.setattr(
                "context_wrapped_function",
                wrap_function(
                    py,
                    "context_wrapped_function",
                    Some(vec!("input_list")),
                    Some("origen_metal._origen_metal.__test__._helpers.contextlib"),
                    Some(locals),
                    Some("import origen_metal._origen_metal"),
                )?
            )?;

            subm.setattr(
                "context_wrapped_function_no_scope",
                wrap_function(
                    py,
                    "context_wrapped_function_no_scope",
                    Some(vec!("input_list")),
                    None,
                    None,
                    Some(
r#"
import origen_metal._origen_metal
__enter__context_wrapped_function_no_scope = origen_metal._origen_metal.__test__._helpers.contextlib.__enter__context_wrapped_function
__exit__context_wrapped_function_no_scope = origen_metal._origen_metal.__test__._helpers.contextlib.__exit__context_wrapped_function
"#
                    )
                )?
            )?;

            let tc = subm.getattr("TestClass")?;
            tc.setattr(
                "context_wrapped_cls_method",
                wrap_class_method(py, "context_wrapped_cls_method", None, None)?
            )?;

            tc.setattr(
                "context_wrapped_ins_method",
                wrap_instance_method(py, "context_wrapped_ins_method", None, None)?
            )?;

            subm.setattr("testing_context_wrapped_function", false)?;
            subm.setattr("testing_context_wrapped_function_no_scope", false)?;
            subm.setattr("testing_context_wrapped_ins_method", false)?;
            subm.setattr("testing_context_wrapped_cls_method", false)?;

            test_mod.add_submodule(subm)?;
            Ok(())
        }

        fn get_contextlib_test_mod<'py>(py: Python<'py>) -> PyResult<Py<PyModule>> {
            let test_mod = super::get_qualified_attr("origen_metal._origen_metal.__test__._helpers.contextlib")?;
            test_mod.extract::<Py<PyModule>>(py)
        }

        /// Given a list with one item, appends "added_perm" and "added_temp".
        /// Additionally, set the 'testing_context_wrapped_function' value in the test mod to True
        #[pyfunction]
        fn __enter__context_wrapped_function<'py>(py: Python<'py>, list: &'py PyList) -> PyResult<(String, &'py PyList)> {
            let test_mod = get_contextlib_test_mod(py)?;

            list.append("added_perm")?;
            list.append("added_temp")?;
            if list.get_item(0).unwrap().extract::<String>()? == "no_scope" {
                test_mod.setattr(py, "testing_context_wrapped_function_no_scope", true)?;
                Ok(("test_wrapped_function_no_scope".to_string(), list))
            } else {
                test_mod.setattr(py, "testing_context_wrapped_function", true)?;
                Ok(("test_wrapped_function".to_string(), list))
            }
        }

        /// Removes the 3rd item (index 2) from the list, which is ssumed to be "add_temp"
        #[pyfunction]
        fn __exit__context_wrapped_function(py: Python, yield_return: &PyAny, _yielded: String, context: &PyList) -> PyResult<()> {
            let test_mod = get_contextlib_test_mod(py)?;
            context.del_item(2)?;

            let indicator = context.get_item(0).unwrap().extract::<String>()?;
            if indicator == "no_scope" {
                test_mod.setattr(py, "testing_context_wrapped_function_no_scope", false)?;
                return Ok(())
            } else if indicator == "error_test" {
                let e = yield_return.downcast::<pyo3::exceptions::PyException>()?;
                context.append(format!(
                    "Found exception for error test: {}",
                    e.to_string().trim(),
                ))?;
            }
            test_mod.setattr(py, "testing_context_wrapped_function", false)?;
            Ok(())
        }

        #[pyclass]
        struct TestClass {
            ins_count: u32
        }

        #[pymethods]
        impl TestClass {
            #[new]
            fn new() -> PyResult<Self> {
                Ok(Self {
                    ins_count: 0,
                })
            }

            #[classmethod]
            fn __enter__context_wrapped_cls_method<'py>(cls: &PyType, py: Python<'py>) -> PyResult<((String, u32), Option<&'py PyDict>)> {
                let test_mod = get_contextlib_test_mod(py)?;
                test_mod.setattr(py, "testing_context_wrapped_cls_method", true)?;

                let cls_count: u32;
                if cls.hasattr("count")? {
                    cls_count = cls.getattr("count")?.extract::<u32>()? + 1;
                } else {
                    cls_count = 1;
                }
                cls.setattr("count", cls_count)?;
                Ok((
                    ("test_wrapped_cls_method".to_string(), cls_count),
                    None
                ))
            }

            #[classmethod]
            fn __exit__context_wrapped_cls_method(_cls: &PyType, py: Python, _yield_return: &PyAny, _yielded: (String, u32), _context: Option<&PyDict>) -> PyResult<()> {
                let test_mod = get_contextlib_test_mod(py)?;
                test_mod.setattr(py, "testing_context_wrapped_cls_method", false)?;
                Ok(())
            }

            fn __enter__context_wrapped_ins_method<'py>(&mut self, py: Python<'py>) -> PyResult<((String, u32), Option<&'py PyDict>)> {
                let test_mod = get_contextlib_test_mod(py)?;
                test_mod.setattr(py, "testing_context_wrapped_ins_method", true)?;

                self.ins_count += 1;
                Ok((
                    ("test_wrapped_ins_method".to_string(), self.ins_count),
                    None
                ))
            }

            fn __exit__context_wrapped_ins_method(&self, py: Python, _yield_return: &PyAny, _yielded: (String, u32), _context: Option<&PyDict>) -> PyResult<()> {
                let test_mod = get_contextlib_test_mod(py)?;
                test_mod.setattr(py, "testing_context_wrapped_ins_method", false)?;
                Ok(())
            }
        }
    }
}