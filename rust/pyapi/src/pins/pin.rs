use super::super::dut::PyDUT;
use origen::DUT;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PyTuple};
#[pyclass]
pub struct Pin {
    pub name: String,
    pub model_id: usize,
}

// #[macro_export]
// macro_rules! pypin {
//     ($py:expr, $name:expr, $model_id:expr) => {
//         Py::new(
//             $py,
//             crate::pins::pin::Pin {
//                 name: String::from($name),
//                 path: String::from(""),
//                 model_id: model_id,
//             },
//         )
//         .unwrap()
//     };
// }

#[pymethods]
impl Pin {
    fn add_metadata(&self, id_str: &str, obj: &PyAny) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let pin = dut._get_mut_pin(self.model_id, &self.name)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let locals = [("origen", py.import("origen")?)].into_py_dict(py);
        let dut = py
            .eval("origen.dut.db", None, Some(&locals))
            .unwrap()
            .downcast_mut::<PyDUT>()?;
        let idx = dut.push_metadata(obj);

        // Store the index of this object, returning an error if the
        pin.add_metadata_id(id_str, idx)?;
        Ok(())
    }

    fn set_metadata(&self, id_str: &str, obj: &PyAny) -> PyResult<bool> {
        let mut dut = DUT.lock().unwrap();
        let pin = dut._get_mut_pin(self.model_id, &self.name)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let locals = [("origen", py.import("origen")?)].into_py_dict(py);
        let dut = py
            .eval("origen.dut.db", None, Some(&locals))
            .unwrap()
            .downcast_mut::<PyDUT>()?;
        match pin.get_metadata_id(id_str) {
            Some(idx) => {
                println!("override!");
                dut.override_metadata_at(idx, obj)?;
                Ok(true)
            }
            None => {
                println!("adding!");
                let idx = dut.push_metadata(obj);
                pin.add_metadata_id(id_str, idx)?;
                Ok(false)
            }
        }
    }

    fn get_metadata(&self, id_str: &str) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let pin = dut._get_pin(self.model_id, &self.name)?;
        let gil = Python::acquire_gil();
        let py = gil.python();
        match pin.metadata.get(id_str) {
            Some(idx) => {
                let locals = [("origen", py.import("origen")?)].into_py_dict(py);
                let obj = py
                    .eval(
                        &format!("origen.dut.db.get_metadata({})", idx),
                        None,
                        Some(&locals),
                    )
                    .unwrap()
                    .to_object(py);
                Ok(obj)
            }
            None => Ok(py.None()),
        }
    }

    #[getter]
    fn get_added_metadata(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let pin = dut._get_pin(self.model_id, &self.name)?;
        Ok(pin.metadata.iter().map(|(k, _)| k.clone()).collect())
    }

    // Even though we're storing the name in this instance, we're going to go back to the core anyway.
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let p = dut._get_pin(self.model_id, &self.name)?;
        Ok(p.name.clone())
    }

    #[getter]
    fn get_data(&self) -> PyResult<u8> {
        let dut = DUT.lock().unwrap();
        let pin = dut._get_pin(self.model_id, &self.name)?;
        Ok(pin.data)
    }

    #[getter]
    fn get_action(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let pin = dut._get_pin(self.model_id, &self.name)?;
        Ok(String::from(pin.action.as_str()))
    }

    #[getter]
    fn get_aliases(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let pin = dut._get_pin(self.model_id, &self.name)?;
        Ok(pin.aliases.clone())
    }

    #[getter]
    fn get_reset_data(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let pin = dut._get_pin(self.model_id, &self.name)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        match pin.reset_data {
            Some(d) => Ok(d.to_object(py)),
            None => Ok(py.None()),
        }
    }

    #[getter]
    fn get_reset_action(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let pin = dut._get_pin(self.model_id, &self.name)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        match pin.reset_action {
            Some(a) => Ok(String::from(a.as_char().to_string()).to_object(py)),
            None => Ok(py.None()),
        }
    }

    #[getter]
    fn get_groups(&self) -> PyResult<std::collections::HashMap<String, usize>> {
        let dut = DUT.lock().unwrap();
        let pin = dut._get_pin(self.model_id, &self.name)?;
        Ok(pin.groups.clone())
    }
}
