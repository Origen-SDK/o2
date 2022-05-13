pub mod data_store;
pub mod data_store_category;

use crate::Result;
use std::any::Any;
use indexmap::IndexMap;
use std::sync::RwLockReadGuard;

pub use crate::utils::revision_control::frontend::RevisionControlFrontendAPI;
pub use data_store::{DataStoreFrontendAPI, DataStoreFeature, FeatureReturn};
pub use data_store_category::DataStoreCategoryFrontendAPI;

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

pub fn require<'a>() -> Result<RwLockReadGuard<'a, Frontend>> {
    let f = crate::FRONTEND.read().unwrap();
    if f.frontend.is_none() {
        bail!("No frontend is currently available!");
    }
    Ok(f)
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

pub fn with_optional_frontend<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(Option<&dyn FrontendAPI>) -> Result<T>,
{
    let f = crate::FRONTEND.read().unwrap();
    // func(f.frontend.as_ref())
    match f.frontend.as_ref() {
        Some(f) => func(Some(f.as_ref())),
        None => func(None),
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

    pub fn with_data_store_category<T, F>(&self, category: &str, mut func: F) -> Result<T>
    where
        F: FnMut(Box<dyn DataStoreCategoryFrontendAPI>) -> Result<T>,
    {
        self.with_frontend( |f| {
            let cat = f.require_data_store_category(category)?;
            func(cat)
        })
    }

    pub fn with_data_store<T, F>(&self, category: &str, data_store: &str, mut func: F) -> Result<T>
    where
        F: FnMut(Box<dyn DataStoreFrontendAPI>) -> Result<T>,
    {
        self.with_frontend( |f| {
            let cat = f.require_data_store_category(category)?;
            let ds = cat.require_data_store(data_store)?;
            func(ds)
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

    fn data_store_categories(&self) -> Result<IndexMap<String, Box<dyn DataStoreCategoryFrontendAPI>>>;
    fn get_data_store_category(&self, category: &str) -> Result<Option<Box<dyn DataStoreCategoryFrontendAPI>>>;
    fn add_data_store_category(&self, category: &str) -> Result<Box<dyn DataStoreCategoryFrontendAPI>>;
    fn remove_data_store_category(&self, category: &str) -> Result<()>;

    fn require_data_store_category(&self, category: &str) -> Result<Box<dyn DataStoreCategoryFrontendAPI>> {
        match self.get_data_store_category(category)? {
            Some(cat) => Ok(cat),
            None => bail!("Required data store category '{}' was not found!", category)
        }
    }

    fn contains_data_store_category(&self, category: &str) -> Result<bool> {
        Ok(match self.get_data_store_category(category)? {
            Some(_) => true,
            None => false
        })
    }

    // TEST_NEEDED
    fn ensure_data_store_category(&self, category: &str) -> Result<(bool, Box<dyn DataStoreCategoryFrontendAPI>)> {
        Ok(match self.get_data_store_category(category)? {
            Some(cat) => (false, cat),
            None => {
                (true, self.add_data_store_category(category)?)
            }
        })
    }

    fn available_data_store_categories(&self) -> Result<Vec<String>> {
        Ok(self.data_store_categories()?.keys().map( |k| k.to_string()).collect())
    }

    fn lookup_current_user(&self) -> Option<Result<Option<String>>> {
        None
    }

    fn as_any(&self) -> &dyn Any;
}
