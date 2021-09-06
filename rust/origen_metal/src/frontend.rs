use crate::Result;
use std::any::Any;

pub use crate::utils::revision_control::frontend::RevisionControlFrontendAPI;

pub fn set_frontend(
    frontend: Box<dyn FrontendAPI + std::marker::Sync + std::marker::Send>,
) -> Result<()> {
    let mut f = crate::FRONTEND.write().unwrap();
    f.set_frontend(frontend)?;
    Ok(())
}

pub fn reset() -> Result<()> {
    let mut f = crate::FRONTEND.write().unwrap();
    f.reset()?;
    Ok(())
}

pub fn frontend_set() -> Result<bool> {
    let f = crate::FRONTEND.read().unwrap();
    Ok(f.frontend.is_some())
}

pub fn with_frontend<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&dyn FrontendAPI) -> Result<T>,
{
    let f = crate::FRONTEND.read().unwrap();
    match f.frontend.as_ref() {
        Some(f) => func(f.as_ref()),
        None => bail!("No frontend is currently available!"),
    }
}

pub struct Frontend {
    frontend: Option<Box<dyn FrontendAPI + std::marker::Sync + std::marker::Send>>,
}

impl Frontend {
    pub fn new() -> Self {
        Self { frontend: None }
    }

    pub fn frontend(&self) -> Option<&dyn FrontendAPI> {
        match self.frontend.as_ref() {
            Some(f) => Some(f.as_ref()),
            None => None,
        }
    }

    pub fn set_frontend(
        &mut self,
        frontend: Box<dyn FrontendAPI + std::marker::Sync + std::marker::Send>,
    ) -> Result<()> {
        self.frontend = Some(frontend);
        Ok(())
    }

    pub fn with_frontend<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&dyn FrontendAPI) -> Result<T>,
    {
        match self.frontend.as_ref() {
            Some(f) => func(f.as_ref()),
            None => bail!("No frontend is currently available!"),
        }
    }

    pub fn with_optional_frontend<T, F>(&self, mut func: F) -> Result<Option<T>>
    where
        F: FnMut(&dyn FrontendAPI) -> Result<T>,
    {
        Ok(match self.frontend.as_ref() {
            Some(f) => Some(func(f.as_ref())?),
            None => None,
        })
    }

    pub fn reset(&mut self) -> Result<()> {
        self.frontend = None;
        Ok(())
    }
}

pub trait FrontendAPI {
    fn revision_control(&self) -> Result<Option<&dyn RevisionControlFrontendAPI>>;

    fn rc(&self) -> Result<Option<&dyn RevisionControlFrontendAPI>> {
        self.revision_control()
    }

    fn require_rc(&self) -> Result<&dyn RevisionControlFrontendAPI> {
        match self.rc()? {
            Some(rc) => Ok(rc),
            None => bail!(&self.default_failed_require_message("revision control")),
        }
    }

    fn default_failed_require_message(&self, obj: &str) -> String {
        format!(
            "A {} component is required to run the previous operation, but is not available!",
            obj,
        )
    }

    fn as_any(&self) -> &dyn Any;
}
