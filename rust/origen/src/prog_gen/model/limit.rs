use crate::prog_gen::ParamValue;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Limit {
    pub kind: LimitType,
    pub value: ParamValue,
    pub unit: Option<String>,
}

impl Limit {
    pub fn unit_str(&self) -> &str {
        match &self.unit {
            Some(x) => x,
            None => "",
        }
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum LimitType {
    EQ,
    GT,
    GTE,
    LT,
    LTE,
}
