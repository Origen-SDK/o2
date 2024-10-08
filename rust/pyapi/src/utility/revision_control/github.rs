use pyo3::prelude::*;
use pyapi_metal::PyOutcome;
use std::collections::HashMap;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "github")?;
    subm.add_wrapped(wrap_pyfunction!(get_current_workflow_name))?;
    subm.add_wrapped(wrap_pyfunction!(dispatch_workflow))?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyfunction]
pub fn get_current_workflow_name() -> PyResult<Option<String>> {
    Ok(origen::utility::github::get_current_workflow_name()?)
}

#[pyfunction]
#[pyo3(signature=(owner, repo, workflow, git_ref, inputs=None))]
pub fn dispatch_workflow(
    owner: &str,
    repo: &str,
    workflow: &str,
    git_ref: &str,
    inputs: Option<HashMap<String, String>>,
) -> PyResult<PyOutcome> {
    let res = origen::utility::github::dispatch_workflow(owner, repo, workflow, git_ref, inputs)?;
    Ok(PyOutcome::from_origen(res))
}
