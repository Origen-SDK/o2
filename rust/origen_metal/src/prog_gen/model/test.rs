use super::template_loader::{TestTemplate, TestTemplateCollection, TestTemplateParameter};
use super::Model;
use super::{Constraint, Limit, ParamType, ParamValue};
use crate::prog_gen::supported_testers::SupportedTester;
use crate::Result;
use indexmap::IndexMap;
use std::str::FromStr;

pub const TEST_NUMBER_ALIASES: [&str; 6] = ["testnumber", "test_number", "number", "testnum", "test_num", "tnum"];

#[derive(Debug, Clone, Serialize)]
pub struct TestCollection {
    pub name: String,
    pub params: IndexMap<String, ParamType>,
    pub default_values: IndexMap<String, ParamValue>,
    pub aliases: IndexMap<String, String>,
    pub constraints: IndexMap<String, Vec<Constraint>>,
    pub collections: IndexMap<String, TestCollection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestCollectionItem {
    pub id: usize,
    pub parent_id: usize,
    pub collection_name: String,
    pub instance_id: String,
    pub available: bool,
    pub params: IndexMap<String, ParamType>,
    pub values: IndexMap<String, ParamValue>,
    pub default_values: IndexMap<String, ParamValue>,
    pub aliases: IndexMap<String, String>,
    pub constraints: IndexMap<String, Vec<Constraint>>,
    pub collection_defs: IndexMap<String, TestCollection>,
    pub collections: IndexMap<String, Vec<usize>>,
    _private: (),
}

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
    pub tname: Option<String>, // Secondary test name, if applicable
    pub indirect: bool,
    /// Defines the names of parameters and their types. Child class can override the type of a parameter
    /// inherited from a parent by adding a parameter of the same name to their params map. Then can also
    /// add additional parameters via the same mechanism. It is not possible to delete a parameter inherited
    /// from a parent Test.
    pub params: IndexMap<String, ParamType>,
    pub values: IndexMap<String, ParamValue>,
    pub default_values: IndexMap<String, ParamValue>,
    pub aliases: IndexMap<String, String>,
    pub constraints: IndexMap<String, Vec<Constraint>>,
    pub collection_defs: IndexMap<String, TestCollection>,
    pub collections: IndexMap<String, Vec<usize>>,
    pub tester: SupportedTester,
    pub class_name: Option<String>,
    /// References an invocation and vice versa
    pub test_id: Option<usize>,
    pub sub_tests: Vec<usize>,
    pub number: Option<usize>,
    /// Tests can directly have a single set of limits, tests with multiple limits are modeled as sub-tests
    pub lo_limit: Option<Limit>,
    pub hi_limit: Option<Limit>,
    // Should remain private, this is to ensure there is no direct construction of test objects
    _private: (),
}

pub struct SortedParams<'a> {
    test: &'a Test,
    sorted_keys: Vec<&'a String>,
}

impl<'a> SortedParams<'a> {
    fn new(test: &'a Test) -> Self {
        let mut keys: Vec<&String> = test.params.keys().collect();
        keys.sort();
        keys.reverse();
        SortedParams {
            test: test,
            sorted_keys: keys,
        }
    }
}

impl<'a> Iterator for SortedParams<'a> {
    type Item = (&'a str, &'a ParamType, Option<&'a ParamValue>);

    // Here, we define the sequence using `.curr` and `.next`.
    // The return type is `Option<T>`:
    //     * When the `Iterator` is finished, `None` is returned.
    //     * Otherwise, the next value is wrapped in `Some` and returned.
    fn next(&mut self) -> Option<(&'a str, &'a ParamType, Option<&'a ParamValue>)> {
        if let Some(k) = self.sorted_keys.pop() {
            Some((
                k,
                self.test.params.get(k).unwrap(),
                self.test.get(k).unwrap(),
            ))
        } else {
            None
        }
    }
}

impl TestCollection {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            params: IndexMap::new(),
            default_values: IndexMap::new(),
            aliases: IndexMap::new(),
            constraints: IndexMap::new(),
            collections: IndexMap::new(),
        }
    }

    pub fn import_test_template(
        &mut self,
        template: &TestTemplateCollection,
        owner_name: &str,
    ) -> Result<()> {
        if let Some(params) = &template.parameters {
            let mut values = IndexMap::new();
            for (name, param) in params {
                import_template_parameter(
                    name,
                    param,
                    owner_name,
                    &mut self.params,
                    &mut values,
                    &mut self.default_values,
                    &mut self.aliases,
                    &mut self.constraints,
                )?;
            }
        }
        if let Some(collections) = &template.collections {
            for (name, collection) in collections {
                let mut c = TestCollection::new(name);
                c.import_test_template(collection, owner_name)?;
                self.collections.insert(name.to_owned(), c);
            }
        }
        Ok(())
    }
}

