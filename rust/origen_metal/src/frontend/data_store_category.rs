use crate::{Result, TypedValueMap};
use super::DataStoreFrontendAPI;
use std::collections::HashMap;

/// Allows for arbitrary "data sources". This could be, for example, a database (any), LDAP, just a random file, etc.
/// Just need a few methods implemented for frontend <-> backend integration. Others are available but are optional.
pub trait DataStoreCategoryFrontendAPI {
    fn name(&self) -> &str;
    fn get_data_store(&self, store: &str) -> Result<Option<Box<dyn DataStoreFrontendAPI>>>;
    fn add_data_store(&self, name: &str, parameters: TypedValueMap, backend_details: Option<TypedValueMap>) -> Result<Box<dyn DataStoreFrontendAPI>>;

    fn data_stores(&self) -> Result<HashMap<String, Box<dyn DataStoreFrontendAPI>>>;

    fn require_data_store(&self, store: &str) -> Result<Box<dyn DataStoreFrontendAPI>> {
        match self.get_data_store(store)? {
            Some(ds) => Ok(ds),
            None => bail!("Required data store '{}' not found in category '{}'", store, self.name())
        }
    }

    fn available_data_stores(&self) -> Result<Vec<String>> {
        Ok(self.data_stores()?.keys().map(|k| k.to_string()).collect::<Vec<String>>())
    }

    fn contains_data_store(&self, store: &str) -> Result<bool> {
        Ok(self.data_stores()?.contains_key(store))
    }

    fn remove_data_store(&self, store: &str) -> Result<()>;
}
