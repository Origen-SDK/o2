use num_bigint::BigUint;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub description: String,
    pub offset: usize,
    pub width: usize,
    pub access: String,
    pub reset: BigUint,
    pub enums: Vec<FieldEnum>,
}

#[pymethods]
impl Field {
    #[new]
    fn new(
        obj: &PyRawObject,
        name: String,
        description: String,
        offset: usize,
        width: usize,
        access: String,
        reset: BigUint,
        enums: Vec<&FieldEnum>,
    ) {
        let mut enum_objs: Vec<FieldEnum> = Vec::new();
        for e in &enums {
            enum_objs.push(FieldEnum {
                name: e.name.to_string(),
                description: e.description.clone(),
                //usage: e.usage.clone(),
                value: e.value.clone(),
            });
        }
        obj.init({
            Field {
                name: name,
                description: description,
                offset: offset,
                width: width,
                access: access,
                reset: reset,
                enums: enum_objs,
            }
        });
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
        obj: &PyRawObject,
        name: String,
        description: String,
        //usage: String,
        value: BigUint,
    ) {
        obj.init({
            FieldEnum {
                name: name,
                description: description,
                //usage: usage,
                value: value,
            }
        });
    }
}