impl TestCollectionItem {
    pub fn from_collection(
        id: usize,
        parent_id: usize,
        instance_id: &str,
        collection: &TestCollection,
    ) -> Self {
        Self {
            id,
            parent_id,
            collection_name: collection.name.clone(),
            instance_id: instance_id.to_string(),
            available: true,
            params: collection.params.clone(),
            values: collection.default_values.clone(),
            default_values: collection.default_values.clone(),
            aliases: collection.aliases.clone(),
            constraints: collection.constraints.clone(),
            collection_defs: collection.collections.clone(),
            collections: IndexMap::new(),
            _private: (),
        }
    }

    pub fn unavailable(id: usize, parent_id: usize, collection_name: &str, instance_id: &str) -> Self {
        Self {
            id,
            parent_id,
            collection_name: collection_name.to_string(),
            instance_id: instance_id.to_string(),
            available: false,
            params: IndexMap::new(),
            values: IndexMap::new(),
            default_values: IndexMap::new(),
            aliases: IndexMap::new(),
            constraints: IndexMap::new(),
            collection_defs: IndexMap::new(),
            collections: IndexMap::new(),
            _private: (),
        }
    }

    pub fn set(
        &mut self,
        param_name_or_alias: &str,
        value: Option<ParamValue>,
        allow_missing: bool,
    ) -> Result<()> {
        if !self.available {
            return Ok(());
        }
        if allow_missing && !self.has_param(param_name_or_alias) {
            return Ok(());
        }
        let param_name = { self.to_param_name(param_name_or_alias)?.to_owned() };
        let kind = self.get_type(&param_name)?;
        if let Some(value) = value {
            let value = coerce_param_value(kind, value)?;
            if value.is_type(kind) || kind == &ParamType::Any {
                if let Some(constraints) = self.constraints.get(&param_name) {
                    for constraint in constraints {
                        if let Err(e) = constraint.is_satisfied(&value) {
                            bail!(
                                "Illegal value applied to attribute '{}' of collection item '{}[{}]': {}",
                                param_name_or_alias,
                                self.collection_name,
                                self.instance_id,
                                e
                            );
                        }
                    }
                }
                self.values.insert(param_name, value);
                Ok(())
            } else {
                bail!(
                    "The type of the given value for '{}' in collection item '{}[{}]' does not match the required type: expected {:?}, given {:?}",
                    param_name,
                    self.collection_name,
                    self.instance_id,
                    kind,
                    value
                )
            }
        } else {
            self.values.remove(&param_name);
            Ok(())
        }
    }

    pub fn has_param(&self, param_name: &str) -> bool {
        self.params.contains_key(param_name) || {
            let n = clean(param_name);
            self.aliases.contains_key(&n)
        }
    }

    pub fn get_type(&self, param_name_or_alias: &str) -> Result<&ParamType> {
        let param_name = self.to_param_name(param_name_or_alias)?;
        Ok(&self.params[param_name])
    }

    pub fn to_param_name<'a>(&'a self, name: &'a str) -> Result<&'a str> {
        if self.params.contains_key(name) {
            Ok(name)
        } else {
            let n = clean(name);
            if self.aliases.contains_key(&n) {
                Ok(&self.aliases[&n])
            } else {
                let available_attributes = self.params.keys().collect::<Vec<&String>>();
                let mut msg = format!(
                    "Collection item '{}[{}]' does not have an attribute named '{}'",
                    self.collection_name,
                    self.instance_id,
                    name
                );
                msg += "\nThe available attributes are:";
                for attr in available_attributes {
                    let attr_type: &ParamType = &self.params[attr];
                    msg += &format!("\n  - {} ({})", attr, attr_type);
                }
                bail!(&msg);
            }
        }
    }
}

