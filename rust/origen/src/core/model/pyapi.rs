//use pyo3::prelude::*;
//use pyo3::wrap_pyfunction;
//
//use super::Model;
//
//#[pymodule]
///// Implements the module _origen.model in Python
//pub fn model(_py: Python, m: &PyModule) -> PyResult<()> {
////    m.add_wrapped(wrap_pyfunction!(number_of_regs))?;
//    m.add_class::<Model>()?;
//
//    Ok(())
//}

//#[pyfunction]
//fn new(name: String) -> PyResult<PyObject> {
//    Ok(Box::into_raw(Box::new(Model::new(name))))
//}


//#[pyfunction]
//fn number_of_regs(ptr: *const Model) -> PyResult<u32> {
//    //let model = unsafe {
//    //    assert!(!ptr.is_null());
//    //    &*ptr
//    //};
//    //Ok(format!("{} - {}", name, offset))
//    //Ok(2)
//    Ok(ptr.number_of_regs())
//}
