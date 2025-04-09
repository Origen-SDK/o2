use origen_metal::prog_gen::test_ids::{TestIDs, AllocationOptions};
use pyo3::{prelude::*, exceptions::{PyValueError, PyRuntimeError}};
use std::{path::PathBuf, collections::HashMap};

pub(crate) fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "test_ids")?;
    subm.add_class::<TestIDInterface>()?;
    m.add_submodule(subm)?;
    Ok(())
}

/// A single LDAP instance
#[pyclass(subclass)]
pub struct TestIDInterface {
    om: TestIDs,
    file: Option<PathBuf>,
}

impl TestIDInterface {
}

#[pymethods]
impl TestIDInterface {
    #[new]
    #[pyo3(signature=(file=None))]
    fn new(
        file: Option<PathBuf>,
    ) -> PyResult<Self> {
        let tids;
        if file.is_some() && file.as_ref().unwrap().exists() {
            tids = TestIDs::from_file(file.as_ref().unwrap())?;
        } else {
            tids = TestIDs::new();
        }
        Ok(Self {
            om: tids,
            file: file,
        })
    }
    
    #[pyo3(signature=(file=None))]
    fn save(&self, file: Option<PathBuf>) -> PyResult<()> {
        if let Some(f) = file {
            self.om.save(&f)?;
        } else if let Some(f) = &self.file {
            self.om.save(f)?;
        } else {
            return Err(PyRuntimeError::new_err("Cannot save a TestIDs object that was not created from a file without providing a file path at save time"));
        }
        Ok(())
    }
    

    fn allocate(&mut self, test_name: String, no_bin: bool, no_softbin: bool, no_number: bool,
                           bin: Option<u32>, softbin: Option<u32>, number: Option<u32>) -> PyResult<HashMap<String, u32>> {
        let opts = AllocationOptions {
            bin: bin,
            softbin: softbin,
            number: number,
            no_bin: no_bin,
            no_softbin: no_softbin,
            no_number: no_number,
        };
        let allocation = self.om.allocate_with_options(&test_name, &opts)?;
        Ok(allocation.to_hashmap())
    }
    
    fn configure(&mut self, kind: String, exclude: bool, start: u32, stop: Option<u32>) -> PyResult<()> {
        match kind.as_str() {
            "bin" => {
                if let Some(stop) = stop {
                    if exclude {
                        self.om.bins.exclude.push_range(start, stop);
                    } else {
                        self.om.bins.include.push_range(start, stop);
                    }
                } else {
                    if exclude {
                        self.om.bins.exclude.push(start);
                    } else {
                        self.om.bins.include.push(start);
                    }
                }
            }
            "softbin" => {
                if let Some(stop) = stop {
                    if exclude {
                        self.om.softbins.exclude.push_range(start, stop);
                    } else {
                        self.om.softbins.include.push_range(start, stop);
                    }
                } else {
                    if exclude {
                        self.om.softbins.exclude.push(start);
                    } else {
                        self.om.softbins.include.push(start);
                    }
                }
            }
            "number" => {
                if let Some(stop) = stop {
                    if exclude {
                        self.om.numbers.exclude.push_range(start, stop);
                    } else {
                        self.om.numbers.include.push_range(start, stop);
                    }
                } else {
                    if exclude {
                        self.om.numbers.exclude.push(start);
                    } else {
                        self.om.numbers.include.push(start);
                    }
                }
            }
            _ => {
                return Err(PyValueError::new_err(format!("Unknown kind '{}', must be one of 'bin', 'softbin', or 'number'", kind)));
            }
        }
        Ok(())
    }
    
    
}