impl Test {
    pub fn new(name: &str, id: usize, tester: SupportedTester) -> Test {
        let mut t = Test {
            id: id,
            name: name.to_string(),
            tname: None,
            indirect: false,
            params: IndexMap::new(),
            values: IndexMap::new(),
            default_values: IndexMap::new(),
            aliases: IndexMap::new(),
            constraints: IndexMap::new(),
            collection_defs: IndexMap::new(),
            collections: IndexMap::new(),
            tester: tester,
            class_name: None,
            // If the test is modelling an invocation then this will reflect the ID of the
            // test being invoked
            test_id: None,
            sub_tests: vec![],
            number: None,
            lo_limit: None,
            hi_limit: None,
            _private: (),
        };
        let clean_name = clean(name);
        if clean_name != name {
            t.aliases.insert(clean_name, name.to_owned());
        }
        t
    }

    pub fn sorted_params(&self) -> SortedParams {
        SortedParams::new(&self)
    }

    /// Applies the values read from a test template file (e.g. JSON) to the current test object
    pub fn import_test_template(&mut self, test_template: &TestTemplate) -> Result<()> {
        self.class_name = test_template.class_name.to_owned();
        if let Some(params) = &test_template.parameters {
            for (name, param) in params {
                import_template_parameter(
                    name,
                    param,
                    &self.name,
                    &mut self.params,
                    &mut self.values,
                    &mut self.default_values,
                    &mut self.aliases,
                    &mut self.constraints,
                )?;
            }
        }
        if let Some(collections) = &test_template.collections {
            for (name, collection) in collections {
                let mut c = TestCollection::new(name);
                c.import_test_template(collection, &self.name)?;
                self.collection_defs.insert(name.to_owned(), c);
            }
        }
        if let Some(params) = &test_template.parameter_list {
            for (name, type_str) in params {
                match ParamType::from_str(&type_str) {
                    Err(msg) => {
                        bail!(
                            "{} (for parameter '{}' in test template '{}')",
                            msg,
                            name,
                            &self.name
                        )
                    }
                    Ok(t) => {
                        self.params.insert(name.to_string(), t);
                        let clean_name = clean(name);
                        if &clean_name != name {
                            self.aliases.insert(clean_name, name.to_owned());
                        }
                    }
                }
            }
        }
        if let Some(aliases) = &test_template.aliases {
            for (new, old) in aliases {
                if self.params.contains_key(old) {
                    self.aliases.insert(clean(new), old.to_owned());
                } else {
                    bail!("Invalid alias: test template '{}' has no parameter '{}' (being aliased to '{}')", &self.name, old, new);
                }
            }
        }
        if let Some(values) = &test_template.values {
            for (name, value) in values {
                let param_name = { self.to_param_name(name)?.to_owned() };
                let v = import_value(self.get_type(&param_name)?, name, &self.name, value)?;
                self.values.insert(param_name, v);
            }
        }
        if let Some(accepted_values) = &test_template.accepted_values {
            for (name, accepted_values) in accepted_values {
                let param_name = { self.to_param_name(name)?.to_owned() };
                let mut values: Vec<ParamValue> = vec![];
                for value in accepted_values {
                    values.push(import_value(
                        self.get_type(&param_name)?,
                        name,
                        &self.name,
                        value,
                    )?);
                }
                self.constraints
                    .insert(param_name, vec![Constraint::In(values)]);
            }
        }
        Ok(())
    }

