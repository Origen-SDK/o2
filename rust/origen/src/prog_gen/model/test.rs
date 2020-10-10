use super::{Constraint, ParamType, ParamValue};
use crate::testers::SupportedTester;
use crate::Result;
use indexmap::IndexMap;

/// This is an abstract data object which is used to model test instances on Teradyne platforms
/// and both test methods and test suites on Advantest platforms.
/// A test template is modelled as a Test where indirect = true, which means that it will never be
/// rendered directly to a test program output file, however it can be referenced as a parent by
/// direct tests which are to be rendered.
/// Child tests can add additional prameters/aliases/defaults and/or inherit or override those from
/// parent tests.
#[derive(Debug)]
pub struct Test {
    pub id: usize,
    pub name: String,
    pub indirect: bool,
    /// Defines the names of parameters and their types. Child class can override the type of a parameter
    /// inherited from a parent by adding a parameter of the same name to their params map. Then can also
    /// add additional parameters via the same mechanism. It is not possible to delete a parameter inherited
    /// from a parent Test.
    pub params: IndexMap<String, ParamType>,
    pub values: IndexMap<String, ParamValue>,
    pub aliases: IndexMap<String, String>,
    pub constraints: IndexMap<String, Vec<Constraint>>,
    pub tester: SupportedTester,
}

impl Test {
    pub fn new(name: &str, id: usize, tester: SupportedTester) -> Test {
        Test {
            id: id,
            name: name.to_string(),
            indirect: false,
            params: IndexMap::new(),
            values: IndexMap::new(),
            aliases: IndexMap::new(),
            constraints: IndexMap::new(),
            tester: tester,
        }
    }

    /// Set the value of the given parameter to the given value, returns an error if the
    /// parameter is not found or if its type does match the type of the given value
    pub fn set(&mut self, param_name: &str, value: ParamValue) -> Result<()> {
        let kind = self.get_type(param_name)?;
        if value.is_type(kind) {
            self.values.insert(param_name.to_string(), value);
            Ok(())
        } else {
            error!("The type of the given value for '{}' in test '{}' does not match the required type: expected {:?}, given {:?}", param_name, &self.name, kind, value)
        }
    }

    /// Returns the type of the given parameter name (or alias), returns an error if the
    /// parameter is not found
    pub fn get_type(&self, param_name: &str) -> Result<&ParamType> {
        if let Some(p) = self.params.get(param_name) {
            Ok(p)
        } else if self.aliases.contains_key(param_name) {
            self.get_type(&self.aliases[param_name])
        } else {
            error!(
                "Test '{}' has no parameter named '{}'",
                &self.name, param_name
            )
        }
    }

    pub fn add_param(
        &mut self,
        name: &str,
        kind: ParamType,
        default_value: Option<ParamValue>,
        aliases: Option<Vec<&str>>,
        constraints: Option<Vec<Constraint>>,
    ) -> Result<()> {
        self.params.insert(name.to_string(), kind);
        if let Some(x) = default_value {
            self.values.insert(name.to_string(), x);
        }
        if let Some(aliases) = aliases {
            for alias in aliases {
                self.aliases.insert(name.to_string(), alias.to_string());
            }
        }
        if let Some(x) = constraints {
            self.constraints.insert(name.to_string(), x);
        }
        Ok(())
    }

    /// Define a simple alias for an existing parameter, returns an error if the
    /// parameter is not found
    pub fn add_alias(&mut self, alias: &str, param_name: &str) -> Result<()> {
        if self.has_param(param_name) {
            self.aliases
                .insert(alias.to_string(), param_name.to_string());
            Ok(())
        } else {
            error!(
                "Test '{}' has no parameter named '{}'",
                &self.name, param_name
            )
        }
    }

    /// Returns true if the test has the given parameter name or alias
    pub fn has_param(&self, param_name: &str) -> bool {
        self.params.contains_key(param_name) || self.aliases.contains_key(param_name)
    }
}
