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
            model: Model::new(id, "".to_string()),
        }
    }

    pub fn get_mut_model(&mut self, path: &str) -> Option<&mut Model> {
        if path == "" {
            return Some(&mut self.model);
        } else {
            return path.split(".")
                .try_fold(&mut self.model, |m, sb| m.sub_blocks.get_mut(sb))
        }
    }

    pub fn get_model(&self, path: &str) -> Option<&Model> {
        path.split(".")
            .try_fold(&self.model, |m, sb| m.sub_blocks.get(sb))
    }

    /// Add a new sub-block model to the existing model at the given hierarchical path
    pub fn create_sub_block(&mut self, path: &str, id: &str) -> Result<()> {
        //println!("create_sub_block called with {}, {}", path, id);
        let model = self
            .get_mut_model(path)
            .expect(&format!("A sub-block was missing in the path '{}'", path));
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