    /// Set the value of the given parameter to the given value, returns an error if the
    /// parameter is not found (unless allow_missing = true), if its type does match the type of the
    /// given value or if any constraints placed on the possible values is violated.
    /// Supplying None for the value will cause any existing value assignment for the given parameter
    /// to be removed.
    pub fn set(
        &mut self,
        param_name_or_alias: &str,
        value: Option<ParamValue>,
        allow_mising: bool,
    ) -> Result<()> {
        if allow_mising && !self.has_param(param_name_or_alias) {
            return Ok(());
        }
        let param_name = { self.to_param_name(param_name_or_alias)?.to_owned() };
        let kind = self.get_type(&param_name)?;
        if let Some(value) = value {
            let value = coerce_param_value(kind, value)?;
            if value.is_type(kind) || kind == &ParamType::Any {
                if let Some(constraints) = self.constraints.get(&param_name) {
                    for constraint in constraints {
                        if let Err(e) = constraint.is_satisfied(&value) {
                            bail!(
                                "Illegal value applied to attribute '{}' of test '{}': {}",
                                param_name_or_alias,
                                &self.name,
                                e
                            );
                        }
                    }
                }
                self.values.insert(param_name, value);
                Ok(())
            } else {
                bail!("The type of the given value for '{}' in test '{}' does not match the required type: expected {:?}, given {:?}", param_name, &self.name, kind, value)
            }
        } else {
            self.values.remove(&param_name);
            Ok(())
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
                self.aliases.insert(clean(name), alias.to_string());
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
            self.aliases.insert(clean(alias), param_name.to_string());
            Ok(())
        } else {
            bail!(
                "Test '{}' has no parameter named '{}'",
                &self.name,
                param_name
            )
        }
    }

    /// Returns true if the test has the given parameter name or alias
    pub fn has_param(&self, param_name: &str) -> bool {
        self.params.contains_key(param_name) || {
            let n = clean(param_name);
            self.aliases.contains_key(&n)
        }
    }

    /// Resolves the given param or alias name to a param name
    pub fn to_param_name<'a>(&'a self, name: &'a str) -> Result<&'a str> {
        if self.params.contains_key(name) {
            Ok(name)
        } else {
            let n = clean(name);
            if self.aliases.contains_key(&n) {
                Ok(&self.aliases[&n])
            } else {
                let available_attributes = self.params.keys().collect::<Vec<&String>>();
                let mut msg = format!(
                    "Test '{}' does not have an attribute named '{}'",
                    self.name,
                    name
                );
                msg += "\nThe available attributes are:";
                for attr in available_attributes {
                    let attr_type: &ParamType = &self.params[attr];
                    msg += &format!("\n  - {} ({})", attr, attr_type);
                }
                bail!(&msg);
            }
        }
    }

    /// Returns the invocation for the test if there is one
    pub fn invocation<'a>(&self, model: &'a Model) -> Option<&'a Test> {
        if let Some(inv_id) = self.test_id {
            return model.test_invocations.get(&inv_id);
        }
        None
    }

    /// Returns a mutable reference to invocation for the test if there is one
    pub fn invocation_mut<'a>(&self, model: &'a mut Model) -> Option<&'a mut Test> {
        if let Some(inv_id) = self.test_id {
            return model.test_invocations.get_mut(&inv_id);
        }
        None
    }

    /// Returns the test for the invocation if there is one
    pub fn test<'a>(&self, model: &'a Model) -> Option<&'a Test> {
        if let Some(test_id) = self.test_id {
            return model.tests.get(&test_id);
        }
        None
    }

    /// Returns a mutable reference to the test for the invocation if there is one
    pub fn test_mut<'a>(&self, model: &'a mut Model) -> Option<&'a mut Test> {
        if let Some(test_id) = self.test_id {
            return model.tests.get_mut(&test_id);
        }
        None
    }
}

fn import_template_parameter(
    name: &str,
    param: &TestTemplateParameter,
    owner_name: &str,
    params: &mut IndexMap<String, ParamType>,
    values: &mut IndexMap<String, ParamValue>,
    default_values: &mut IndexMap<String, ParamValue>,
    aliases: &mut IndexMap<String, String>,
    constraints: &mut IndexMap<String, Vec<Constraint>>,
) -> Result<()> {
    let kind = match &param.kind {
        Some(k) => match ParamType::from_str(k) {
            Err(msg) => {
                bail!(
                    "{} (for parameter '{}' in test template '{}')",
                    msg,
                    name,
                    owner_name
                )
            }
            Ok(t) => t,
        },
        None => ParamType::String,
    };
    params.insert(name.to_owned(), kind.clone());
    let clean_name = clean(name);
    if &clean_name != name {
        aliases.insert(clean_name, name.to_owned());
    }
    if let Some(param_aliases) = &param.aliases {
        for alias in param_aliases {
            aliases.insert(clean(alias), name.to_owned());
        }
    }
    if let Some(value) = &param.value {
        let v = import_value(&kind, name, owner_name, value)?;
        values.insert(name.to_owned(), v.clone());
        default_values.insert(name.to_owned(), v);
    }
    if let Some(accepted_values) = &param.accepted_values {
        let mut accepted: Vec<ParamValue> = vec![];
        for value in accepted_values {
            accepted.push(import_value(&kind, name, owner_name, value)?);
        }
        constraints.insert(name.to_owned(), vec![Constraint::In(accepted)]);
    }
    Ok(())
}

