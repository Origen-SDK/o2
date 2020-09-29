use super::TestProgram;
use crate::Result;
use indexmap::IndexMap;

#[derive(Debug)]
pub enum ParamValue {
    String(String),
    Int(i128),
    UInt(u128),
    Number(f64),
    Current(f64),
    Voltage(f64),
    Time(f64),
    Frequency(f64),
    Bool(bool),
}

impl ParamValue {
    pub fn is_type(&self, kind: &ParamType) -> bool {
        match self {
            ParamValue::String(_) => kind == &ParamType::String,
            ParamValue::Int(_) => kind == &ParamType::Int,
            ParamValue::UInt(_) => kind == &ParamType::UInt,
            ParamValue::Number(_) => kind == &ParamType::Number,
            ParamValue::Current(_) => kind == &ParamType::Current,
            ParamValue::Voltage(_) => kind == &ParamType::Voltage,
            ParamValue::Time(_) => kind == &ParamType::Time,
            ParamValue::Frequency(_) => kind == &ParamType::Frequency,
            ParamValue::Bool(_) => kind == &ParamType::Bool,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParamType {
    String,
    Int,
    UInt,
    Number,
    Current,
    Voltage,
    Time,
    Frequency,
    Bool,
}

#[derive(Debug)]
pub enum Constraint {
    In(Vec<ParamValue>),
    GT(ParamValue),
    GTE(ParamValue),
    LT(ParamValue),
    LTE(ParamValue),
}

/// Represents an individual test being called/invoked by a test program flow, it associates
/// a test with additional parameterization that is relevant to a particular invocation of it
/// within a flow.
/// The invocation parameters can be very simple, e.g. only really to capture things like the
/// test name column in a Teradyne platform, or more elaborate as in an Advantest example where
/// it is used to model test suites.
#[derive(Debug)]
pub struct TestInvocation {
    /// The ID of the Test to be invoked, i.e. the test instance/test method
    test_id: usize,
    /// The parameters associated with this particular call from the flow, i.e. the test suite
    test_inv_id: usize,
}

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
    /// Optional parent definition
    pub parent_id: Option<usize>,
    /// Defines the names of parameters and their types. Child class can override the type of a parameter
    /// inherited from a parent by adding a parameter of the same name to their params map. Then can also
    /// add additional parameters via the same mechanism. It is not possible to delete a parameter inherited
    /// from a parent Test.
    pub params: IndexMap<String, ParamType>,
    pub values: IndexMap<String, ParamValue>,
    pub aliases: IndexMap<String, String>,
    pub constraints: IndexMap<String, Vec<Constraint>>,
}

impl Test {
    pub fn new(name: &str, id: usize) -> Test {
        Test {
            id: id,
            name: name.to_string(),
            indirect: false,
            parent_id: None,
            params: IndexMap::new(),
            values: IndexMap::new(),
            aliases: IndexMap::new(),
            constraints: IndexMap::new(),
        }
    }

    /// Set the value of the given parameter to the given value, returns an error if the
    /// parameter is not found or if its type does match the type of the given value
    pub fn set(&mut self, param_name: &str, value: ParamValue, prog: &TestProgram) -> Result<()> {
        let kind = self.get_type(param_name, prog)?;
        if value.is_type(kind) {
            self.values.insert(param_name.to_string(), value);
            Ok(())
        } else {
            error!("The type of the given value for '{}' in test '{}' does not match the required type: expected {:?}, given {:?}", param_name, &self.name, kind, value)
        }
    }

    /// Returns the type of the given parameter name (or alias), returns an error if the
    /// parameter is not found
    pub fn get_type<'a>(
        &'a self,
        param_name: &str,
        prog: &'a TestProgram,
    ) -> Result<&'a ParamType> {
        if let Some(p) = self.params.get(param_name) {
            Ok(p)
        } else if let Some(p) = self.parent(prog)? {
            Ok(p.get_type(param_name, prog)?)
        } else {
            error!(
                "Test '{}' has no parameter named '{}'",
                &self.name, param_name
            )
        }
    }

    /// Returns the parent of this test or None
    pub fn parent<'a>(&self, prog: &'a TestProgram) -> Result<Option<&'a Test>> {
        match self.parent_id {
            Some(p) => return Ok(Some(prog.get_test(p)?)),
            None => return Ok(None),
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
    pub fn add_alias(&mut self, alias: &str, param_name: &str, prog: &TestProgram) -> Result<()> {
        if self.has_param(param_name, prog)? {
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

    /// Returns the type of the given parameter name (or alias), returns an error if the
    /// parameter is not found
    pub fn has_param(&self, param_name: &str, prog: &TestProgram) -> Result<bool> {
        if self.params.contains_key(param_name) {
            Ok(true)
        } else if let Some(p) = self.parent(prog)? {
            p.has_param(param_name, prog)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug)]
pub struct TestCollection {
    pub name: String,
    pub test_ids: Vec<usize>,
}

impl TestCollection {
    pub fn new(name: &str) -> TestCollection {
        TestCollection {
            name: name.to_string(),
            test_ids: vec![],
        }
    }
}
