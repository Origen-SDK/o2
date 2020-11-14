use super::template_loader::TestTemplate;
use super::{Constraint, ParamType, ParamValue};
use crate::testers::SupportedTester;
use crate::Result;
use indexmap::IndexMap;
use std::str::FromStr;

/// This is an abstract data object which is used to model test instances on Teradyne platforms
/// and both test methods and test suites on Advantest platforms.
/// A test template is modelled as a Test where indirect = true, which means that it will never be
/// rendered directly to a test program output file, however it can be referenced as a parent by
/// direct tests which are to be rendered.
/// Child tests can add additional prameters/aliases/defaults and/or inherit or override those from
/// parent tests.
#[derive(Debug, Clone, Serialize)]
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
    pub class_name: Option<String>,
    pub test_id: Option<usize>,
    // Should remain private, this is to ensure there is no direct construction of test objects
    _private: bool,
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
            class_name: None,
            /// If the test is modelling an invocation then this will reflect the ID of the
            /// test being invoked
            test_id: None,
            _private: true,
        }
    }

    /// Applies the values read from a test template file (e.g. JSON) to the current test object
    pub fn import_test_template(&mut self, test_template: &TestTemplate) -> Result<()> {
        self.class_name = test_template.class_name.to_owned();
        if let Some(params) = &test_template.parameters {
            for (name, param) in params {
                let kind = match &param.kind {
                    Some(k) => match ParamType::from_str(k) {
                        Err(msg) => {
                            return error!(
                                "{} (for parameter '{}' in test template '{}')",
                                msg, name, &self.name
                            )
                        }
                        Ok(t) => t,
                    },
                    None => ParamType::String,
                };
                self.params.insert(name.to_owned(), kind.clone());
                if let Some(aliases) = &param.aliases {
                    for alias in aliases {
                        self.aliases.insert(alias.to_owned(), name.to_owned());
                    }
                }
                if let Some(value) = &param.value {
                    let v = self.import_value(&kind, name, value)?;
                    self.values.insert(name.to_owned(), v);
                }
                if let Some(accepted_values) = &param.accepted_values {
                    let mut values: Vec<ParamValue> = vec![];
                    for value in accepted_values {
                        values.push(self.import_value(&kind, name, value)?);
                    }
                    self.constraints
                        .insert(name.to_owned(), vec![Constraint::In(values)]);
                }
            }
        }
        if let Some(params) = &test_template.parameter_list {
            for (name, type_str) in params {
                match ParamType::from_str(&type_str) {
                    Err(msg) => {
                        return error!(
                            "{} (for parameter '{}' in test template '{}')",
                            msg, name, &self.name
                        )
                    }
                    Ok(t) => {
                        self.params.insert(name.to_string(), t);
                    }
                }
            }
        }
        if let Some(aliases) = &test_template.aliases {
            for (new, old) in aliases {
                if self.params.contains_key(old) {
                    self.aliases.insert(new.to_owned(), old.to_owned());
                } else {
                    return error!("Invalid alias: test template '{}' has no parameter '{}' (being aliased to '{}')", &self.name, old, new);
                }
            }
        }
        if let Some(values) = &test_template.values {
            for (name, value) in values {
                let param_name = { self.to_param_name(name)?.to_owned() };
                let v = self.import_value(self.get_type(&param_name)?, name, value)?;
                self.values.insert(param_name, v);
            }
        }
        if let Some(accepted_values) = &test_template.accepted_values {
            for (name, accepted_values) in accepted_values {
                let param_name = { self.to_param_name(name)?.to_owned() };
                let mut values: Vec<ParamValue> = vec![];
                for value in accepted_values {
                    values.push(self.import_value(self.get_type(&param_name)?, name, value)?);
                }
                self.constraints
                    .insert(param_name, vec![Constraint::In(values)]);
            }
        }
        Ok(())
    }

    fn import_value(
        &self,
        kind: &ParamType,
        name: &str,
        value: &serde_json::Value,
    ) -> Result<ParamValue> {
        match kind {
            &ParamType::String => match value.as_str() {
                Some(x) => Ok(ParamValue::String(x.to_owned())),
                None => error!(
                    "Value given for '{}' in test '{}' is not a string: '{:?}'",
                    name, &self.name, value
                ),
            },
            &ParamType::Int => match value.as_i64() {
                Some(x) => Ok(ParamValue::Int(x)),
                None => error!(
                    "Value given for '{}' in test '{}' is not an integer: '{:?}'",
                    name, &self.name, value
                ),
            },
            &ParamType::UInt => match value.as_u64() {
                Some(x) => Ok(ParamValue::UInt(x)),
                None => error!(
                    "Value given for '{}' in test '{}' is not an unsigned integer: '{:?}'",
                    name, &self.name, value
                ),
            },
            &ParamType::Float => match value.as_f64() {
                Some(x) => Ok(ParamValue::Float(x)),
                None => error!(
                    "Value given for '{}' in test '{}' is not a float: '{:?}'",
                    name, &self.name, value
                ),
            },
            &ParamType::Current => match value.as_f64() {
                Some(x) => Ok(ParamValue::Current(x)),
                None => error!(
                    "Value given for '{}' in test '{}' is not a number: '{:?}'",
                    name, &self.name, value
                ),
            },
            &ParamType::Voltage => match value.as_f64() {
                Some(x) => Ok(ParamValue::Voltage(x)),
                None => error!(
                    "Value given for '{}' in test '{}' is not a number: '{:?}'",
                    name, &self.name, value
                ),
            },
            &ParamType::Time => match value.as_f64() {
                Some(x) => Ok(ParamValue::Time(x)),
                None => error!(
                    "Value given for '{}' in test '{}' is not a number: '{:?}'",
                    name, &self.name, value
                ),
            },
            &ParamType::Frequency => match value.as_f64() {
                Some(x) => Ok(ParamValue::Frequency(x)),
                None => error!(
                    "Value given for '{}' in test '{}' is not a number: '{:?}'",
                    name, &self.name, value
                ),
            },
            &ParamType::Bool => match value.as_bool() {
                Some(x) => Ok(ParamValue::Bool(x)),
                None => error!(
                    "Value given for '{}' in test '{}' is not a boolean: '{:?}'",
                    name, &self.name, value
                ),
            },
            &ParamType::Any => Ok(ParamValue::Any(format!("{}", value))),
        }
    }

    /// Set the value of the given parameter to the given value, returns an error if the
    /// parameter is not found (unless allow_missing = true), if its type does match the type of the
    /// given value or if any constraints placed on the possible values is violated
    pub fn set(
        &mut self,
        param_name_or_alias: &str,
        value: ParamValue,
        allow_mising: bool,
    ) -> Result<()> {
        if allow_mising && !self.has_param(param_name_or_alias) {
            return Ok(());
        }
        let param_name = { self.to_param_name(param_name_or_alias)?.to_owned() };
        let kind = self.get_type(&param_name)?;
        if value.is_type(kind) || kind == &ParamType::Any {
            if let Some(constraints) = self.constraints.get(&param_name) {
                for constraint in constraints {
                    if let Err(e) = constraint.is_satisfied(&value) {
                        return error!(
                            "Illegal value applied to attribute '{}' of test '{}': {}",
                            param_name_or_alias, &self.name, e
                        );
                    }
                }
            }
            self.values.insert(param_name, value);
            Ok(())
        } else {
            error!("The type of the given value for '{}' in test '{}' does not match the required type: expected {:?}, given {:?}", param_name, &self.name, kind, value)
        }
    }

    /// Get the value of the given attribute
    pub fn get(&self, param_name_or_alias: &str) -> Result<Option<&ParamValue>> {
        let param_name = self.to_param_name(param_name_or_alias)?;
        Ok(self.values.get(param_name))
    }

    /// Returns the type of the given parameter name (or alias), returns an error if the
    /// parameter is not found
    pub fn get_type(&self, param_name_or_alias: &str) -> Result<&ParamType> {
        let param_name = self.to_param_name(param_name_or_alias)?;
        Ok(&self.params[param_name])
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

    /// Resolves the given param or alias name to a param name
    pub fn to_param_name<'a>(&'a self, name: &'a str) -> Result<&'a str> {
        if self.params.contains_key(name) {
            Ok(name)
        } else if self.aliases.contains_key(name) {
            Ok(&self.aliases[name])
        } else {
            error!(
                "Test '{}' does not have a parameter named '{}'",
                self.name, name
            )
        }
    }
}
