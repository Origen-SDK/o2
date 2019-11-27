pub mod pyapi;
pub mod registers;
pub mod pins;

use registers::Reg;
use std::collections::HashMap;
//use pyo3::prelude::*;

//#[pyclass]
#[derive(Debug)]
pub struct Model {
    name: String,
    // TODO: This should be richer, more like an IP-XACT memory map block
    registers: HashMap<String, Reg>,
}


////#[cfg(feature = "python")]
//#[pymethods]
//impl Model {
//    #[new]
//    fn new(obj: &PyRawObject, name: String) {
//        obj.init({
//            Model {
//                name: name,
//                registers: HashMap::new(),
//            }
//        });
//    }
//}

//impl Model {
//    fn add_reg(&mut self, name: String, offset: u32) {
//        let r = Reg{ name: name.clone(), offset: offset };
//        self.registers.insert(name, r);
//    }
//
//    fn number_of_regs(&self) -> u32 {
//        self.registers.len() as u32
//    }
//}
