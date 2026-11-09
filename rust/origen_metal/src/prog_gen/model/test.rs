use super::template_loader::{TestTemplate, TestTemplateCollection, TestTemplateParameter};
use super::Model;
use super::{Constraint, Limit, ParamType, ParamValue};
use crate::prog_gen::supported_testers::SupportedTester;
use crate::Result;
use indexmap::IndexMap;
use serde::ser::SerializeStruct;
use std::str::FromStr;
use std::sync::Arc;

pub const TEST_NUMBER_ALIASES: [&str; 6] = [
    "testnumber",
    "test_number",
    "number",
    "testnum",
    "test_num",
    "tnum",
];

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
///
/// The parameters, defaults, aliases, constraints and collection definitions are a
/// `TestDefinition`, which the tests made from one template share (a program can have tens of
/// thousands of them), see `definition_mut()`.
#[derive(Debug, Clone)]
pub struct Test {
    pub id: usize,
    pub name: String,
    pub tname: Option<String>, // Secondary test name, if applicable
    pub indirect: bool,
    /// The parameters, defaults, aliases, constraints and collection definitions, shared with the
    /// other tests made from the same template
    pub definition: Arc<TestDefinition>,
    pub values: IndexMap<String, ParamValue>,
    pub collections: IndexMap<String, Vec<usize>>,
    pub tester: SupportedTester,
    pub class_name: Option<String>,
    /// The template library this test method was loaded from.
    pub template_library: Option<String>,
    /// The template name within `template_library`.
    pub template_name: Option<String>,
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

/// The part of a `Test` that comes from its template
#[derive(Debug, Clone, Default, Serialize)]
pub struct TestDefinition {
    /// Defines the names of parameters and their types. Child class can override the type of a parameter
    /// inherited from a parent by adding a parameter of the same name to their params map. Then can also
    /// add additional parameters via the same mechanism. It is not possible to delete a parameter inherited
    /// from a parent Test.
    pub params: IndexMap<String, ParamType>,
    pub default_values: IndexMap<String, ParamValue>,
    pub aliases: IndexMap<String, String>,
    pub constraints: IndexMap<String, Vec<Constraint>>,
    pub collection_defs: IndexMap<String, TestCollection>,
}

impl serde::Serialize for Test {
    /// The fields of a test, in the order they had before the definition was shared
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Test", 21)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("tname", &self.tname)?;
        s.serialize_field("indirect", &self.indirect)?;
        s.serialize_field("params", &self.definition.params)?;
        s.serialize_field("values", &self.values)?;
        s.serialize_field("default_values", &self.definition.default_values)?;
        s.serialize_field("aliases", &self.definition.aliases)?;
        s.serialize_field("constraints", &self.definition.constraints)?;
        s.serialize_field("collection_defs", &self.definition.collection_defs)?;
        s.serialize_field("collections", &self.collections)?;
        s.serialize_field("tester", &self.tester)?;
        s.serialize_field("class_name", &self.class_name)?;
        s.serialize_field("template_library", &self.template_library)?;
        s.serialize_field("template_name", &self.template_name)?;
        s.serialize_field("test_id", &self.test_id)?;
        s.serialize_field("sub_tests", &self.sub_tests)?;
        s.serialize_field("number", &self.number)?;
        s.serialize_field("lo_limit", &self.lo_limit)?;
        s.serialize_field("hi_limit", &self.hi_limit)?;
        s.serialize_field("_private", &self._private)?;
        s.end()
    }
}

pub struct SortedParams<'a> {
    test: &'a Test,
    sorted_keys: Vec<&'a String>,
}

impl<'a> SortedParams<'a> {
    fn new(test: &'a Test) -> Self {
        let mut keys: Vec<&String> = test.definition.params.keys().collect();
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
                self.test.definition.params.get(k).unwrap(),
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

    pub fn unavailable(
        id: usize,
        parent_id: usize,
        collection_name: &str,
        instance_id: &str,
    ) -> Self {
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
            self.values.shift_remove(&param_name);
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
                    self.collection_name, self.instance_id, name
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
            definition: Arc::new(TestDefinition::default()),
            values: IndexMap::new(),
            collections: IndexMap::new(),
            tester: tester,
            class_name: None,
            template_library: None,
            template_name: None,
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
            t.definition_mut()
                .aliases
                .insert(clean_name, name.to_owned());
        }
        t
    }

    /// The test's definition, to change it: it is copied first if other tests share it
    pub fn definition_mut(&mut self) -> &mut TestDefinition {
        Arc::make_mut(&mut self.definition)
    }

