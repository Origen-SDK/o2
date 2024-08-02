use super::DataStoreCategoryFrontendAPI;
use crate::Result;
use crate::{Outcome, TypedValue, TypedValueMap, TypedValueVec};

// TODO needed?
#[derive(Debug, Clone, Display, PartialEq)]
pub enum DataStoreFeature {
    Get,
    Store,
    PopulateUser,
    Other(String),
}

pub struct FeatureReturn {
    implemented: bool,
    outcome: Result<Outcome>,
}

impl FeatureReturn {
    pub fn new(result: Result<Outcome>) -> Self {
        Self {
            implemented: true,
            outcome: result,
        }
    }

    pub fn new_unimplemented(msg: String) -> Self {
        Self {
            implemented: false,
            outcome: Err(error!(&msg)),
        }
    }

    pub fn outcome(&self) -> Result<&Outcome> {
        match &self.outcome {
            Ok(o) => Ok(o),
            Err(e) => Err((*e).clone()),
        }
    }

    pub fn implemented(&self) -> bool {
        self.implemented
    }
}

// TODO clean up
pub trait DataStoreFrontendAPI {
    fn name(&self) -> Result<&str>;
    fn category(&self) -> Result<Box<dyn DataStoreCategoryFrontendAPI>>;
    // fn features(&self) -> &Vec<DataStoreFeature>;
    fn class(&self, backend: Option<TypedValueMap>) -> Result<String>;
    // fn init(&self, parameters: TypedValueMap) -> Result<Option<Outcome>>;
    // fn load(&self)

    fn call(
        &self,
        func: &str,
        pos_args: Option<TypedValueVec>,
        kw_args: Option<TypedValueMap>,
        backend: Option<TypedValueMap>,
    ) -> Result<Outcome>;

    /// Gets an object from the datastore, returning 'None' if it doesn't exists
    fn get(&self, key: &str) -> Result<Option<TypedValue>>;

    fn contains(&self, query: &str) -> Result<bool>;

    fn remove(&self, key: &str) -> Result<Option<TypedValue>>;

    fn store(&self, key: &str, obj: TypedValue) -> Result<bool>;

    fn items(&self) -> Result<TypedValueMap>;

    fn keys(&self) -> Result<Vec<String>> {
        Ok(self
            .items()?
            .typed_values()
            .keys()
            .map(|k| k.to_string())
            .collect())
    }

    //--- User Features ---//

    /// Custom function to populate a user
    fn populate_user(&self, _user_id: &str, _ds_name: &str) -> Result<FeatureReturn> {
        // (false, _feature_not_implemented("populate_user"))
        self.unimplemented(current_func!())
    }

    /// Custom function to validate a user's password
    fn validate_password(
        &self,
        _username: &str,
        _password: &str,
        _user_id: &str,
        _ds_name: &str,
    ) -> Result<FeatureReturn> {
        self.unimplemented(current_func!())
    }

    fn unimplemented(&self, feature: &str) -> Result<FeatureReturn> {
        Ok(FeatureReturn::new_unimplemented(
            self._feature_not_implemented(feature)?,
        ))
    }

    fn _feature_not_implemented(&self, feature: &str) -> Result<String> {
        Ok(format!(
            "'{}' does not implement feature '{}' (data store category: '{}')",
            self.name()?,
            feature.to_string(),
            self.category()?.name()
        ))
    }
}