fn coerce_param_value(kind: &ParamType, value: ParamValue) -> Result<ParamValue> {
    Ok(match (kind, value) {
        (ParamType::Any, value) => value,
        (ParamType::Int, ParamValue::UInt(v)) => match i64::try_from(v) {
            Ok(v) => ParamValue::Int(v),
            Err(_) => ParamValue::UInt(v),
        },
        (ParamType::UInt, ParamValue::Int(v)) if v >= 0 => ParamValue::UInt(v as u64),
        (_, value) => value,
    })
}

fn import_value(
    kind: &ParamType,
    name: &str,
    owner_name: &str,
    value: &serde_json::Value,
) -> Result<ParamValue> {
    match kind {
        ParamType::String => match value {
            serde_json::Value::String(x) => Ok(ParamValue::String(x.to_owned())),
            _ => bail!(
                "Value given for '{}' in test '{}' is not a string: '{:?}'",
                name,
                owner_name,
                value
            ),
        },
        ParamType::Int => parse_i64(value).map(ParamValue::Int).ok_or_else(|| {
            crate::Error::new(&format!(
                "Value given for '{}' in test '{}' is not an integer: '{:?}'",
                name, owner_name, value
            ))
        }),
        ParamType::UInt => parse_u64(value).map(ParamValue::UInt).ok_or_else(|| {
            crate::Error::new(&format!(
                "Value given for '{}' in test '{}' is not an unsigned integer: '{:?}'",
                name, owner_name, value
            ))
        }),
        ParamType::Float => parse_f64(value).map(ParamValue::Float).ok_or_else(|| {
            crate::Error::new(&format!(
                "Value given for '{}' in test '{}' is not a float: '{:?}'",
                name, owner_name, value
            ))
        }),
        ParamType::Current => parse_f64(value).map(ParamValue::Current).ok_or_else(|| {
            crate::Error::new(&format!(
                "Value given for '{}' in test '{}' is not a number: '{:?}'",
                name, owner_name, value
            ))
        }),
        ParamType::Voltage => parse_f64(value).map(ParamValue::Voltage).ok_or_else(|| {
            crate::Error::new(&format!(
                "Value given for '{}' in test '{}' is not a number: '{:?}'",
                name, owner_name, value
            ))
        }),
        ParamType::Time => parse_f64(value).map(ParamValue::Time).ok_or_else(|| {
            crate::Error::new(&format!(
                "Value given for '{}' in test '{}' is not a number: '{:?}'",
                name, owner_name, value
            ))
        }),
        ParamType::Frequency => {
            parse_f64(value).map(ParamValue::Frequency).ok_or_else(|| {
                crate::Error::new(&format!(
                    "Value given for '{}' in test '{}' is not a number: '{:?}'",
                    name, owner_name, value
                ))
            })
        }
        ParamType::Bool => parse_bool(value).map(ParamValue::Bool).ok_or_else(|| {
            crate::Error::new(&format!(
                "Value given for '{}' in test '{}' is not a boolean: '{:?}'",
                name, owner_name, value
            ))
        }),
        ParamType::Any => Ok(ParamValue::Any(match value {
            serde_json::Value::String(v) => v.clone(),
            _ => value.to_string(),
        })),
    }
}

fn parse_i64(value: &serde_json::Value) -> Option<i64> {
    if let Some(v) = value.as_i64() {
        Some(v)
    } else if let Some(v) = value.as_u64() {
        i64::try_from(v).ok()
    } else if let Some(v) = value.as_str() {
        v.parse::<i64>().ok()
    } else {
        None
    }
}

