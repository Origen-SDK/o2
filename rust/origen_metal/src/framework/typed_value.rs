use crate::Result;
use indexmap::IndexMap as IM;
use num_bigint::{BigInt, BigUint};
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use toml::Value;
use std::iter::FromIterator;
use num_traits::cast::ToPrimitive;
use std::collections::HashMap;

const DATA: &str = "data";
const CLASS: &str = "__origen_encoded_class__";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TypedValue {
    None,
    String(String),
    Usize(usize),
    BigInt(BigInt),
    BigUint(BigUint),
    Bool(bool),
    Float(f64),
    Vec(Vec<Self>),
    Map(Map),
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

impl TypedValue {
    pub fn into_optional<'a, T: TryFrom<&'a TypedValue, Error = crate::Error>>(
        tv: Option<&'a TypedValue>,
    ) -> Result<Option<T>> {
        Ok(match tv {
            Some(val) => Some(val.try_into()?),
            None => None,
        })
    }

    pub fn to_class_str(&self) -> &str {
        match self {
            // TEST_NEEDED for none
            Self::None => "none",
            Self::String(_) => "string",
            Self::BigInt(_) => "bigint",
            Self::BigUint(_) => "biguint",
            Self::Bool(_) => "bool",
            Self::Float(_) => "float",
            Self::Serialized(_, _, _) => "serialized",
            Self::Usize(_) => "usize",
            Self::Vec(_) => "vec",
            Self::Map(_) => "map",
        }
    }

    pub fn to_toml_value(&self) -> Result<Value> {
        let mut toml_map = toml::value::Table::new();
        match self {
            Self::None => {
                toml_map.insert(data!(), Value::String("none".to_string()));
            }
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
            // TEST_NEEDED add tests for usize
            Self::Usize(u) => {
                toml_map.insert(data!(), Value::String(BigUint::from(*u).to_str_radix(10)));
            }
            Self::Vec(data) => {
                let mut values_vec: Vec<Value> = vec![];
                for d in data.iter() {
                    values_vec.push(d.to_toml_value()?);
                }
                toml_map.insert("vec".to_string(), Value::Array(values_vec));
            }
            // TEST_NEEDED for map
            Self::Map(map) => {
                toml_map.insert("map".to_string(), Value::try_from(map)?);
            }
        }
        toml_map.insert(class!(), Value::String(self.to_class_str().to_string()));
        Ok(Value::Table(toml_map))
    }

    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }

    pub fn as_string(&self) -> Result<String> {
        match self {
            Self::String(s) => Ok(s.clone()),
            _ => bail!(&self.conversion_error_msg("string")),
        }
    }

    pub fn as_bigint(&self) -> Result<BigInt> {
        match self {
            Self::BigInt(b) => Ok(b.clone()),
            _ => bail!(&self.conversion_error_msg("bigint")),
        }
    }

    pub fn as_biguint(&self) -> Result<BigUint> {
        match self {
            Self::BigUint(b) => Ok(b.clone()),
            _ => bail!(&self.conversion_error_msg("biguint")),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            _ => bail!(&self.conversion_error_msg("bool")),
        }
    }

    pub fn as_float(&self) -> Result<f64> {
        match self {
            Self::Float(f) => Ok(*f),
            _ => bail!(&self.conversion_error_msg("float")),
        }
    }

    pub fn as_vec(&self) -> Result<Vec<Self>> {
        match self {
            Self::Vec(v) => Ok(v.clone()),
            _ => bail!(&self.conversion_error_msg("vector")),
        }
    }

    // TEST_NEEDED
    pub fn as_map(&self) -> Result<&Map> {
        match self {
            Self::Map(m) => Ok(&m),
            _ => bail!(&self.conversion_error_msg("map"))
        }
    }

    fn conversion_error_msg(&self, expected: &str) -> String {
        format!(
            "Requested TypedValue as '{}', but it is of type '{}'",
            expected,
            self.to_class_str()
        )
    }

    fn as_i32(&self) -> Result<i32> {
        let big_val = self.as_bigint()?;
        match big_val.to_i32() {
            Some(i) => Ok(i),
            None => bail!("Cannot represent value {} as i32", big_val)
        }
    }

    pub fn as_option(&self) -> Option<&Self> {
        match self {
            Self::None => None,
            _ => Some(&self)
        }
    }
}

