#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Variable {
    pub name: String,
    pub variable_type: VariableType,
    pub operation: VariableOperation,
}

impl Variable {
    pub fn new(
        name: String,
        variable_type: VariableType,
        operation: VariableOperation,
    ) -> Variable {
        Variable {
            name: name,
            variable_type: variable_type,
            operation: operation,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VariableType {
    Flag,
    Enable,
    Job,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VariableOperation {
    Reference,
    Set,
}
