use crate::core::model::Model;
use crate::error::Error;
use crate::Result;

#[derive(Debug)]
pub struct DUT {
    pub id: String,
    pub model: Model,
}

impl DUT {
    pub fn new(id: String) -> DUT {
        DUT {
            id: id.clone(),
            model: Model::new("".to_string(), "".to_string()),
        }
    }

    /// Get a mutable reference to the model at the given path
    /// Note that the path is relative to the DUT, i.e. it should not include 'dut.'
    pub fn get_mut_model(&mut self, path: &str) -> Result<&mut Model> {
        if path == "" {
            return Ok(&mut self.model);
        } else {
            let mut m = &mut self.model;
            for field in path.split(".") {
                m = match m.sub_blocks.get_mut(field) {
                    Some(x) => x,
                    None => {
                        return Err(Error::new(&format!(
                            "The block '{}' does not contain a sub-block called '{}'",
                            m.display_path, field
                        )))
                    }
                }
            }
            return Ok(m);
        }
    }

    /// Get a read-only reference to the model at the given path, use get_mut_model if
    /// you need to modify the returned model
    /// Note that the path is relative to the DUT, i.e. it should not include 'dut.'
    pub fn get_model(&self, path: &str) -> Result<&Model> {
        if path == "" {
            return Ok(&self.model);
        } else {
            let mut m = &self.model;
            for field in path.split(".") {
                m = match m.sub_blocks.get(field) {
                    Some(x) => x,
                    None => {
                        return Err(Error::new(&format!(
                            "The block '{}' does not contain a sub-block called '{}'",
                            m.display_path, field
                        )))
                    }
                }
            }
            return Ok(m);
        }
    }

    /// Add a new sub-block model to the existing model at the given hierarchical path
    pub fn create_sub_block(&mut self, path: &str, id: &str) -> Result<()> {
        //println!("create_sub_block called with {}, {}", path, id);
        let model = self.get_mut_model(path)?;
        let child = Model::new(id.to_string(), path.to_string());

        if model.sub_blocks.contains_key(id) {
            let p;
            if path == "" {
                p = "dut";
            } else {
                p = "dut.";
            }
            return Err(Error::new(&format!(
                "The block '{}{}' already contains a sub-block called '{}'",
                p, path, id
            )));
        } else {
            model.sub_blocks.insert(id.to_string(), child);
            return Ok(());
        }
    }
}