fn parse_u64(value: &serde_json::Value) -> Option<u64> {
    if let Some(v) = value.as_u64() {
        Some(v)
    } else if let Some(v) = value.as_i64() {
        u64::try_from(v).ok()
    } else if let Some(v) = value.as_str() {
        v.parse::<u64>().ok()
    } else {
        None
    }
}

fn parse_f64(value: &serde_json::Value) -> Option<f64> {
    if let Some(v) = value.as_f64() {
        Some(v)
    } else if let Some(v) = value.as_str() {
        match v.trim() {
            "Infinity" => Some(f64::INFINITY),
            "-Infinity" => Some(f64::NEG_INFINITY),
            other => other.parse::<f64>().ok(),
        }
    } else {
        None
    }
}

fn parse_bool(value: &serde_json::Value) -> Option<bool> {
    if let Some(v) = value.as_bool() {
        Some(v)
    } else if let Some(v) = value.as_str() {
        match v.trim().to_ascii_lowercase().as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    } else {
        None
    }
}

fn clean(name: &str) -> String {
    name.to_lowercase().replace("_", "").replace(".", "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prog_gen::supported_testers::SupportedTester;

    #[test]
    fn imports_recursive_collection_defs_and_stringified_values() {
        let template: TestTemplate = serde_json::from_str(
            r#"{
                "class_name": "com.example.Test",
                "parameters": {
                    "enableSoftset": {
                        "kind": "boolean",
                        "value": "true"
                    }
                },
                "collections": {
                    "tsen": {
                        "parameters": {
                            "zDataDeltaLimit": {
                                "kind": "double",
                                "value": "0.0"
                            }
                        },
                        "collections": {
                            "registers": {
                                "parameters": {
                                    "zLowLimit": {
                                        "kind": "double",
                                        "value": "-Infinity"
                                    }
                                }
                            }
                        }
                    }
                }
            }"#,
        )
        .unwrap();

        let mut test = Test::new("example", 1, SupportedTester::V93KSMT8);
        test.import_test_template(&template).unwrap();

        assert_eq!(
            test.values.get("enableSoftset"),
            Some(&ParamValue::Bool(true))
        );
        let tsen = test.collection_defs.get("tsen").unwrap();
        assert_eq!(
            tsen.default_values.get("zDataDeltaLimit"),
            Some(&ParamValue::Float(0.0))
        );
        let registers = tsen.collections.get("registers").unwrap();
        assert_eq!(
            registers.default_values.get("zLowLimit"),
            Some(&ParamValue::Float(f64::NEG_INFINITY))
        );
    }

    #[test]
    fn test_set_coerces_uint_to_int_for_flat_test_attrs() {
        let mut test = Test::new("example", 1, SupportedTester::V93KSMT8);
        test.add_param("width", ParamType::Int, None, None, None)
            .unwrap();

        test.set("width", Some(ParamValue::UInt(8)), false).unwrap();

        assert_eq!(test.values.get("width"), Some(&ParamValue::Int(8)));
    }

    #[test]
    fn test_set_coerces_uint_to_int_for_collection_item_attrs() {
        let mut collection = TestCollection::new("softsetVariables");
        collection
            .params
            .insert("width".to_string(), ParamType::Int);
        let mut item = TestCollectionItem::from_collection(1, 0, "v3", &collection);

        item.set("width", Some(ParamValue::UInt(8)), false).unwrap();

        assert_eq!(item.values.get("width"), Some(&ParamValue::Int(8)));
    }

    #[test]
    fn test_set_coerces_non_negative_int_to_uint() {
        let mut test = Test::new("example", 1, SupportedTester::V93KSMT8);
        test.add_param("count", ParamType::UInt, None, None, None)
            .unwrap();

        test.set("count", Some(ParamValue::Int(8)), false).unwrap();

        assert_eq!(test.values.get("count"), Some(&ParamValue::UInt(8)));
    }

    #[test]
    fn test_set_rejects_negative_int_for_uint() {
        let mut test = Test::new("example", 1, SupportedTester::V93KSMT8);
        test.add_param("count", ParamType::UInt, None, None, None)
            .unwrap();

        let err = test
            .set("count", Some(ParamValue::Int(-1)), false)
            .unwrap_err();

        assert!(err
            .to_string()
            .contains("does not match the required type"));
    }
}
