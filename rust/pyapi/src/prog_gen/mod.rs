//! Implements Python bindings for program generation data structures and functions

use origen::core::tester::TesterSource;
use origen_metal::prog_gen::Model;
use origen_metal::prog_gen::SupportedTester;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
use std::thread;
use regex::Regex;
use std::path::PathBuf;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "prog_gen")?;
    subm.add_wrapped(wrap_pyfunction!(render))?;
    subm.add_wrapped(wrap_pyfunction!(resolve_file_reference))?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyfunction]
fn resolve_file_reference(path: &str) -> PyResult<String> {
    let file = origen::with_current_job(|job| {
        let mut pt = PathBuf::from(".");
        for p in Regex::new(r"(\\|/)").unwrap().split(path) {
            pt.push(p);
        }
        job.resolve_file_reference(&pt, Some(vec!["py"]))
    })?;
    Ok(file.to_str().unwrap().to_string())
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
                        bail!("Python based tester targets are not supported for program generation yet, no action taken for target: {}", g);
                        //Ok((vec![], Model::new(g)))
                    }
                    TesterSource::Internal(t) => {
                        let mut tester = origen::tester();
                        let files = tester.render_program_for_target_at(i, true);
                        match files {
                            Err(e) => {
                                let msg = e.to_string();
                                if continue_on_fail {
                                    origen::STATUS.inc_unhandled_error_count();
                                    log_error!("{}", &msg);
                                    Ok((vec![], Model::new(t.id_prog_gen())))
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
        let mut models: HashMap<SupportedTester, Model> = HashMap::new();
        for thread in threads {
            match thread.join() {
                Err(_e) => log_error!("Something has gone wrong when doing the final program render"),
                Ok(v) => match v {
                    Err(e) => log_error!("{}", e),
                    Ok(paths_and_model) => {
                        for path in &paths_and_model.0 {
                            generated_files.push(format!("{}", path.display()));
                        }
                        models.insert(paths_and_model.1.tester.clone(), paths_and_model.1);
                    }
                }
            }
        }

        // Could hand over the model here in future to allow the app to generate additional output from it

        Ok(generated_files)
    })
}

