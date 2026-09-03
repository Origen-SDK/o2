use num_bigint::BigUint;
use origen::core::model::registers::register::FieldContainer as OrigenField;
use origen::core::model::registers::register::FieldEnum as OrigenFieldEnum;
use origen::core::model::registers::register::ResetVal as OrigenResetVal;
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
    #[pyo3(signature=(
        name,
        description,
        offset,
        width,
        access,
        resets,
        enums,
        filename=None,
        lineno=None,
    ))]
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

impl Field {
    pub fn to_origen_field(&self) -> OrigenField {
        OrigenField {
            name: self.name.clone(),
            description: self.description.clone(),
            offset: self.offset,
            width: self.width,
            access: self.access.clone(),
            resets: {
                if let Some(_resets) = &self.resets {
                    Some(
                        _resets
                            .iter()
                            .map(|res| res.to_origen_reset_val())
                            .collect(),
                    )
                } else {
                    None
                }
            },
            enums: self
                .enums
                .iter()
                .map(|e| e.to_origen_field_enum())
                .collect(),
            filename: self.filename.clone(),
            lineno: self.lineno,
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
        value: BigUint,
    ) -> Self {
        FieldEnum {
            name: name,
            description: description,
            //usage: usage,
            value: value,
        }
    }
}

impl FieldEnum {
    pub fn to_origen_field_enum(&self) -> OrigenFieldEnum {
        OrigenFieldEnum {
            name: self.name.clone(),
            description: self.name.clone(),
            value: self.value.clone(),
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
    fn new(name: String, value: BigUint, mask: Option<BigUint>) -> Self {
        ResetVal {
            name: name,
            value: value,
            mask: mask,
        }
    }
}

impl ResetVal {
    pub fn to_origen_reset_val(&self) -> OrigenResetVal {
        OrigenResetVal {
            name: self.name.clone(),
            value: self.value.clone(),
            mask: self.mask.clone(),
        }
    }
}
