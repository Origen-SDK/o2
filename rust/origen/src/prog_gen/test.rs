use indexmap::IndexMap;

pub enum ParamType {
    //String,
//Integer,
}

/// Represents an individual test in a test program flow
pub struct Test {
    _name: String,
    _test_def_id: usize,
}

/// Defines the args of a test instance/method/suite, including their names, data type,
/// aliases and default values
pub struct Definition {
    /// Optional parent definition
    pub test_def_id: Option<usize>,
    /// Defines the names of parameters and their data type
    pub params: IndexMap<String, Option<ParamType>>,
    pub aliases: IndexMap<String, String>,
    pub defaults: IndexMap<String, Value>,
}

//pub struct Param {}

/// A value is a storage element for test parameter values, providing storage for
/// different data types with the one used defined by the mapping for the given
/// parameter
pub struct Value {
    _string: Option<String>,
}
