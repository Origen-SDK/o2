// FOR_PR figure out what'sn needed here vs. what's staying in FE
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use pyapi_metal::{pypath, key_error, key_exception};
use origen::ORIGEN_CONFIG;
use origen_metal::indexmap::IndexMap;
use std::path::PathBuf;
use pyo3::exceptions::PyKeyError;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "plugins")?;
    subm.add_wrapped(wrap_pyfunction!(from_origen_cli))?;
    subm.add_wrapped(wrap_pyfunction!(default))?;
    subm.add_wrapped(wrap_pyfunction!(get_plugin_roots))?;
    subm.add_wrapped(wrap_pyfunction!(display_plugin_roots))?;
    subm.add_wrapped(wrap_pyfunction!(find_plugin_roots))?;
    subm.add_wrapped(wrap_pyfunction!(collect_plugin_roots))?;
    subm.add_class::<Plugins>()?;
    subm.add_class::<Plugin>()?;
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

#[pyfunction]
pub fn from_origen_cli(py: Python, plugin_configs: &PyDict) -> PyResult<Plugins> {
    Plugins::from_pl_config_dict(py, plugin_configs)
}

// FOR_PR not sure what's needed from below

#[pyfunction]
pub fn default(py: Python) -> PyResult<Plugins> {
    Plugins::from_roots(py)
}

// #[pymodule]
// pub fn plugins(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_wrapped(plugins);
//     m.add_class::<Plugins>()?;
//     m.add_class::<Plugin>()?;
//     Ok(())
// }

#[pyclass]
pub struct Plugins {
    plugins: IndexMap<String, Py<Plugin>>
}

#[pymethods]
impl Plugins {
    #[getter]
    pub fn plugins<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let retn = PyDict::new(py);
        for (n, pl) in self.plugins.iter() {
            retn.set_item(n, pl)?;
        }
        Ok(retn)
    }

    #[getter]
    pub fn names(&self) -> PyResult<Vec<String>> {
        self.keys()
    }

    pub fn get(&self, key: &str) -> PyResult<Option<&Py<Plugin>>> {
        Ok(if let Some(pl) = self.plugins.get(key) {
            // PyDataStoreCategory::autoload_category(cat.borrow(py).into(), py)?;
            Some(pl)
        } else {
            None
        })
    }

    fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self.plugins.keys().map(|k| k.to_string()).collect())
    }

    // TODO  dictlike
    // fn values(&self) -> PyResult<Vec<&Py<Plugin>>> {
    //     Ok(self.plugins.iter().map(|(_, cat)| cat).collect())
    // }

    // TODO  dictlike
    // fn items(&self) -> PyResult<Vec<(String, &Py<Plugin>)>> {
    //     Ok(self
    //         .plugins
    //         .iter()
    //         .map(|(n, cat)| (n.to_string(), cat))
    //         .collect())
    // }

    fn __getitem__(&self, key: &str) -> PyResult<&Py<Plugin>> {
        if let Some(pl) = self.get(key)? {
            Ok(pl)
        } else {
            key_error!(format!("Unknown plugin '{}'", key))
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.plugins.len())
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PluginsIter> {
        Ok(PluginsIter {
            keys: slf.keys().unwrap(),
            i: 0,
        })
    }
}

impl Plugins {
    pub fn from_pl_config_dict(py: Python, plugin_configs: &PyDict) -> PyResult<Self> {
        Ok(Self {
            plugins: {
                let mut plugins = IndexMap::new();
                for (n, pl) in plugin_configs {
                    let name = n.extract::<String>()?;
                    plugins.insert(name.to_owned(), {
                        let cfg = pl.extract::<&PyDict>()?;
                        let r = cfg.get_item("root").ok_or_else(|| PyKeyError::new_err(format!("A 'root' is required for plugin '{}'", name)))?.extract::<PathBuf>()?;
                        Py::new(py, Plugin {
                            name: name,
                            root: r,
                        })?
                    });
                }
                plugins
            }
        })
    }

    pub fn from_roots(py: Python) -> PyResult<Self> {
        let roots = get_plugin_roots(py)?;
        Ok(Self {
            plugins: {
                let mut plugins = IndexMap::new();
                for (n, r) in roots {
                    let name = n.extract::<String>()?;
                    let root = r.extract::<PathBuf>()?;
                    plugins.insert(name.to_owned(), Py::new(py, Plugin { name: name, root: root })?);
                    // plugins.insert(name.to_owned(), {
                    //     let cfg = pl.extract::<&PyDict>()?;
                    //     let r = cfg.get_item("root").ok_or_else(|| PyKeyError::new_err(format!("A 'root' is required for plugin '{}'", name)))?.extract::<PathBuf>()?;
                    //     Py::new(py, Plugin {
                    //         name: name,
                    //         root: r,
                    //     })?
                    // });
                }
                plugins
            }
        })
    }
}

#[pyclass]
pub struct PluginsIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pymethods]
impl PluginsIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        if slf.i >= slf.keys.len() {
            return Ok(None);
        }
        let name = slf.keys[slf.i].clone();
        slf.i += 1;
        Ok(Some(name))
    }
}

#[pyclass]
pub struct Plugin {
    name: String,
    root: PathBuf,
}

#[pymethods]
impl Plugin {
    #[getter]
    pub fn name(&self) -> PyResult<&String> {
        Ok(&self.name)
    }

    #[getter]
    pub fn root(&self, py: Python) -> PyResult<PyObject> {
        Ok(pypath!(py, self.root.display()))
    }
}
