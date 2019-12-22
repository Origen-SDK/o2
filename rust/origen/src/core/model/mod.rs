pub mod pins;
pub mod registers;
//use crate::error::Error;
//use crate::Result;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Model {
    pub name: String,
    //pub parent_id: usize,
    /// Returns the path to this model for displaying to a user, e.g. in error messages.
    pub display_path: String,
    /// All children of this block/model, which are themselves models
    pub sub_blocks: HashMap<String, usize>,
    /// All registers owned by this model are arranged within memory maps
    pub memory_maps: HashMap<String, usize>,
    // Pins
    // Levels
    // Timing
    // Specs
}

impl Model {
    //pub fn new(name: String, parent_id: usize) -> Model {
    pub fn new(name: String) -> Model {
        //let mut p = "dut".to_string();
        //if parent_path != "" {
        //    p = format!("{}.{}", p, parent_path);
        //}
        //if id != "" {
        //    p = format!("{}.{}", p, id);
        //}
        Model {
            name: name,
            //parent_path: parent_path,
            display_path: "TBD".to_string(),
            sub_blocks: HashMap::new(),
            memory_maps: HashMap::new(),
        }
    }

    /// Returns the hierarchical name of the model and the offset for console displays
    pub fn console_header(&self) -> (String, usize) {
        let l = format!("{}", self.display_path);
        let mut names: Vec<&str> = l.split(".").collect();
        names.pop();
        if names.is_empty() {
            (l + "\n", 1)
        } else {
            let s = names.join(".").chars().count() + 2;
            (l + "\n", s)
        }
    }

    //pub fn get_reg(
    //    &self,
    //    memory_map: Option<&str>,
    //    address_block: Option<&str>,
    //    id: &str,
    //) -> Result<&Register> {
    //    let map_id = memory_map.unwrap_or("default");
    //    let ab_id = address_block.unwrap_or("default");
    //    // TODO: bubble the errors here
    //    let map = match self.memory_maps.get(map_id) {
    //        Some(x) => x,
    //        None => {
    //            return Err(Error::new(&format!(
    //                "The block '{}' does not contain a memory-map called '{}'",
    //                self.display_path, map_id
    //            )))
    //        }
    //    };
    //    let ab = match map.address_blocks.get(ab_id) {
    //        Some(x) => x,
    //        None => {
    //            return Err(Error::new(&format!(
    //            "The block '{}' does not contain an address-block called '{}' in memory-map '{}'",
    //            self.display_path, ab_id, map_id
    //        )))
    //        }
    //    };
    //    match ab.registers.get(id) {
    //        Some(x) => return Ok(x),
    //        None => {
    //            return Err(Error::new(&format!(
    //            "The block '{}' does not contain a register called '{}' in address-block '{}.{}'",
    //            self.display_path, id, map_id, ab_id
    //        )))
    //        }
    //    };
    //}
}
