use num_bigint::BigUint;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub offset: usize,
    pub width: usize,
    pub access: Option<String>,
    pub resets: Option<Vec<ResetVal>>,
    pub enums: Vec<FieldEnum>,
    pub filename: Option<String>,
    pub lineno: Option<usize>,
}

#[pymethods]
impl Field {
    #[new]
    fn new(
        name: String,
        description: Option<String>,
        offset: usize,
        width: usize,
        access: Option<String>,
        resets: Option<Vec<PyRef<ResetVal>>>,
        enums: Vec<PyRef<FieldEnum>>,
        filename: Option<String>,
        lineno: Option<usize>,
    ) -> Self {
        let mut enum_objs: Vec<FieldEnum> = Vec::new();
        for e in &enums {
            enum_objs.push(FieldEnum {
                name: e.name.to_string(),
                description: e.description.clone(),
                //usage: e.usage.clone(),
                value: e.value.clone(),
            });
        }
        let rsts: Option<Vec<ResetVal>>;
        if resets.is_some() {
            let mut _rsts: Vec<ResetVal> = Vec::new();
            for r in &resets.unwrap() {
                _rsts.push(ResetVal {
                    name: r.name.to_string(),
                    value: r.value.clone(),
                    mask: r.mask.clone(),
                });
            }
            rsts = Some(_rsts);
        } else {
            rsts = None;
        }
            Field {
                name: name,
                description: description,
                offset: offset,
                width: width,
                access: access,
                resets: rsts,
                enums: enum_objs,
                filename: filename,
                lineno: lineno,
            }
    }
}

#[pyclass]
#[derive(Debug)]
pub struct FieldEnum {
    pub name: String,
    pub description: String,
    //pub usage: String,
    pub value: BigUint,
}

#[pymethods]
impl FieldEnum {
    #[new]
    fn new(
        name: String,
        description: String,
        //usage: String,
        value: u128,
    ) -> Self {
            FieldEnum {
                name: name,
                description: description,
                //usage: usage,
                value: BigUint::from(value),
            }
    }
}

#[pyclass]
#[derive(Debug)]
pub struct ResetVal {
    pub name: String,
    pub value: BigUint,
    pub mask: Option<BigUint>,
}

#[pymethods]
impl ResetVal {
    #[new]
    fn new(name: String, value: u128, mask: Option<u128>) -> Self {
            ResetVal {
                name: name,
                value: BigUint::from(value),
                mask: match mask {
                    Some(m) => Some(BigUint::from(m)),
                    None => None
                },
            }
    }
}
