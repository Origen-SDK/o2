use super::super::meta::py_like_apis::list_like_api::{ListLikeAPI, ListLikeIter};
use super::pin_collection::PinCollection;
use super::pin_actions::PinActions;
use super::super::pins::extract_pin_transaction;
use origen::DUT;
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PySlice, PyTuple};
use num_bigint::BigUint;
use origen::Transaction;

#[pyclass]
#[derive(Clone)]
pub struct PinGroup {
    pub name: String,
    pub model_id: usize,
}

#[pymethods]
impl PinGroup {
    // Even though we're storing the name in this instance, we're going to go back to the core anyway.
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.name.clone())
    }

    fn with_mask(&self, mask: usize) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        dut.set_pin_group_nonsticky_mask(self.model_id, &self.name, mask)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            Self {
                name: self.name.clone(),
                model_id: self.model_id,
            },
        )
        .unwrap())
    }

    #[getter]
    fn get_pin_names(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.pin_names(&dut)?)
    }

    #[getter]
    fn get_actions(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();

        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        let pin_actions = grp.get_actions(&dut)?;
        Ok(PinActions {actions: pin_actions}.into_py(py))
    }

    fn drive(slf: PyRef<Self>, data: BigUint) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.update(&dut, &Transaction::new_write(data, grp.len())?)?;
        Ok(slf.into())
    }

    fn verify(slf: PyRef<Self>, data: BigUint) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.update(&dut, &Transaction::new_verify(data, grp.len())?)?;
        Ok(slf.into())
    }

    fn capture(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.update(&dut, &Transaction::new_capture(grp.len())?)?;
        Ok(slf.into())
    }

    fn highz(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.update(&dut, &Transaction::new_highz(grp.len())?)?;
        Ok(slf.into())
    }

    #[setter]
    fn actions(slf: PyRef<Self>, actions: &PyAny) -> PyResult<()> {
        Self::set_actions(slf, actions, None)?;
        Ok(())
    }

    #[args(kwargs = "**")]
    fn set_actions(slf: PyRef<Self>, actions: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.update(&dut, &extract_pin_transaction(actions, kwargs)?)?;
        Ok(slf.into())
    }

    fn reset(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.reset(&dut)?;
        Ok(slf.into())
    }

    #[getter]
    fn get_width(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.len())
    }

    #[getter]
    fn get_reset_actions(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        let pin_actions = grp.get_reset_actions(&dut)?;
        Ok(PinActions {actions: pin_actions}.into_py(py))
    }

    #[getter]
    fn get_big_endian(&self) -> PyResult<bool> {
        let is_little_endian = self.get_little_endian()?;
        Ok(!is_little_endian)
    }

    #[getter]
    fn get_little_endian(&self) -> PyResult<bool> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.is_little_endian())
    }

    #[args(kwargs = "**")]
    fn cycle(slf: PyRef<Self>, kwargs: Option<&PyDict>) -> PyResult<Py<Self>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);
        locals.set_item("origen", py.import("origen")?)?;
        locals.set_item("kwargs", kwargs.to_object(py))?;

        py.eval(&format!("origen.tester.cycle(**(kwargs or {{}}))"), None, Some(&locals))?;
        Ok(slf.into())
    }

    fn repeat(slf: PyRef<Self>, count: usize) -> PyResult<Py<Self>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);
        locals.set_item("origen", py.import("origen")?)?;
        py.eval(&format!("origen.tester.repeat({})", count), None, Some(&locals))?;
        Ok(slf.into())
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinGroup {
    // Need to overwrite contains to account for aliasing
    fn __contains__(&self, item: &PyAny) -> PyResult<bool> {
        if let Ok(s) = item.extract::<String>() {
            let dut = DUT.lock().unwrap();
            Ok(dut.pin_group_contains(self.model_id, &self.name, &s)?)
        } else {
            Ok(false)
        }
    }
}

impl ListLikeAPI for PinGroup {
    fn item_ids(&self, dut: &std::sync::MutexGuard<origen::core::dut::Dut>) -> Vec<usize> {
        let grp = dut._get_pin_group(self.model_id, &self.name).unwrap();
        grp.pin_ids.clone()
    }

    // Grabs a single pin and puts it in an anonymous pin collection
    fn new_pyitem(&self, py: Python, idx: usize) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(Py::new(py, PinCollection::from_ids_unchecked(vec![grp.pin_ids[idx]], Some(grp.endianness)))?.to_object(py))
    }

    fn __iter__(&self) -> PyResult<ListLikeIter> {
        Ok(ListLikeIter {
            parent: Box::new((*self).clone()),
            i: 0,
        })
    }

    fn ___getslice__(&self, slice: &PySlice) -> PyResult<PyObject> {
        let mut ids: Vec<usize> = vec![];
        {
            let dut = DUT.lock().unwrap();
            let grp = dut._get_pin_group(self.model_id, &self.name).unwrap();
            let indices = slice.indices((grp.pin_ids.len() as i32).into())?;

            let mut i = indices.start;
            if indices.step > 0 {
                while i < indices.stop {
                    ids.push(grp.pin_ids[i as usize]);
                    i += indices.step;
                }
            } else {
                while i > indices.stop {
                    ids.push(grp.pin_ids[i as usize]);
                    i += indices.step;
                }
            }
        }
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PinCollection::from_ids_unchecked(ids, None))?.to_object(py))
    }
}

#[pyproto]
impl pyo3::class::mapping::PyMappingProtocol for PinGroup {
    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        ListLikeAPI::__getitem__(self, idx)
    }

    fn __len__(&self) -> PyResult<usize> {
        ListLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinGroup {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<ListLikeIter> {
        ListLikeAPI::__iter__(&*slf)
    }
}