impl<T> From<Option<T>> for TypedValue
where
    TypedValue: From<T>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Self::None,
        }
    }
}

impl<T> From<Vec<T>> for TypedValue
where
    TypedValue: From<T>,
{
    fn from(values: Vec<T>) -> Self {
        Self::Vec(values.into_iter().map(|v| v.into()).collect::<Vec<Self>>())
    }
}

impl<'a, T> From<std::slice::Iter<'a, T>> for TypedValue
where
    TypedValue: From<&'a T>,
{
    fn from(values: std::slice::Iter<'a, T>) -> Self {
        Self::Vec(values.into_iter().map(|v| v.into()).collect::<Vec<Self>>())
    }
}

impl From<&str> for TypedValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for TypedValue {
    fn from(value: String) -> Self {
        Self::String(value.to_string())
    }
}

impl From<&String> for TypedValue {
    fn from(value: &String) -> Self {
        Self::String(value.to_string())
    }
}

impl From<bool> for TypedValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<&bool> for TypedValue {
    fn from(value: &bool) -> Self {
        Self::Bool(*value)
    }
}

impl From<usize> for TypedValue {
    fn from(value: usize) -> Self {
        Self::Usize(value)
    }
}

impl From<u64> for TypedValue {
    fn from(value: u64) -> Self {
        Self::BigUint(BigUint::from(value))
    }
}

impl From<i32> for TypedValue {
    fn from(value: i32) -> Self {
        Self::BigInt(BigInt::from(value))
    }
}

impl <'a, V>From<&'a HashMap<String, V>> for TypedValue where
    TypedValue: From<&'a V>,
{
    fn from(map: &'a HashMap<String, V>) -> Self {
        let mut tvm = Map::new();
        for (k, v) in map {
            tvm.insert(k, v);
        }
        Self::Map(tvm)
    }
}

impl TryFrom<TypedValue> for String {
    type Error = crate::Error;

    fn try_from(value: TypedValue) -> std::result::Result<String, Self::Error> {
        value.as_string()
    }
}

impl TryFrom<&TypedValue> for String {
    type Error = crate::Error;

    fn try_from(value: &TypedValue) -> std::result::Result<String, Self::Error> {
        value.as_string()
    }
}

impl TryFrom<TypedValue> for bool {
    type Error = crate::Error;

    fn try_from(value: TypedValue) -> std::result::Result<bool, Self::Error> {
        value.as_bool()
    }
}

impl TryFrom<&TypedValue> for bool {
    type Error = crate::Error;

    fn try_from(value: &TypedValue) -> std::result::Result<bool, Self::Error> {
        value.as_bool()
    }
}

impl TryFrom<TypedValue> for i32 {
    type Error = crate::Error;

    fn try_from(value: TypedValue) -> std::result::Result<i32, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&TypedValue> for i32 {
    type Error = crate::Error;

    fn try_from(value: &TypedValue) -> std::result::Result<i32, Self::Error> {
        value.as_i32()
    }
}

impl TryFrom<TypedValue> for Vec<String> {
    type Error = crate::Error;

