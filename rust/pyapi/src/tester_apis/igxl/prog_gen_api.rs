use super::IGXL;
use crate::prog_gen::{Group, PatternGroup, Test, TestInvocation};
use crate::utility::caller::src_caller_meta;
use origen::prog_gen::{flow_api, GroupType, ParamValue, PatternGroupType};
use origen::testers::SupportedTester;
use origen::{Error, Result};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

#[pyclass]
#[derive(Debug, Clone)]
pub struct Patset {
    pub name: String,
    pub tester: SupportedTester,
    pub id: usize,
}

#[pymethods]
impl IGXL {
    #[args(library = "None", kwargs = "**")]
    fn new_test_instance(
        &mut self,
        name: String,
        template: String,
        library: Option<String>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        let library = match library {
            Some(x) => x,
            None => "std".to_string(),
        };

        let t = Test::new(
            name.clone(),
            self.tester.to_owned(),
            library,
            template,
            kwargs,
        )?;

        t.set_attr("test_name", Some(ParamValue::String(name)))?;

        Ok(t)
    }

    #[args(kwargs = "**")]
    pub fn new_flow_line(&mut self, kwargs: Option<&PyDict>) -> PyResult<TestInvocation> {
        let t = TestInvocation::new("_".to_owned(), self.tester.to_owned(), kwargs)?;
        Ok(t)
    }

    #[args(pattern = "None", patterns = "None")]
    fn new_patset(
        &mut self,
        name: String,
        pattern: Option<&PyAny>,
        patterns: Option<&PyAny>,
    ) -> PyResult<PatternGroup> {
        let pg = PatternGroup::new(name, self.tester.clone(), Some(PatternGroupType::Patset))?;
        if let Some(p) = pattern {
            for pat in extract_vec_string("pattern", p)? {
                pg.append(pat, None)?;
            }
        }
        if let Some(p) = patterns {
            for pat in extract_vec_string("patterns", p)? {
                pg.append(pat, None)?;
            }
        }
        Ok(pg)
    }

    // Set the cpu wait flags for the given test instance
    #[args(flags = "*")]
    fn set_wait_flags(&mut self, test_instance: &Test, flags: &PyTuple) -> PyResult<()> {
        let mut clean_flags: Vec<String> = vec![];
        for fl in flags {
            let mut bad = true;
            if let Ok(f) = fl.extract::<String>() {
                match f.to_lowercase().as_str() {
                    "a" | "b" | "c" | "d" => {
                        clean_flags.push(f.to_lowercase().to_owned());
                        bad = false;
                    }
                    _ => {}
                }
            }
            if bad {
                return Err(PyErr::from(Error::new(&format!(
                "Illegal argument given to set_wait_flags '{}', should be a String flag name, e.g. \"a\", \"b\", etc.",
                fl
            ))));
            }
        }
        flow_api::set_wait_flags(test_instance.id, clean_flags, src_caller_meta())?;
        Ok(())
    }

    fn test_instance_group(&mut self, name: String) -> PyResult<Group> {
        let g = Group::new(name, Some(self.tester.to_owned()), GroupType::Test, None);
        Ok(g)
    }
}

fn extract_vec_string(arg_name: &str, val: &PyAny) -> Result<Vec<String>> {
    if let Ok(v) = val.extract::<String>() {
        Ok(vec![v])
    } else if let Ok(v) = val.extract::<Vec<String>>() {
        Ok(v)
    } else {
        bail!(
            "Illegal value for argument '{}', expected a String or a List of Strings, got: {}",
            arg_name,
            val
        )
    }
}
