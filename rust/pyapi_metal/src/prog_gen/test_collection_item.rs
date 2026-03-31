use super::src_caller_meta;
use super::to_param_value;
use origen_metal::prog_gen::{flow_api, ParamValue};
use origen_metal::Result;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct TestCollectionItem {
    pub id: usize,
}

impl TestCollectionItem {
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    pub fn set_attr_value(
        &self,
        name: &str,
        value: Option<ParamValue>,
        allow_missing: bool,
    ) -> Result<()> {
        flow_api::set_test_attr(self.id, name, value, allow_missing, src_caller_meta())?;
        Ok(())
    }
}

#[pymethods]
impl TestCollectionItem {
    #[pyo3(signature=(name, value=None, allow_missing=false))]
    pub fn set_attr(&self, name: &str, value: Option<&PyAny>, allow_missing: bool) -> Result<()> {
        let value = match value {
            Some(v) => to_param_value(v)?,
            None => None,
        };
        self.set_attr_value(name, value, allow_missing)?;
        Ok(())
    }

    #[pyo3(signature=(collection_name, instance_id, allow_missing=false))]
    pub fn add_collection_item(
        &self,
        collection_name: &str,
        instance_id: &str,
        allow_missing: bool,
    ) -> Result<TestCollectionItem> {
        let id = flow_api::define_test_collection_item(
            self.id,
            collection_name,
            instance_id,
            allow_missing,
            src_caller_meta(),
        )?;
        Ok(TestCollectionItem::new(id))
    }

    fn __setattr__(&mut self, name: &str, value: &PyAny) -> PyResult<()> {
        self.set_attr_value(name, to_param_value(value)?, false)?;
        Ok(())
    }
}