    fn try_from(value: TypedValue) -> std::result::Result<Vec<String>, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&TypedValue> for Vec<String> {
    type Error = crate::Error;

    fn try_from(value: &TypedValue) -> std::result::Result<Vec<String>, Self::Error> {
        let strs = value.as_vec()?;
        Ok(strs.iter().map(|s| s.as_string()).collect::<Result<Vec<String>>>()?)
    }
}

impl TryFrom<&Value> for TypedValue {
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
                                    let mut elements: Vec<TypedValue> = vec![];
                                    for el in data.iter() {
                                        elements.push(Self::try_from(el)?);
                                    }
                                    return Ok(Self::Vec(elements));
                                }
                            }
                            // TODO
                            // } else if encoded_class == "map" {
                            //     if let Some(data_val) = a.get("map") {
                            //         if let Some(data) = data_val.as_table() {
                            //             let mut elements: HashMap<Self, Self> = HashMap::new();
                            //             for (k, el) in data.iter() {
                            //                 elements.insert(k, Self::try_from(el))?;
                            //             }
                            //             return Ok(Self::Map(elements));
                            //         }
                            //     }
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
                            } else if encoded_class == "none" {
                                // TODO need a check here?
                                // if let Some(data) = data_val.is_none() {
                                return Ok(Self::None);
                                // }
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
                // TODO
                // Generic Hashmap
                // Probably add support for this add some point
                bail!("TypedValue conversion from generic Value::Table is not implemented yet")
            }
            _ => bail!(
                "Cannot convert toml::Value {} to origen_metal::TypedValue",
                value
            ),
        }
    }
}

/// Wrapper around a vector of typed values
#[derive(Debug, Clone)]
pub struct TypedValueVec {
    pub typed_values: Vec<TypedValue>,
}

impl Default for TypedValueVec {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedValueVec {
    pub fn new() -> Self {
        Self {
            typed_values: vec![],
        }
    }

    pub fn typed_values(&self) -> &Vec<TypedValue> {
        &self.typed_values
    }
}

impl FromIterator<TypedValue> for TypedValueVec {
    fn from_iter<I: IntoIterator<Item = TypedValue>>(iter: I) -> Self {
        Self {
            typed_values: {
                let mut v = vec![];
                for i in iter {
                    v.push(i);
                }
                v
            },
        }
    }
}

type Tvm = IM<String, TypedValue>;

/// Wrapper around an indexmap of typed values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Map {
    pub typed_values: IM<String, TypedValue>,
}

impl std::convert::AsRef<Map> for Map {
    fn as_ref(&self) -> &Map {
        &self
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    pub fn new() -> Self {
        Self {
            typed_values: IM::new(),
        }
    }

    pub fn typed_values(&self) -> &Tvm {
        &self.typed_values
    }

    pub fn insert(&mut self, key: &str, obj: impl Into<TypedValue>) -> Option<TypedValue> {
        self.typed_values.insert(key.to_string(), obj.into())
    }

    pub fn into_pairs(&self) -> Vec<(String, TypedValue)> {
        self.typed_values
            .iter()
            .map(|(n, tv)| (n.to_string(), tv.clone()))
            .collect()
    }

    pub fn get(&self, key: &str) -> Option<&TypedValue> {
        self.typed_values.get(key)
    }

    pub fn keys(&self) -> indexmap::map::Keys<String, TypedValue> {
        self.typed_values.keys()
    }

    pub fn len(&self) -> usize {
        self.typed_values.len()
    }

    pub fn iter(&self) -> indexmap::map::Iter<String, TypedValue> {
        self.typed_values.iter()
    }
}

impl From<&Self> for Map {
    fn from(map: &Self) -> Self {
        Self {
            typed_values: map.typed_values.to_owned(),
        }
    }
}

impl From<IM<String, TypedValue>> for Map {
    fn from(map: IM<String, TypedValue>) -> Self {
        Self { typed_values: map }
    }
}

impl From<&IM<String, TypedValue>> for Map {
    fn from(map: &IM<String, TypedValue>) -> Self {
        Self {
            typed_values: map.to_owned(),
        }
    }
}

impl TryFrom<Map> for toml::map::Map<String, Value> {
    type Error = crate::Error;

    fn try_from(map: Map) -> std::result::Result<Self, Self::Error> {
        let mut tmap = Self::new();
        for (k, v) in map.iter() {
            tmap.insert(k.to_owned(), v.to_toml_value()?);
        }
        Ok(tmap)
    }
}