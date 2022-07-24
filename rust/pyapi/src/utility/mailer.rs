use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyapi_metal::_helpers::{new_py_obj, pytype_from_str};
use origen_metal::utils::mailer::MaillistsTOMLConfig;
use pyapi_metal::utils::mailer::{Mailer, Maillists, OM_MAILER_CLASS_QP, OM_MAILLISTS_CLASS_QP};

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "mailer")?;
    subm.add_wrapped(wrap_pyfunction!(boot_mailer))?;
    subm.add_wrapped(wrap_pyfunction!(boot_maillists))?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pyfunction]
pub (crate) fn boot_mailer(py: Python) -> PyResult<Option<PyObject>> {
    init_mailer(py).or_else(|e| {
        log_error!("Unable to initialize mailer:");
        log_error!("{}", e.to_string());
        Ok(None)
    })
}

pub (crate) fn init_mailer(py: Python) -> PyResult<Option<PyObject>> {
    if let Some(mc) = &origen::ORIGEN_CONFIG.mailer {
        log_trace!("Booting Mailer from Origen config...");
        let mailer_obj = new_py_obj(
            py,
            pytype_from_str(py, mc.class.as_ref().map_or(OM_MAILER_CLASS_QP, |c| c.as_str()))?,
            Some(Mailer::toml_config_into_args(py, mc)?),
            None,
        )?;
        log_trace!("... Done!");
        Ok(Some(mailer_obj.to_object(py)))
    } else {
        log_trace!("No mailer configuration found!");
        Ok(None)
    }
}

#[pyfunction]
pub (crate) fn boot_maillists(py: Python) -> PyResult<Option<PyObject>> {
    match init_maillists(py) {
        Ok(mls_obj) => Ok(Some(mls_obj)),
        Err(e) => {
            log_error!("Unable to initialize maillists:");
            log_error!("{}", e.to_string());
            Ok(None)
        }
    }
}

pub (crate) fn init_maillists(py: Python) -> PyResult<PyObject> {
    let mut default_dirs: Vec<String> = vec![];
    let mut mls_config: MaillistsTOMLConfig;

    // Check for maillists in the install directory
    if let Some(path) = &origen::STATUS.cli_location() {
        default_dirs.push(path.parent().unwrap().display().to_string());
    }

    if let Some(app) = &origen::STATUS.app {
        let mut d = app.root.clone();
        d.push("config");
        if d.exists() {
            default_dirs.push(d.display().to_string());
            d.push("maillists");
            if d.exists() {
                default_dirs.push(d.display().to_string());
            }
        }
    }

    if let Some(config) = &origen::ORIGEN_CONFIG.maillists {
        log_trace!("Booting Maillists from Origen config...");
        mls_config = config.clone();
    } else {
        log_trace!("Booting Maillists from Origen defaults...");
        mls_config = MaillistsTOMLConfig::default();
    }
    if let Some(dirs) = mls_config.directories {
        default_dirs.extend(dirs);
    }
    mls_config.directories = Some(default_dirs);

    let args_kwargs = Maillists::toml_config_into_args(
        py,
        "origen",
        Some(true),
        &mls_config
    )?;

    let mls_obj = new_py_obj(
        py,
        pytype_from_str(py, mls_config.class.as_ref().map_or(OM_MAILLISTS_CLASS_QP, |c| c.as_str()))?,
        Some(args_kwargs.0),
        Some(args_kwargs.1),
    )?;
    log_trace!("... Done!");
    Ok(mls_obj.to_object(py))
}
