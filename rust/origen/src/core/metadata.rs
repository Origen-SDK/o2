use crate::Result;
use num_bigint::{BigInt, BigUint};
use std::convert::TryFrom;
use std::str::FromStr;
use toml::Value;

const DATA: &str = "data";
const CLASS: &str = "__origen_encoded_class__";

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Metadata {
    String(String),
    Usize(usize),
    BigInt(BigInt),
    BigUint(BigUint),
    Bool(bool),
    Float(f64),
    Vec(Vec<Self>),
    Serialized(Vec<u8>, Option<String>, Option<String>), // Data, Serializer, Optional Data Type/Class
                                                         // ... as needed
}

macro_rules! data {
    () => {{
        DATA.to_string()
    }};
}

macro_rules! class {
    () => {{
        CLASS.to_string()
    }};
}

impl Metadata {
    pub fn to_class_str(&self) -> &str {
        match self {
            Self::String(_) => "string",
            Self::BigInt(_) => "bigint",
            Self::BigUint(_) => "biguint",
            Self::Bool(_) => "bool",
            Self::Float(_) => "float",
            Self::Serialized(_, _, _) => "serialized",
            Self::Usize(_) => "usize",
            Self::Vec(_) => "vec",
        }
    }

    pub fn to_toml_value(&self) -> Result<Value> {
        let mut toml_map = toml::value::Table::new();
        match self {
            Self::String(s) => {
                toml_map.insert(data!(), Value::String(s.to_string()));
            }
            Self::BigInt(b) => {
                toml_map.insert(data!(), Value::String(b.to_str_radix(10)));
            }
            Self::BigUint(b) => {
                toml_map.insert(data!(), Value::String(b.to_str_radix(10)));
            }
            Self::Bool(b) => {
                toml_map.insert(data!(), Value::Boolean(*b));
            }
            Self::Float(f) => {
                toml_map.insert(data!(), Value::Float(*f));
            }
            Self::Serialized(data, serializer, class) => {
                toml_map.insert(
                    data!(),
                    Value::Array(
                        data.iter()
                            .map(|byte| Value::Integer(*byte as i64))
                            .collect::<Vec<Value>>(),
                    ),
                );
                if let Some(s) = serializer {
                    toml_map.insert("serializer".to_string(), Value::String(s.to_string()));
                }
                if let Some(c) = class {
                    toml_map.insert("class".to_string(), Value::String(c.to_string()));
                }
            }
            Self::Vec(data) => {
                let mut values_vec: Vec<Value> = vec![];
                for d in data.iter() {
                    values_vec.push(d.to_toml_value()?);
                }
                toml_map.insert("vec".to_string(), Value::Array(values_vec));
            }
            _ => bail!("Cannot convert metadata {:?} to a toml map value", self),
        }
        toml_map.insert(class!(), Value::String(self.to_class_str().to_string()));
        Ok(Value::Table(toml_map))
    }

    pub fn as_string(&self) -> Result<String> {
        match self {
            Self::String(s) => Ok(s.clone()),
            _ => bail!(
                "Requested Metadata as string, but it is of type {}",
                self.to_class_str()
            ),
        }
    }

    pub fn as_bigint(&self) -> Result<BigInt> {
        match self {
            Self::BigInt(b) => Ok(b.clone()),
            _ => bail!(
                "Requested Metadata as bigint, but it is of type {}",
                self.to_class_str()
            ),
        }
    }

    pub fn as_biguint(&self) -> Result<BigUint> {
        match self {
            Self::BigUint(b) => Ok(b.clone()),
            _ => bail!(
                "Requested Metadata as biguint, but it is of type {}",
                self.to_class_str()
            ),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            _ => bail!(
                "Requested Metadata as bool, but it is of type {}",
                self.to_class_str()
            ),
        }
    }

    pub fn as_float(&self) -> Result<f64> {
        match self {
            Self::Float(f) => Ok(*f),
            _ => bail!(
                "Requested Metadata as float, but it is of type {}",
                self.to_class_str()
            ),
        }
    }

    pub fn as_vec(&self) -> Result<Vec<Self>> {
        match self {
            Self::Vec(v) => Ok(v.clone()),
            _ => bail!(
                "Requested Metadata as float, but it is of type {}",
                self.to_class_str()
            ),
        }
    }
}

impl TryFrom<&Value> for Metadata {
    type Error = crate::Error;

    fn try_from(value: &Value) -> crate::Result<Self> {
        match value {
            Value::String(s) => Ok(Self::String(s.to_string())),
            Value::Boolean(b) => Ok(Self::Bool(*b)),
            Value::Integer(i) => Ok(Self::BigInt(BigInt::from(*i))),
            Value::Table(a) => {
                if let Some(encoded_class_value) = a.get(CLASS) {
                    if let Some(encoded_class) = encoded_class_value.as_str() {
                        if encoded_class == "vec" {
                            if let Some(data_val) = a.get("vec") {
                                if let Some(data) = data_val.as_array() {
                                    let mut elements: Vec<Metadata> = vec![];
                                    for el in data.iter() {
                                        elements.push(Self::try_from(el)?);
                                    }
                                    return Ok(Self::Vec(elements));
                                }
                            }
                        }

                        if let Some(data_val) = a.get("data") {
                            if encoded_class == "biguint" {
                                if let Some(data) = data_val.as_str() {
                                    return Ok(Self::BigUint(BigUint::from_str(data)?));
                                }
                            } else if encoded_class == "bigint" {
                                if let Some(data) = data_val.as_str() {
                                    return Ok(Self::BigInt(BigInt::from_str(data)?));
                                }
                            } else if encoded_class == "bool" {
                                if let Some(data) = data_val.as_bool() {
                                    return Ok(Self::Bool(data));
                                }
                            } else if encoded_class == "float" {
                                if let Some(data) = data_val.as_float() {
                                    return Ok(Self::Float(data));
                                }
                            } else if encoded_class == "string" {
                                if let Some(data) = data_val.as_str() {
                                    return Ok(Self::String(data.to_string()));
                                }
                            } else if encoded_class == "serialized" {
                                if let Some(bytes) = data_val.as_array() {
                                    let mut retn: Vec<u8> = vec![];
                                    for byte in bytes.iter() {
                                        match byte {
                                            Value::Integer(b) => {
                                                retn.push(*b as u8);
                                            }
                                            _ => bail!("Data was not serialized!"),
                                        }
                                    }
                                    return Ok(Self::Serialized(
                                        retn,
                                        if let Some(serializer) = a.get("serializer") {
                                            if let Some(s) = serializer.as_str() {
                                                Some(s.to_string())
                                            } else {
                                                bail!("serializer was not of type String");
                                            }
                                        } else {
                                            None
                                        },
                                        if let Some(class) = a.get("class") {
                                            if let Some(c) = class.as_str() {
                                                Some(c.to_string())
                                            } else {
                                                bail!("class was not of type String");
                                            }
                                        } else {
                                            None
                                        },
                                    ));
                                }
                            } else {
                                bail!("Cannot decode from type {}", encoded_class);
                            }
                        }
                    }
                }
                // Generic Hashmap
                // Probably add support for this add some point
                bail!("Metadata conversion from generic Value::Table is not implemented yet")
            }
            _ => bail!("Cannot convert toml::Value {} to origen::Metadata", value),
        }
    }
}
