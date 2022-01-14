use crate::Result;
use std::fmt::Display;
use crate::{TypedValue, TypedValueMap, TypedValueVec, Outcome};
use super::DataStoreCategoryFrontendAPI;

// TODO needed?
#[derive(Debug, Clone, Display, PartialEq)]
pub enum DataStoreFeature {
    Get,
    Store,
    PopulateUser,
    Other(String),
}

// TODO clean up
pub trait DataStoreFrontendAPI {
    fn name(&self) -> Result<&str>;
    fn category(&self) -> Result<Box<dyn DataStoreCategoryFrontendAPI>>;
    // fn features(&self) -> &Vec<DataStoreFeature>;
    fn class(&self, backend: Option<TypedValueMap>) -> Result<String>;
    // fn init(&self, parameters: TypedValueMap) -> Result<Option<Outcome>>;

    fn call(&self, func: &str, pos_args: Option<TypedValueVec>, kw_args: Option<TypedValueMap>, backend: Option<TypedValueMap>) -> Result<Outcome>;

    /// Gets an object from the datastore, returning 'None' if it doesn't exists
    fn get(&self, key: &str) -> Result<Option<TypedValue>>;

    fn contains(&self, query: &str) -> Result<bool>;

    fn remove(&self, key: &str) -> Result<Option<TypedValue>>;

    fn store(&self, key: &str, obj: TypedValue) -> Result<bool>;

    fn items(&self) -> Result<TypedValueMap>;

    fn keys(&self) -> Result<Vec<String>> {
        Ok(self.items()?.typed_values().keys().map(|k| k.to_string()).collect())
    }

    // /// Custom function to populate a user
    // // fn populate_user(user: &User) -> (bool, Result<Outcome>) {
    // //     (false, _feature_not_implemented("populate_user"))
    // // }

    // /// Generic handler for checking if a feature is support and/or implemented.
    // /// Aside: unimplemented features are runtime issues, not compile time issues.
    // fn _feature_error(&self, func_name: &str, feature: &DataStoreFeature) -> String {
    //     if self.features().contains(feature) {
    //         self._feature_not_implemented(feature)
    //     } else {
    //         self._feature_not_supported(feature)
    //     }
    // }

    // fn _feature_not_supported(&self, feature: &DataStoreFeature) -> String {
    //     format!(
    //         "'{}' does not support feature '{}' (data store category: '{}')",
    //         self.name(),
    //         feature.to_string(),
    //         self.category().to_string()
    //     )
    // }

    // fn _feature_not_implemented(&self, feature: &DataStoreFeature) -> String {
    //     format!(
    //         "'{}' does not implemented feature '{}' (data store category: '{}')",
    //         self.name(),
    //         feature.to_string(),
    //         self.category().to_string()
    //     )
    // }
}
