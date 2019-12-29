pub mod pins;
pub mod registers;
use crate::error::Error;
use crate::Dut;
use crate::Result;
use std::sync::MutexGuard;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Model {
    pub id: usize,
    pub name: String,
    /// The only one without a parent is the top-level DUT model
    pub parent_id: Option<usize>,
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
    pub fn new(id: usize, name: String, parent_id: Option<usize>) -> Model {
        Model {
            id: id,
            name: name,
            parent_id: parent_id,
            sub_blocks: HashMap::new(),
            memory_maps: HashMap::new(),
        }
    }

    /// Returns the hierarchical name of the model and the offset for console displays
    pub fn console_header(&self, dut: &MutexGuard<Dut>) -> (String, usize) {
        let l = format!("{}", self.display_path(dut));
        let mut names: Vec<&str> = l.split(".").collect();
        names.pop();
        if names.is_empty() {
            (l + "\n", 1)
        } else {
            let s = names.join(".").chars().count() + 2;
            (l + "\n", s)
        }
    }

    /// Get the ID for the given memory map name, throw an error if it doesn't exist
    pub fn get_memory_map_id(&self, name: &str) -> Result<usize> {
        match self.memory_maps.get(name) {
            Some(x) => Ok(*x),
            None => {
                return Err(Error::new(&format!(
                    "The block '{}' does not have a memory map named '{}'",
                    self.name, name
                )))
            }
        }
    }

    /// Returns the path to this model for displaying to a user, e.g. in error messages.
    pub fn display_path(&self, dut: &MutexGuard<Dut>) -> String {
        match self.parent_id {
            Some(p) => {
                let parent = dut.get_model(p).unwrap();
                return format!("{}.{}", parent.display_path(dut), self.name);
            }
            None => return format!("{}", self.name),
        }
    }
}
