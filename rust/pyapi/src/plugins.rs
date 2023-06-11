use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use pyapi_metal::key_exception;
use origen::ORIGEN_CONFIG;
use std::path::PathBuf;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "plugins")?;
    subm.add_wrapped(wrap_pyfunction!(get_plugin_roots))?;
    subm.add_wrapped(wrap_pyfunction!(display_plugin_roots))?;
    subm.add_wrapped(wrap_pyfunction!(find_plugin_roots))?;
    subm.add_wrapped(wrap_pyfunction!(collect_plugin_roots))?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyfunction]
fn get_plugin_roots<'py>(py: Python<'py>) -> PyResult<&'py PyDict> {
    let pl_roots;
    if let Some(plugins) = ORIGEN_CONFIG.plugins.as_ref() {
        if plugins.collect() {
            pl_roots = collect_plugin_roots(py)?;
        } else {
            pl_roots = PyDict::new(py);
        }

        if let Some(plugins_to_load) = plugins.load.as_ref() {
            for (n, r) in find_plugin_roots(py, plugins_to_load.iter().map( |pl| pl.name.as_str()).collect::<Vec<&str>>())?.iter() {
                pl_roots.set_item(n, r)?;
            }
        }
    } else {
        pl_roots = collect_plugin_roots(py)?;
    }
    Ok(pl_roots)
}

#[pyfunction]
fn display_plugin_roots(py: Python) -> PyResult<()> {
    for (pl, path) in get_plugin_roots(py)?.iter() {
        println!("success|{}|{}", pl.extract::<String>()?, path.extract::<PathBuf>()?.display());
    }
    Ok(())
}

#[pyfunction]
fn find_plugin_roots<'py>(py: Python<'py>, plugins: Vec<&str>) -> PyResult<&'py PyDict> {
    let l = PyDict::new(py);
    l.set_item("plugin_paths", PyDict::new(py))?;
    py.run(&format!(
r#"
from pathlib import Path
import importlib, importlib_metadata

for to_load in [{}]:
    s = importlib.util.find_spec(to_load)
    if s.origin:
        root = Path(s.origin).parent
        if root.joinpath("origen.plugin.toml").exists():
            plugin_paths[to_load] = root
    elif s.submodule_search_locations:
        for root in s.submodule_search_locations:
            root = Path(root)
            if root.joinpath("origen.plugin.toml").exists():
                plugin_paths[to_load] = root
"#,
        plugins.iter().map( |n| format!("'{}'", n)).collect::<Vec<String>>().join(",")),
        None,
        Some(l)
    )?;
    Ok(l.get_item("plugin_paths").ok_or_else( || key_exception!("Error finding plugin roots: expected 'plugin_paths' key."))?.extract()?)
}

#[pyfunction]
fn collect_plugin_roots<'py>(py: Python<'py>) -> PyResult<&'py PyDict> {
    let l = PyDict::new(py);
    l.set_item("plugin_paths", PyDict::new(py))?;
    py.run(
r#"
from pathlib import Path
import importlib, importlib_metadata

for dist in importlib_metadata.distributions():
    n = str(Path(dist._path).name).split('-')[0].lower()
    s = importlib.util.find_spec(n)
    if s:
        if s.origin:
            root = Path(s.origin).parent
            if root.joinpath("origen.plugin.toml").exists():
                plugin_paths[n] = root
        elif s.submodule_search_locations:
            for root in s.submodule_search_locations:
                root = Path(root)
                if root.joinpath("origen.plugin.toml").exists():
                    plugin_paths[n] = root
"#,
        None,
        Some(l)
    )?;
    Ok(l.get_item("plugin_paths").ok_or_else( || key_exception!("Error collecting plugin roots: expected 'plugin_paths' key."))?.extract()?)
}