    pub fn sorted_params(&self) -> SortedParams<'_> {
        SortedParams::new(&self)
    }

    /// Applies the values read from a test template file (e.g. JSON) to the current test object
    pub fn import_test_template(&mut self, test_template: &TestTemplate) -> Result<()> {
        self.class_name = test_template.class_name.to_owned();
        if let Some(params) = &test_template.parameters {
            let definition: &mut TestDefinition = Arc::make_mut(&mut self.definition);
            for (name, param) in params {
                import_template_parameter(
                    name,
                    param,
                    &self.name,
                    &mut definition.params,
                    &mut self.values,
                    &mut definition.default_values,
                    &mut definition.aliases,
                    &mut definition.constraints,
                )?;
            }
        }
        if let Some(collections) = &test_template.collections {
            for (name, collection) in collections {
                let mut c = TestCollection::new(name);
                c.import_test_template(collection, &self.name)?;
                self.definition_mut()
                    .collection_defs
                    .insert(name.to_owned(), c);
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
                        let definition: &mut TestDefinition = self.definition_mut();
                        definition.params.insert(name.to_string(), t);
                        let clean_name = clean(name);
                        if &clean_name != name {
                            definition.aliases.insert(clean_name, name.to_owned());
                        }
                    }
                }
            }
        }
        if let Some(aliases) = &test_template.aliases {
            for (new, old) in aliases {
                if self.definition.params.contains_key(old) {
                    self.definition_mut()
                        .aliases
                        .insert(clean(new), old.to_owned());
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
                self.definition_mut()
                    .constraints
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
                if let Some(constraints) = self.definition.constraints.get(&param_name) {
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
            self.values.shift_remove(&param_name);
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
        Ok(&self.definition.params[param_name])
    }

    pub fn add_param(
        &mut self,
        name: &str,
        kind: ParamType,
        default_value: Option<ParamValue>,
        aliases: Option<Vec<&str>>,
        constraints: Option<Vec<Constraint>>,
    ) -> Result<()> {
        self.definition_mut().params.insert(name.to_string(), kind);
        if let Some(x) = default_value {
            self.values.insert(name.to_string(), x);
        }
        if let Some(aliases) = aliases {
            for alias in aliases {
                self.definition_mut()
                    .aliases
                    .insert(clean(name), alias.to_string());
            }
        }
        if let Some(x) = constraints {
            self.definition_mut()
                .constraints
                .insert(name.to_string(), x);
        }
        Ok(())
    }

    /// Define a simple alias for an existing parameter, returns an error if the
    /// parameter is not found
    pub fn add_alias(&mut self, alias: &str, param_name: &str) -> Result<()> {
        if self.has_param(param_name) {
            self.definition_mut()
                .aliases
                .insert(clean(alias), param_name.to_string());
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
        self.definition.params.contains_key(param_name) || {
            let n = clean(param_name);
            self.definition.aliases.contains_key(&n)
        }
    }

    /// Resolves the given param or alias name to a param name
    pub fn to_param_name<'a>(&'a self, name: &'a str) -> Result<&'a str> {
        if self.definition.params.contains_key(name) {
            Ok(name)
        } else {
            let n = clean(name);
            if self.definition.aliases.contains_key(&n) {
                Ok(&self.definition.aliases[&n])
            } else {
                let available_attributes = self.definition.params.keys().collect::<Vec<&String>>();
                let mut msg = format!(
                    "Test '{}' does not have an attribute named '{}'",
                    self.name, name
                );
                msg += "\nThe available attributes are:";
                for attr in available_attributes {
                    let attr_type: &ParamType = &self.definition.params[attr];
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
        (ParamType::Class, ParamValue::String(v)) => ParamValue::Class(v),
        (ParamType::ListStrings, ParamValue::List(values)) => ParamValue::List(
            values
                .into_iter()
                .map(|value| match value {
                    ParamValue::String(v) | ParamValue::Class(v) => Ok(ParamValue::String(v)),
                    ParamValue::Bool(v) => Ok(ParamValue::String(v.to_string())),
                    ParamValue::Int(v) => Ok(ParamValue::String(v.to_string())),
                    ParamValue::UInt(v) => Ok(ParamValue::String(v.to_string())),
                    ParamValue::Float(v) => Ok(ParamValue::String(v.to_string())),
                    ParamValue::Current(v) => Ok(ParamValue::String(v.to_string())),
                    ParamValue::Voltage(v) => Ok(ParamValue::String(v.to_string())),
                    ParamValue::Time(v) => Ok(ParamValue::String(v.to_string())),
                    ParamValue::Frequency(v) => Ok(ParamValue::String(v.to_string())),
                    ParamValue::Any(v) => Ok(ParamValue::String(v)),
                    ParamValue::List(_) => bail!("Nested parameter lists are not supported"),
                })
                .collect::<Result<Vec<ParamValue>>>()?,
        ),
        (ParamType::ListClasses, ParamValue::List(values)) => ParamValue::List(
            values
                .into_iter()
                .map(|value| match value {
                    ParamValue::String(v) | ParamValue::Class(v) => Ok(ParamValue::Class(v)),
                    ParamValue::List(_) => bail!("Nested parameter lists are not supported"),
                    value => Ok(value),
                })
                .collect::<Result<Vec<ParamValue>>>()?,
        ),
        (ParamType::Int, ParamValue::UInt(v)) => match i64::try_from(v) {
            Ok(v) => ParamValue::Int(v),
            Err(_) => ParamValue::UInt(v),
        },
        (ParamType::UInt, ParamValue::Int(v)) if v >= 0 => ParamValue::UInt(v as u64),
        (ParamType::Float, ParamValue::Int(v)) => ParamValue::Float(v as f64),
        (ParamType::Float, ParamValue::UInt(v)) => ParamValue::Float(v as f64),
        (ParamType::Current, ParamValue::Int(v)) => ParamValue::Current(v as f64),
        (ParamType::Current, ParamValue::UInt(v)) => ParamValue::Current(v as f64),
        (ParamType::Current, ParamValue::Float(v)) => ParamValue::Current(v),
        (ParamType::Voltage, ParamValue::Int(v)) => ParamValue::Voltage(v as f64),
        (ParamType::Voltage, ParamValue::UInt(v)) => ParamValue::Voltage(v as f64),
        (ParamType::Voltage, ParamValue::Float(v)) => ParamValue::Voltage(v),
        (ParamType::Time, ParamValue::Int(v)) => ParamValue::Time(v as f64),
        (ParamType::Time, ParamValue::UInt(v)) => ParamValue::Time(v as f64),
        (ParamType::Time, ParamValue::Float(v)) => ParamValue::Time(v),
        (ParamType::Frequency, ParamValue::Int(v)) => ParamValue::Frequency(v as f64),
        (ParamType::Frequency, ParamValue::UInt(v)) => ParamValue::Frequency(v as f64),
        (ParamType::Frequency, ParamValue::Float(v)) => ParamValue::Frequency(v),
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
        ParamType::Class => match value {
            serde_json::Value::String(x) => Ok(ParamValue::Class(x.to_owned())),
            _ => bail!(
                "Value given for '{}' in test '{}' is not a class/reference string: '{:?}'",
                name,
                owner_name,
                value
            ),
        },
        ParamType::ListStrings | ParamType::ListClasses => {
            let parsed;
            let values = match value {
                serde_json::Value::Array(values) => values,
                serde_json::Value::String(encoded) => {
                    parsed = serde_json::from_str::<serde_json::Value>(encoded).map_err(|_| {
                        crate::Error::new(&format!(
                            "Value given for '{}' in test '{}' is not a JSON list: '{:?}'",
                            name, owner_name, value
                        ))
                    })?;
                    parsed.as_array().ok_or_else(|| {
                        crate::Error::new(&format!(
                            "Value given for '{}' in test '{}' is not a list: '{:?}'",
                            name, owner_name, value
                        ))
                    })?
                }
                _ => bail!(
                    "Value given for '{}' in test '{}' is not a list: '{:?}'",
                    name,
                    owner_name,
                    value
                ),
            };
            let class_items = kind == &ParamType::ListClasses;
            let mut result = Vec::with_capacity(values.len());
            for item in values {
                let item = match item {
                    serde_json::Value::String(v) if class_items => ParamValue::Class(v.clone()),
                    serde_json::Value::String(v) => ParamValue::String(v.clone()),
                    serde_json::Value::Bool(v) => ParamValue::Bool(*v),
                    serde_json::Value::Number(v) if v.is_i64() => {
                        ParamValue::Int(v.as_i64().unwrap())
                    }
                    serde_json::Value::Number(v) if v.is_u64() => {
                        ParamValue::UInt(v.as_u64().unwrap())
                    }
                    serde_json::Value::Number(v) => ParamValue::Float(v.as_f64().unwrap()),
                    _ => bail!(
                        "Value given for '{}' in test '{}' contains a non-scalar list item: '{:?}'",
                        name,
                        owner_name,
                        item
                    ),
                };
                result.push(item);
            }
            Ok(ParamValue::List(result))
        }
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
        ParamType::Frequency => parse_f64(value).map(ParamValue::Frequency).ok_or_else(|| {
            crate::Error::new(&format!(
                "Value given for '{}' in test '{}' is not a number: '{:?}'",
                name, owner_name, value
            ))
        }),
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

    fn model_with_two_continuity_tests() -> Model {
        let mut model = Model::new(SupportedTester::V93KSMT7);
        model.create_flow("main").unwrap();
        model.select_flow("main").unwrap();
        for (id, name) in [(1, "first"), (2, "second")] {
            model
                .add_test_from_template(
                    id,
                    name.to_string(),
                    &SupportedTester::V93KSMT7,
                    "continuity",
                    Some("dc_tml"),
                )
                .unwrap();
        }
        model
    }

    #[test]
    fn tests_of_a_template_share_its_definition_until_one_changes_it() {
        let mut model = model_with_two_continuity_tests();
        assert!(Arc::ptr_eq(
            &model.tests[&1].definition,
            &model.tests[&2].definition
        ));
        assert!(!model.tests[&1].definition.params.is_empty());

        // Setting values leaves the definition shared
        let name: String = model.tests[&1]
            .definition
            .params
            .keys()
            .next()
            .unwrap()
            .clone();
        model
            .tests
            .get_mut(&1)
            .unwrap()
            .set(&name, None, false)
            .unwrap();
        assert!(Arc::ptr_eq(
            &model.tests[&1].definition,
            &model.tests[&2].definition
        ));

        // Changing one test's definition copies it, the other test and the template keep theirs
        let params = model.tests[&2].definition.params.len();
        model
            .tests
            .get_mut(&1)
            .unwrap()
            .add_param(
                "extraParam",
                ParamType::Int,
                None,
                Some(vec!["extra"]),
                None,
            )
            .unwrap();
        assert!(!Arc::ptr_eq(
            &model.tests[&1].definition,
            &model.tests[&2].definition
        ));
        assert!(model.tests[&1].has_param("extraParam"));
        assert!(!model.tests[&2].has_param("extraParam"));
        assert_eq!(model.tests[&2].definition.params.len(), params);
        model
            .add_test_from_template(
                3,
                "third".to_string(),
                &SupportedTester::V93KSMT7,
                "continuity",
                Some("dc_tml"),
            )
            .unwrap();
        assert!(!model.tests[&3].has_param("extraParam"));
        assert!(Arc::ptr_eq(
            &model.tests[&2].definition,
            &model.tests[&3].definition
        ));
    }

    #[test]
    fn tests_serialize_their_fields_as_before() {
        let model = model_with_two_continuity_tests();
        let json = serde_json::to_value(&model.tests[&1]).unwrap();
        let keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.as_str())
            .collect();
        let mut expected = vec![
            "id",
            "name",
            "tname",
            "indirect",
            "params",
            "values",
            "default_values",
            "aliases",
            "constraints",
            "collection_defs",
            "collections",
            "tester",
            "class_name",
            "template_library",
            "template_name",
            "test_id",
            "sub_tests",
            "number",
            "lo_limit",
            "hi_limit",
            "_private",
        ];
        // serde_json's map is sorted unless its preserve_order feature is enabled
        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        if keys == sorted_keys {
            expected.sort();
        }
        assert_eq!(keys, expected);
        let text = serde_json::to_string(&model.tests[&1]).unwrap();
        let positions: Vec<usize> = [
            "\"id\"",
            "\"params\"",
            "\"values\"",
            "\"default_values\"",
            "\"collection_defs\"",
            "\"_private\"",
        ]
        .iter()
        .map(|key| text.find(key).unwrap())
        .collect();
        let mut sorted_positions = positions.clone();
        sorted_positions.sort();
        assert_eq!(positions, sorted_positions);
        assert_eq!(
            json["params"],
            serde_json::to_value(&model.tests[&1].definition.params).unwrap()
        );
    }

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
        let tsen = test.definition.collection_defs.get("tsen").unwrap();
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

        assert!(err.to_string().contains("does not match the required type"));
    }

    #[test]
    fn imports_v93k_param_kind_aliases() {
        let template: TestTemplate = serde_json::from_str(
            r#"{
                "class_name": "com.example.Test",
                "parameters": {
                    "registerSetup.pinList": {
                        "kind": "PinString",
                        "value": "BP_BTDO"
                    },
                    "trigger.delay": {
                        "kind": "SpecValue",
                        "value": "0[ms]"
                    },
                    "search.specVar": {
                        "kind": "SpecVariable",
                        "value": "specs.Nominal"
                    },
                    "search.focusedPins": {
                        "kind": "ContextPins",
                        "value": "Tj_PWM"
                    },
                    "mode.select": {
                        "kind": "OptionList",
                        "value": "pass/fail"
                    },
                    "bitNumbers": {
                        "kind": "long",
                        "value": "0"
                    }
                }
            }"#,
        )
        .unwrap();

        let mut test = Test::new("example", 1, SupportedTester::V93KSMT8);
        test.import_test_template(&template).unwrap();

        assert_eq!(
            test.definition.params.get("registerSetup.pinList"),
            Some(&ParamType::String)
        );
        assert_eq!(
            test.definition.params.get("trigger.delay"),
            Some(&ParamType::String)
        );
        assert_eq!(
            test.definition.params.get("search.specVar"),
            Some(&ParamType::String)
        );
        assert_eq!(
            test.definition.params.get("search.focusedPins"),
            Some(&ParamType::String)
        );
        assert_eq!(
            test.definition.params.get("mode.select"),
            Some(&ParamType::String)
        );
        assert_eq!(
            test.definition.params.get("bitNumbers"),
            Some(&ParamType::Int)
        );

        assert_eq!(
            test.values.get("registerSetup.pinList"),
            Some(&ParamValue::String("BP_BTDO".to_string()))
        );
        assert_eq!(
            test.values.get("trigger.delay"),
            Some(&ParamValue::String("0[ms]".to_string()))
        );
        assert_eq!(
            test.values.get("mode.select"),
            Some(&ParamValue::String("pass/fail".to_string()))
        );
        assert_eq!(test.values.get("bitNumbers"), Some(&ParamValue::Int(0)));
    }

    #[test]
    fn imports_class_and_list_values_and_constraints() {
        let template: TestTemplate = serde_json::from_str(
            r#"{
                "parameters": {
                    "decodingType": {
                        "kind": "class",
                        "value": "SMU14",
                        "accepted_values": ["SMU14", "SMU16"]
                    },
                    "pins": {"kind": "list_strings", "value": "[]"},
                    "categories": {
                        "kind": "list_classes",
                        "value": ["DEFECT_SCREEN"]
                    }
                }
            }"#,
        )
        .unwrap();
        let mut test = Test::new("example", 1, SupportedTester::V93KSMT8);
        test.import_test_template(&template).unwrap();
        assert_eq!(
            test.definition.params.get("decodingType"),
            Some(&ParamType::Class)
        );
        assert_eq!(
            test.values.get("decodingType"),
            Some(&ParamValue::Class("SMU14".to_string()))
        );
        assert_eq!(test.values.get("pins"), Some(&ParamValue::List(Vec::new())));
        assert_eq!(
            test.values.get("categories"),
            Some(&ParamValue::List(vec![ParamValue::Class(
                "DEFECT_SCREEN".to_string()
            )]))
        );
        test.set(
            "decodingType",
            Some(ParamValue::String("SMU16".to_string())),
            false,
        )
        .unwrap();
        assert_eq!(
            test.values.get("decodingType"),
            Some(&ParamValue::Class("SMU16".to_string()))
        );
    }

    #[test]
    fn coerces_numeric_values_to_declared_types() {
        assert_eq!(
            coerce_param_value(&ParamType::Float, ParamValue::Int(2)).unwrap(),
            ParamValue::Float(2.0)
        );
        assert_eq!(
            coerce_param_value(&ParamType::Current, ParamValue::UInt(3)).unwrap(),
            ParamValue::Current(3.0)
        );
        assert_eq!(
            coerce_param_value(&ParamType::Voltage, ParamValue::Float(1.5)).unwrap(),
            ParamValue::Voltage(1.5)
        );
        assert_eq!(
            coerce_param_value(
                &ParamType::ListStrings,
                ParamValue::List(vec![ParamValue::Int(3), ParamValue::Bool(true)])
            )
            .unwrap(),
            ParamValue::List(vec![
                ParamValue::String("3".to_string()),
                ParamValue::String("true".to_string())
            ])
        );
    }

    #[test]
    fn rejects_nested_parameter_lists_during_coercion() {
        let nested = ParamValue::List(vec![ParamValue::List(vec![ParamValue::String(
            "nested".to_string(),
        )])]);
        assert!(coerce_param_value(&ParamType::ListStrings, nested.clone())
            .unwrap_err()
            .to_string()
            .contains("Nested parameter lists"));
        assert!(coerce_param_value(&ParamType::ListClasses, nested)
            .unwrap_err()
            .to_string()
            .contains("Nested parameter lists"));
    }
}
